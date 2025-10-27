use axum::{Router, routing::post};
use crate::controllers::user_controller::{register_handler, login_handler};
use crate::db::UserDb;

pub fn user_routes(db: UserDb) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .with_state(db)
}
