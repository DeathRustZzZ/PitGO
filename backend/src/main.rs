use application::customer::ports::CustomerRepository;
use axum::{
    Router,
    routing::{get, post},
};
use infrastructure::InMemoryCustomerRepository;
mod routers;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    customer_repository: Arc<dyn CustomerRepository>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        customer_repository: Arc::new(InMemoryCustomerRepository::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/customers", post(routers::customer::create_customer))
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
