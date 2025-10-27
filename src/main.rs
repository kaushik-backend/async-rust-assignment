mod app;
mod db;
mod models;
mod controllers;
mod services;
mod routes;
mod middlewares;


use dotenvy::dotenv;
use std::env;
use axum::serve;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = app::build_app().await;

    println!("Server running at http://{}", addr);
    serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
        .await
        .unwrap();
}
