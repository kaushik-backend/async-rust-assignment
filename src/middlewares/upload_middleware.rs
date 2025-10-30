use axum::{extract::Multipart, http::StatusCode, response::{Json}};
use serde_json::json;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use std::collections::HashMap;

/// Handle multiple file uploads and return a map of field names to file paths
pub async fn handle_multiple_uploads(
    mut multipart: Multipart,
    fields: &[&str], // List of field names to look for
) -> Result<HashMap<String, String>, (StatusCode, Json<serde_json::Value>)> {
    let mut file_paths: HashMap<String, String> = HashMap::new();
    let upload_dir = "./uploads";

    // Ensure upload directory exists
    if !Path::new(upload_dir).exists() {
        tokio::fs::create_dir_all(upload_dir).await.unwrap();
    }

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        // Use as_ref() to avoid moving the field
        if let Some(name) = field.name().map(|n| n.to_string()) {
            // Check if the field name is in the allowed fields
            if fields.contains(&name.as_str()) {
                let file_name = field
                    .file_name()
                    .unwrap_or("default.jpg")
                    .to_string();
                let file_path = format!("{}/{}", upload_dir, Uuid::new_v4().to_string() + &file_name);
                let bytes = field.bytes().await.unwrap_or_default();

                let mut file = tokio::fs::File::create(&file_path)
                    .await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "File save failed"}))))?;
                file.write_all(&bytes).await.unwrap();

                // Store the file path with the field name as the key
                file_paths.insert(name, file_path);
            }
        }
    }

    Ok(file_paths)
}
