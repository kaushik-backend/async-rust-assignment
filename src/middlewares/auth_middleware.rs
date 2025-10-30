use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::user_model::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: UserRole,
    pub exp: usize,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
    pub role: UserRole,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

        let secret = env::var("JWT_SECRET").unwrap_or("mysecret".to_string());
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string()))?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
            role: token_data.claims.role,
        })
    }
}

/// Middleware helper for role-based routes
pub fn require_role(user: &AuthUser, allowed: &[UserRole]) -> Result<(), (StatusCode, String)> {
    if allowed.contains(&user.role) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            format!("Forbidden: Role {:?} not allowed", user.role),
        ))
    }
}
