use axum::{
    Router,
    routing::{post, put},
};
use crate::controllers::vehicle_controller::{create_vehicle_handler, update_vehicle_handler};
use crate::db::VehicleDb;

pub fn vehicle_routes(db: VehicleDb) -> Router {
    Router::new()
        .route("/vehicle", post(create_vehicle_handler))
        .route("/vehicle/:id", put(update_vehicle_handler))
        .with_state(db)
}
