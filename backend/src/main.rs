use application::{
    customer::ports::CustomerRepository, ownership::ports::VehicleOwnershipRepository,
    vehicle::ports::VehicleRepository,
};
use axum::{
    Router,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::{get, post},
};
use infrastructure::customer_repository::InMemoryCustomerRepository;
use infrastructure::vehicle_ownership_repository::InMemoryVehicleOwnershipRepository;
use infrastructure::vehicle_repository::InMemoryVehicleRepository;
mod error;
mod routers;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    customer_repository: Arc<dyn CustomerRepository>,
    vehicle_repository: Arc<dyn VehicleRepository>,
    vehicle_ownership_repository: Arc<dyn VehicleOwnershipRepository>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST]);

    let state = AppState {
        customer_repository: Arc::new(InMemoryCustomerRepository::new()),
        vehicle_repository: Arc::new(InMemoryVehicleRepository::new()),
        vehicle_ownership_repository: Arc::new(InMemoryVehicleOwnershipRepository::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/customers", post(routers::customer::create_customer))
        .route("/customers/{id}", get(routers::customer::get_customer))
        .route("/vehicles", post(routers::vehicle::create_vehicle))
        .route("/vehicles/{id}", get(routers::vehicle::get_vehicle))
        .route(
            "/vehicles/{vehicle_id}/ownerships",
            post(routers::vehicle_ownership::create_vehicle_ownership),
        )
        .layer(cors)
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
