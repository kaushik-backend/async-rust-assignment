use axum::Router;
use crate::{db, routes::task_routes};

pub async fn build_app() -> Router {
    let db = db::connect().await;
    task_routes::create_task_routes(db)
}
