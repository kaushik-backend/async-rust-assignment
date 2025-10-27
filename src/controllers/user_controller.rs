use axum::{Json, extract::State};
use crate::{db::UserDb, models::user_model::{RegisterUser, LoginUser}};
use crate::services::user_service::{register_user, login_user};
use serde_json::json;

pub async fn register_handler(
    State(db): State<UserDb>,
    Json(payload): Json<RegisterUser>,
) -> Json<serde_json::Value> {
    match register_user(&db, payload).await {
        Ok(user) => Json(json!({ "message": "User registered", "email": user.email })),
        Err(e) => Json(json!({ "error": e })),
    }
}

pub async fn login_handler(
    State(db): State<UserDb>,
    Json(payload): Json<LoginUser>,
) -> Json<serde_json::Value> {
    match login_user(&db, payload).await {
        Ok(token) => Json(json!({ "token": token })),
        Err(e) => Json(json!({ "error": e })),
    }
}
