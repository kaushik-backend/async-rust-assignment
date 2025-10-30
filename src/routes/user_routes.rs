use axum::{Router, routing::{post,put}};
use crate::controllers::user_controller::{register_handler, login_handler,update_user_handler};
use crate::db::UserDb;

pub fn user_routes(db: UserDb) -> Router {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/user/:id", put(update_user_handler))
        .with_state(db)
}
