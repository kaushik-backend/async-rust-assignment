use crate::models::user_model::{RegisterUser, LoginUser, User};
use crate::db::UserDb;
use bcrypt::{hash, verify, DEFAULT_COST};
use mongodb::bson::{doc, oid::ObjectId, DateTime};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn register_user(db: &UserDb, user: RegisterUser) -> Result<User, String> {
    let hashed = hash(&user.password, DEFAULT_COST).map_err(|e| e.to_string())?;
    let new_user = User {
        id: Some(ObjectId::new()),
        name: user.name,
        email: user.email,
        password: hashed,
        created_at: Some(DateTime::now()),
    };

    let collection = db.lock().await;
    let result = collection.insert_one(&new_user, None).await;
    drop(collection);

    result.map_err(|e| e.to_string())?;
    Ok(new_user)
}

pub async fn login_user(db: &UserDb, creds: LoginUser) -> Result<String, String> {
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
        exp,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|e| e.to_string())?;

    Ok(token)
}
