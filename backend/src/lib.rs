//! Testable HTTP application assembly for PitGO.
//!
//! The binary entry point starts a server; this library assembles the same
//! router for both the server and HTTP integration tests.
//!
//! Тестируемая сборка HTTP-приложения PitGO.
//!
//! Бинарная точка входа запускает сервер; эта библиотека собирает тот же
//! роутер и для сервера, и для HTTP-интеграционных тестов.

use std::sync::Arc;

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
use tower_http::cors::CorsLayer;

mod error;
mod routers;
#[cfg(test)]
mod tests;

/// Shared application state injected into HTTP handlers.
///
/// The fields remain private so handlers depend only on repository ports while
/// callers choose a state through explicit constructors.
///
/// Общее состояние, внедряемое в HTTP-обработчики.
///
/// Поля остаются приватными: обработчики зависят только от портов репозиториев,
/// а вызывающий код выбирает состояние через явные конструкторы.
#[derive(Clone)]
pub struct AppState {
    pub(crate) customer_repository: Arc<dyn CustomerRepository>,
    pub(crate) vehicle_repository: Arc<dyn VehicleRepository>,
    pub(crate) vehicle_ownership_repository: Arc<dyn VehicleOwnershipRepository>,
}

impl AppState {
    /// Creates isolated in-memory state for local development or one test.
    ///
    /// Every call allocates fresh repositories. This makes integration tests
    /// independent: no test can observe data created by another test.
    ///
    /// Создаёт изолированное состояние в памяти для локальной разработки или
    /// одного теста.
    ///
    /// Каждый вызов создаёт новые репозитории. Поэтому интеграционные тесты
    /// независимы: один тест не может увидеть данные, созданные другим.
    #[must_use]
    pub fn in_memory() -> Self {
        Self {
            customer_repository: Arc::new(InMemoryCustomerRepository::new()),
            vehicle_repository: Arc::new(InMemoryVehicleRepository::new()),
            vehicle_ownership_repository: Arc::new(InMemoryVehicleOwnershipRepository::new()),
        }
    }
}

/// Assembles the complete HTTP application around the supplied state.
///
/// CORS is part of the application boundary, so it belongs here rather than
/// only in `main`: tests and production exercise the same public behavior.
///
/// Собирает полное HTTP-приложение вокруг переданного состояния.
///
/// CORS — часть границы приложения, поэтому он находится здесь, а не только в
/// `main`: тесты и production используют одинаковое публичное поведение.
pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_headers([CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST]);

    Router::new()
        .route("/health", get(health))
        .route("/customers", post(routers::customer::create_customer))
        .route("/customers/{id}", get(routers::customer::get_customer))
        .route("/vehicles", post(routers::vehicle::create_vehicle))
        .route("/vehicles/{id}", get(routers::vehicle::get_vehicle))
        .route(
            "/vehicles/{vehicle_id}/ownerships",
            post(routers::vehicle_ownership::create_vehicle_ownership),
        )
        .route(
            "/vehicles/{vehicle_id}/ownerships/{ownership_id}",
            get(routers::vehicle_ownership::get_vehicle_ownership),
        )
        .layer(cors)
        .with_state(state)
}

/// Handles `GET /health`.
async fn health() -> &'static str {
    "OK"
}
