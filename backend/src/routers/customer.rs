use axum::{Json, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

/// Request body for creating a new customer
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub customer_id: Uuid,
}

/// Create a new customer
pub async fn create_customer(Json(body): Json<CreateCustomerRequest>) -> StatusCode {
    println!("Customer id {}", body.customer_id);
    StatusCode::CREATED
}
