// use std::path::Path;
use uuid::Uuid;

use axum::{
    extract::{Multipart, Path as AxPath, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::{
    db::UserDb,
    middlewares::auth_middleware::{AuthUser,require_role},
    models::user_model::{LoginUser, RegisterUser, UserRole},
    services::user_service::{login_user, register_user, update_user},
};

/// POST /register
/// Accepts multipart/form-data with fields:
///  - name (text)
///  - email (text)
///  - password (text)
///  - profileImage (file, optional)
pub async fn register_handler(
    State(db): State<UserDb>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut name = String::new();
    let mut email = String::new();
    let mut password = String::new();
    let mut role: Option<UserRole> = None;
    let mut profile_image_path: Option<String> = None;

    println!("Parsing multipart form...");

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name_raw = field.name().unwrap_or("unknown").to_string();
        let field_name = field_name_raw.trim().to_lowercase();

        println!("Received field: '{}'", field_name);

        match field_name.as_str() {
            "name" => {
                if let Ok(text) = field.text().await {
                    name = text.trim().to_string();
                    println!("Parsed name: {}", name);
                }
            }
            "email" => {
                if let Ok(text) = field.text().await {
                    email = text.trim().to_string();
                    println!("Parsed email: {}", email);
                }
            }
            "password" => {
                if let Ok(text) = field.text().await {
                    password = text.trim().to_string();
                    println!("Parsed password: {}", password);
                }
            }
            "role" => {
                if let Ok(text) = field.text().await {
                    let parsed = match text.to_lowercase().as_str() {
                        "admin" => Some(UserRole::Admin),
                        "user" => Some(UserRole::User),
                        _ => None,
                    };
                    role = parsed.or(Some(UserRole::User));
                }
            }
            "profile_image" => {
                let upload_dir = "./uploads";
                tokio::fs::create_dir_all(upload_dir).await.unwrap();

                let file_name = field.file_name().unwrap_or("file").to_string();
                let file_path = format!("{}/{}_{}", upload_dir, Uuid::new_v4(), file_name);

                let bytes = field.bytes().await.unwrap();
                let mut file = tokio::fs::File::create(&file_path).await.unwrap();
                file.write_all(&bytes).await.unwrap();

                profile_image_path = Some(file_path.clone());
                println!(" Saved image to {}", file_path);
            }
            _ => println!("Ignoring unknown field: {}", field_name),
        }
    }

    println!(" name={} email={} password={}", name, email, password);

    if name.is_empty() || email.is_empty() || password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields: name/email/password" })),
        );
    }

    let payload = RegisterUser {
        name,
        email,
        password,
        role,
    };
    match register_user(&db, payload, profile_image_path).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(json!({ "message": "User created", "user": user })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}

/// POST /login
pub async fn login_handler(
    State(db): State<UserDb>,
    Json(payload): Json<LoginUser>,
) -> Json<serde_json::Value> {
    match login_user(&db, payload).await {
        Ok(token_struct) => Json(json!({ "token": token_struct })),
        Err(e) => {
            eprintln!("login_user error: {}", e);
            Json(json!({ "error": e }))
        }
    }
}

/// PUT /user/:id
/// PUT /user/:id
pub async fn update_user_handler(
    State(db): State<UserDb>,
    AuthUser { user_id, role }: AuthUser,
    AxPath(id): AxPath<String>,
    Json(payload): Json<RegisterUser>,
) -> Json<serde_json::Value> {
    // Rule: Admins/SuperAdmins can update anyone
    // Regular users can only update their own profile
    if user_id != id {
        if let Err(err) = require_role(
            &AuthUser {
                user_id: user_id.clone(),
                role: role.clone(),
            },
            &[UserRole::Admin],
        ) {
            return Json(json!({
                "success": false,
                "message": err.1
            }));
        }
    }

    match update_user(&db, &id, payload, &user_id).await {
        Ok(user) => Json(json!({
            "success": true,
            "message": "User updated successfully",
            "data": {
                "id": user.id.map(|i| i.to_hex()),
                "name": user.name,
                "email": user.email,
                "role": user.role,
                "profile_image": user.profile_image,
            }
        })),
        Err(err) => {
            eprintln!("update_user error: {}", err);
            Json(json!({
                "success": false,
                "message": err
            }))
        }
    }
}

/// sanitizer to avoid problematic characters in filenames
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect::<String>()
}
