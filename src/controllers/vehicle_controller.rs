use axum::{
    extract::{Multipart, Path as AxPath, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    db::VehicleDb,
    middlewares::auth_middleware::{require_role, AuthUser},
    models::{user_model::UserRole, vehicle_model::CreateVehicle},
    services::vehicle_service::{create_vehicle, update_vehicle},
};

/// POST /vehicles
/// Accepts multipart/form-data:
/// - make (text)
/// - model (text)
/// - year (text)
/// - files[] (file(s), optional)
pub async fn create_vehicle_handler(
    State(db): State<VehicleDb>,
    AuthUser { user_id, .. }: AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut make = String::new();
    let mut model = String::new();
    let mut year = String::new();
    let mut file_paths: Vec<String> = vec![];

    println!("Parsing vehicle multipart form...");

    while let Ok(Some( field)) = multipart.next_field().await {
        let field_name_raw = field.name().unwrap_or("unknown").to_string();
        let field_name = field_name_raw.trim().to_lowercase();

        println!("Received field: '{}'", field_name);

        match field_name.as_str() {
            "make" => {
                if let Ok(text) = field.text().await {
                    make = text.trim().to_string();
                }
            }
            "model" => {
                if let Ok(text) = field.text().await {
                    model = text.trim().to_string();
                }
            }
            "year" => {
                if let Ok(text) = field.text().await {
                    year = text.trim().to_string();
                }
            }
            "files" | "files[]" | "file" => {
                let upload_dir = "./uploads/vehicles";
                tokio::fs::create_dir_all(upload_dir).await.unwrap();

                let original_name = field.file_name().unwrap_or("file").to_string();
                let safe_name = sanitize_filename(&original_name);
                let file_path = format!("{}/{}_{}", upload_dir, Uuid::new_v4(), safe_name);

                let bytes = field.bytes().await.unwrap();
                let mut file = tokio::fs::File::create(&file_path).await.unwrap();
                file.write_all(&bytes).await.unwrap();

                println!("Uploaded vehicle file: {}", file_path);
                file_paths.push(file_path);
            }
            _ => println!("Ignoring unknown field: {}", field_name),
        }
    }

    if make.is_empty() || model.is_empty() || year.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required fields: make/model/year" })),
        );
    }

    let payload = CreateVehicle { make, model, year };

    match create_vehicle(&db, user_id, payload, Some(file_paths)).await {
        Ok(vehicle) => (
            StatusCode::CREATED,
            Json(json!({ "message": "Vehicle created", "vehicle": vehicle })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}

/// PUT /vehicles/:id
/// Only Admin can update vehicle records.
pub async fn update_vehicle_handler(
    State(db): State<VehicleDb>,
    AuthUser { role, .. }: AuthUser,
    AxPath(id): AxPath<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Require Admin role
    if let Err((code, msg)) = require_role(
        &AuthUser {
            user_id: "".to_string(),
            role: role.clone(),
        },
        &[UserRole::Admin],
    ) {
        return (code, Json(json!({ "error": msg })));
    }

    let mut make = String::new();
    let mut model = String::new();
    let mut year = String::new();
    let mut file_paths: Vec<String> = vec![];

    while let Ok(Some( field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_lowercase();
        match field_name.as_str() {
            "make" => {
                if let Ok(text) = field.text().await {
                    make = text;
                }
            }
            "model" => {
                if let Ok(text) = field.text().await {
                    model = text;
                }
            }
            "year" => {
                if let Ok(text) = field.text().await {
                    year = text;
                }
            }
            "files" | "files[]" | "file" => {
                let upload_dir = "./uploads/vehicles";
                tokio::fs::create_dir_all(upload_dir).await.unwrap();

                let safe_name = sanitize_filename(field.file_name().unwrap_or("file"));
                let file_path = format!("{}/{}_{}", upload_dir, Uuid::new_v4(), safe_name);

                let bytes = field.bytes().await.unwrap();
                let mut file = tokio::fs::File::create(&file_path).await.unwrap();
                file.write_all(&bytes).await.unwrap();

                file_paths.push(file_path);
            }
            _ => (),
        }
    }

    let payload = CreateVehicle { make, model, year };

    match update_vehicle(&db, &id, payload, Some(file_paths)).await {
        Ok(vehicle) => (
            StatusCode::OK,
            Json(json!({ "message": "Vehicle updated successfully", "vehicle": vehicle })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        ),
    }
}

/// Small sanitizer for filenames
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
        .collect::<String>()
}
