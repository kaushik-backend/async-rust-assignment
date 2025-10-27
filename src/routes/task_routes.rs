use axum::routing::{get, post, put, delete};
use axum::Router;
use crate::controllers::task_controller::*;

use crate::db::TaskDb;

pub fn create_task_routes(db: TaskDb) -> Router {
    Router::new()
        .route("/tasks", post(create_task).get(get_tasks))
        .route("/tasks/:id", get(get_task).put(update_task).delete(delete_task))
        .with_state(db)
}
