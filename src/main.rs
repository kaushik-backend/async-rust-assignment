use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime as ChronoDateTime, NaiveDate, Utc};
use dotenvy::dotenv;
use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    options::ClientOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

//
// ====================== MODELS ======================
//

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub title: String,
    pub description: Option<String>,

    #[serde(default = "default_status")]
    pub status: String,

    #[serde(default = "default_priority")]
    pub priority: String,

    pub assignee: Option<String>,
    pub due_date: Option<DateTime>,

    #[serde(default = "now")]
    pub created_at: DateTime,

    #[serde(default = "now")]
    pub updated_at: DateTime,
}

fn now() -> DateTime {
    DateTime::now()
}

fn default_status() -> String {
    "todo".to_string()
}

fn default_priority() -> String {
    "medium".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>, // 👈 changed from BSON::DateTime → String
}

type Db = Arc<Mutex<Collection<Task>>>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Get MongoDB details from environment
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DATABASE_NAME").unwrap_or_else(|_| "async_rust_db".to_string());

    // Parse Mongo connection
    let client_options = ClientOptions::parse(&uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database(&db_name);
    let collection = db.collection::<Task>("tasks");
    let shared_db = Arc::new(Mutex::new(collection));

    // Define routes
    let app = Router::new()
        .route("/tasks", post(create_task).get(get_tasks))
        .route(
            "/tasks/:id",
            get(get_task).put(update_task).delete(delete_task),
        )
        .with_state(shared_db);

    // Get port from environment (Render provides PORT)
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    println!("Server running on http://{}", addr);

    // Listen on all interfaces (important for Render)
    axum::serve(
        tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Failed to bind to address"),
        app,
    )
    .await
    .unwrap();
}

//
// ====================== HELPERS ======================
//

fn parse_due_date(due_date: &Option<String>) -> Option<DateTime> {
    if let Some(d) = due_date {
        if let Ok(parsed) = ChronoDateTime::parse_from_rfc3339(d) {
            return Some(DateTime::from_millis(parsed.timestamp_millis()));
        }
        if let Ok(parsed) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            let dt = parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
            return Some(DateTime::from_millis(dt.timestamp_millis()));
        }
    }
    None
}

//
// ====================== ROUTES ======================
//

// POST /tasks
async fn create_task(State(db): State<Db>, Json(payload): Json<CreateTask>) -> Json<Task> {
    let new_task = Task {
        id: None,
        title: payload.title,
        description: payload.description,
        status: payload.status.unwrap_or_else(default_status),
        priority: payload.priority.unwrap_or_else(default_priority),
        assignee: payload.assignee,
        due_date: parse_due_date(&payload.due_date),
        created_at: now(),
        updated_at: now(),
    };

    let collection = db.lock().await;
    let result = collection.insert_one(&new_task, None).await.unwrap();

    let mut created_task = new_task.clone();
    created_task.id = result.inserted_id.as_object_id();

    Json(created_task)
}

// GET /tasks
async fn get_tasks(State(db): State<Db>) -> Json<Vec<Task>> {
    let collection = db.lock().await;
    let mut cursor = collection.find(doc! {}, None).await.unwrap();
    let mut tasks = vec![];

    while let Some(task) = cursor.try_next().await.unwrap() {
        tasks.push(task);
    }

    Json(tasks)
}

// GET /tasks/:id
async fn get_task(State(db): State<Db>, Path(id): Path<String>) -> Json<Option<Task>> {
    let collection = db.lock().await;
    let obj_id = ObjectId::parse_str(&id).unwrap();
    let task = collection
        .find_one(doc! { "_id": obj_id }, None)
        .await
        .unwrap();
    Json(task)
}

// PUT /tasks/:id
async fn update_task(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(payload): Json<CreateTask>,
) -> Json<Option<Task>> {
    let collection = db.lock().await;
    let obj_id = ObjectId::parse_str(&id).unwrap();

    collection
        .update_one(
            doc! { "_id": obj_id },
            doc! {
                "$set": {
                    "title": payload.title,
                    "description": payload.description,
                    "status": payload.status.unwrap_or_else(default_status),
                    "priority": payload.priority.unwrap_or_else(default_priority),
                    "assignee": payload.assignee,
                    "due_date": parse_due_date(&payload.due_date),
                    "updated_at": now(),
                }
            },
            None,
        )
        .await
        .unwrap();

    let updated = collection
        .find_one(doc! { "_id": obj_id }, None)
        .await
        .unwrap();
    Json(updated)
}

// DELETE /tasks/:id
async fn delete_task(State(db): State<Db>, Path(id): Path<String>) -> Json<&'static str> {
    let collection = db.lock().await;
    let obj_id = ObjectId::parse_str(&id).unwrap();
    collection
        .delete_one(doc! { "_id": obj_id }, None)
        .await
        .unwrap();
    Json("Task deleted")
}
