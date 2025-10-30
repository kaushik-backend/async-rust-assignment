use crate::db::UserDb;
use crate::models::user_model::{LoginUser, RegisterUser, User, UserRole};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::{doc, oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role:UserRole,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role:UserRole,
} 

pub async fn register_user(
    db: &UserDb,
    user: RegisterUser,
    profile_image_path: Option<String>, 
) -> Result<User, String> {
    let hashed = hash(&user.password, DEFAULT_COST).map_err(|e| e.to_string())?;
    let new_user = User {
        id: Some(ObjectId::new()),
        name: user.name,
        email: user.email,
        password: hashed,
        profile_image: profile_image_path,
        role:user.role.unwrap_or(UserRole::User),
        created_at: Some(DateTime::now()),
    };

    let collection = db.lock().await;
    collection
        .insert_one(&new_user, None)
        .await
        .map_err(|e| e.to_string())?;
    drop(collection);

    Ok(new_user)
}

pub async fn login_user(db: &UserDb, creds: LoginUser) -> Result<LoginResponse, String> {
    let collection = db.lock().await;
    let user = collection
        .find_one(doc! {"email": &creds.email}, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Invalid email or password")?;
    drop(collection);

    if !verify(&creds.password, &user.password).map_err(|e| e.to_string())? {
        return Err("Invalid email or password".to_string());
    }

    let secret = env::var("JWT_SECRET").unwrap_or("mysecret".to_string());
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.unwrap().to_hex(),
        role: user.role.clone(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| e.to_string())?;

    let user_response = UserResponse {
        id: user.id.unwrap().to_hex(),
        name: user.name.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
    };

    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

pub async fn update_user(
    db: &UserDb,
    id: &str,
    payload: RegisterUser,
    user_id: &str,
) -> Result<User, String> {
    // Ensure user can only update their own account (basic authorization)
    if id != user_id {
        return Err("Unauthorized: You can only update your own profile".to_string());
    }

    let obj_id = ObjectId::parse_str(id).map_err(|_| "Invalid user ID".to_string())?;
    let collection = db.lock().await;

    // Find existing user
    let existing_user = collection
        .find_one(doc! { "_id": &obj_id }, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("User not found")?;

    // Hash new password if changed
    let new_password = if payload.password != existing_user.password {
        hash(&payload.password, DEFAULT_COST).map_err(|e| e.to_string())?
    } else {
        existing_user.password
    };

    let updated_doc = doc! {
        "$set": {
            "name": &payload.name,
            "email": &payload.email,
            "password": &new_password,
            "updated_at": DateTime::now(),
        }
    };

    // Update user in DB
    collection
        .update_one(doc! { "_id": &obj_id }, updated_doc, None)
        .await
        .map_err(|e| e.to_string())?;

    // Fetch the updated user to return
    let updated_user = collection
        .find_one(doc! { "_id": &obj_id }, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Failed to fetch updated user")?;

    Ok(updated_user)
}

