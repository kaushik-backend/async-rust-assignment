use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
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
