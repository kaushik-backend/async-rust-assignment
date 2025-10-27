use std::sync::Arc;
use tokio::sync::Mutex;
use mongodb::{options::ClientOptions, Client, Collection};
use crate::models::task_model::Task;
use std::env;

pub type Db = Arc<Mutex<Collection<Task>>>;

pub async fn connect() -> Db {
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("DATABASE_NAME").unwrap_or("async_rust_db".to_string());

    let client_options = ClientOptions::parse(&uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database(&db_name);
    let collection = db.collection::<Task>("tasks");

    Arc::new(Mutex::new(collection))
}
