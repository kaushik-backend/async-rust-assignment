use axum::{extract::{Path, State}, Json};
use crate::{
    db::TaskDb,
    models::task_model::{Task, CreateTask},
    services::task_service,
    middlewares::auth_middleware::AuthUser, 
};
use mongodb::bson::oid::ObjectId;

pub async fn create_task(
    State(db): State<TaskDb>,
    AuthUser { user_id }: AuthUser, 
    Json(payload): Json<CreateTask>,
) -> Json<Task> {
    let user_obj_id = ObjectId::parse_str(&user_id).unwrap();
    Json(task_service::create_task(&db, payload, user_obj_id).await)
}

pub async fn get_tasks(State(db): State<TaskDb>) -> Json<Vec<Task>> {
    Json(task_service::get_all_tasks(&db,None).await)
}


pub async fn get_task(
    State(db): State<TaskDb>,
    Path(id): Path<String>,
) -> Json<Option<Task>> {
    Json(task_service::get_task_by_id(&db, &id, None).await)
}


pub async fn update_task(
    State(db): State<TaskDb>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<String>,
    Json(payload): Json<CreateTask>,
) -> Json<Option<Task>> {
    Json(task_service::update_task(&db, &id, payload, &user_id).await)
}

pub async fn delete_task(
    State(db): State<TaskDb>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<String>,
) -> Json<&'static str> {
    let deleted = task_service::delete_task(&db, &id, &user_id).await;
    if deleted {
        Json("Task deleted")
    } else {
        Json("Not authorized or Task not found")
    }
}
