use std::sync::Arc;
use tokio::sync::Mutex;
use mongodb::{options::ClientOptions, Client, Collection};
use crate::models::{user_model::User,vehicle_model::Vehicle};
use std::env;

// pub type TaskDb = Arc<Mutex<Collection<Task>>>;
pub type UserDb = Arc<Mutex<Collection<User>>>;
pub type VehicleDb = Arc<Mutex<Collection<Vehicle>>>;

async fn get_client() -> Client {
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client_options = ClientOptions::parse(&uri).await.unwrap();
    Client::with_options(client_options).unwrap()
}

// pub async fn connect_task_collection() -> TaskDb {
//     let db_name = env::var("DATABASE_NAME").unwrap_or("async_rust_db".to_string());
//     let client = get_client().await;
//     let db = client.database(&db_name);
//     let collection = db.collection::<Task>("tasks");
//     Arc::new(Mutex::new(collection))
// }

pub async fn connect_user_collection() -> UserDb {
    let db_name = env::var("DATABASE_NAME").unwrap_or("async_rust_db".to_string());
    let client = get_client().await;
    let db = client.database(&db_name);
    let collection = db.collection::<User>("users");
    Arc::new(Mutex::new(collection))
}

pub async fn connect_vehicle_collection() -> VehicleDb {  
    let db_name = env::var("DATABASE_NAME").unwrap_or("async_rust_db".to_string());
    let client = get_client().await;
    let db = client.database(&db_name);
    let collection = db.collection::<Vehicle>("vehicles");
    Arc::new(Mutex::new(collection))
}
