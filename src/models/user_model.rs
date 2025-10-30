use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq,Eq)]
pub enum  UserRole {
    Admin,
    User,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub profile_image: Option<String>, 
    #[serde(default)]
    pub role:UserRole,
    #[serde(default)]
    pub created_at: Option<DateTime>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}
