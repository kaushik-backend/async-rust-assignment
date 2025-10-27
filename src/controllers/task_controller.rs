use axum::{extract::{Path, State}, Json};
use crate::{
    db::Db,
    models::task_model::{Task, CreateTask},
    services::task_service,
};

pub async fn create_task(State(db): State<Db>, Json(payload): Json<CreateTask>) -> Json<Task> {
    Json(task_service::create_task(&db, payload).await)
}

pub async fn get_tasks(State(db): State<Db>) -> Json<Vec<Task>> {
    Json(task_service::get_all_tasks(&db).await)
}

pub async fn get_task(State(db): State<Db>, Path(id): Path<String>) -> Json<Option<Task>> {
    Json(task_service::get_task_by_id(&db, &id).await)
}

pub async fn update_task(State(db): State<Db>, Path(id): Path<String>, Json(payload): Json<CreateTask>) -> Json<Option<Task>> {
    Json(task_service::update_task(&db, &id, payload).await)
}

pub async fn delete_task(State(db): State<Db>, Path(id): Path<String>) -> Json<&'static str> {
    let deleted = task_service::delete_task(&db, &id).await;
    if deleted { Json("Task deleted") } else { Json("Task not found") }
}
