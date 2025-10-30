use axum::Router;

use crate::db;
use crate::routes::{user_routes, vehicle_routes};

pub async fn build_app() -> Router {
    // let task_db = db::connect_task_collection().await;
    let user_db = db::connect_user_collection().await;
    let vehicle_db = db::connect_vehicle_collection().await;

    // let task_router = task_routes::create_task_routes(task_db);
    let user_router = user_routes::user_routes(user_db);
    let vehicle_router = vehicle_routes::vehicle_routes(vehicle_db);

    Router::new()
        .nest("/api/v1", user_router)
        .nest("/api/v1", vehicle_router)
}
