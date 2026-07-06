use application::{customer::ports::CustomerRepository, vehicle::ports::VehicleRepository};
use axum::{
    Router,
    routing::{get, post},
};
use infrastructure::customer_repository::InMemoryCustomerRepository;
use infrastructure::vehicle_repository::InMemoryVehicleRepository;
mod error;
mod routers;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    customer_repository: Arc<dyn CustomerRepository>,
    vehicle_repository: Arc<dyn VehicleRepository>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        customer_repository: Arc::new(InMemoryCustomerRepository::new()),
        vehicle_repository: Arc::new(InMemoryVehicleRepository::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/customers", post(routers::customer::create_customer))
        .route("/vehicles", post(routers::vehicle::create_vehicle))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!("Backend listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Faild to bind TCP listener");

    axum::serve(listener, app).await.expect("Server failed");
}

async fn health() -> &'static str {
    "OK"
}
