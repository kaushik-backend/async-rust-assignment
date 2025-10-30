use mongodb::bson::{doc, oid::ObjectId, DateTime};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};

use crate::db::VehicleDb;
use crate::models::vehicle_model::{CreateVehicle, Vehicle};

/// Create a new vehicle record
pub async fn create_vehicle(
    db: &VehicleDb,
    user_id: String,
    payload: CreateVehicle,
    file_paths: Option<Vec<String>>,
) -> Result<Vehicle, String> {
    let user_obj_id = ObjectId::parse_str(&user_id).map_err(|_| "Invalid user ID".to_string())?;

    let new_vehicle = Vehicle {
        id: None,
        user_id: user_obj_id,
        make: payload.make,
        model: payload.model,
        year: payload.year,
        files: file_paths,
        created_at: Some(DateTime::now()),
        updated_at: Some(DateTime::now()),
    };

    let collection = db.lock().await;

    let insert_result = collection
        .insert_one(&new_vehicle, None)
        .await
        .map_err(|e| e.to_string())?;

    let inserted_id = insert_result
        .inserted_id
        .as_object_id()
        .ok_or("Failed to get inserted ID")?;

    let vehicle = collection
        .find_one(doc! { "_id": inserted_id }, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Vehicle not found after insert")?;

    Ok(vehicle)
}

/// Update a vehicle (Admin only)
pub async fn update_vehicle(
    db: &VehicleDb,
    id: &str,
    payload: CreateVehicle,
    file_paths: Option<Vec<String>>,
) -> Result<Vehicle, String> {
    let obj_id = ObjectId::parse_str(id).map_err(|_| "Invalid vehicle ID".to_string())?;

    let mut update_doc = doc! {
        "updated_at": DateTime::now()
    };

    if !payload.make.is_empty() {
        update_doc.insert("make", payload.make);
    }
    if !payload.model.is_empty() {
        update_doc.insert("model", payload.model);
    }
    if !payload.year.is_empty() {
        update_doc.insert("year", payload.year);
    }
    if let Some(paths) = file_paths {
        update_doc.insert("files", paths);
    }

    let collection = db.lock().await;

    let updated = collection
        .find_one_and_update(
            doc! { "_id": obj_id },
            doc! { "$set": update_doc },
            FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build(),
        )
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Vehicle not found")?;

    Ok(updated)
}
