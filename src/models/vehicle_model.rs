use serde::{Deserialize, Serialize};
use mongodb::bson::{oid::ObjectId, DateTime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub user_id: ObjectId,
    pub make: String,
    pub model: String,
    pub year: String,

    pub files: Option<Vec<String>>, 

    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVehicle {
    pub make: String,
    pub model: String,
    pub year: String,
}
