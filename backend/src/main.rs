//! PitGO backend binary entry point.
//!
//! The reusable HTTP application is assembled in the `backend` library so the
//! same routes can be exercised in production and integration tests.
//!
//! Точка входа бинарного backend-приложения PitGO.
//!
//! Переиспользуемое HTTP-приложение собирается в библиотеке `backend`, поэтому
//! production и интеграционные тесты используют одни и те же маршруты.

use std::net::SocketAddr;

/// Starts the HTTP server.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = backend::app(backend::AppState::in_memory());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::info!(%addr, "Backend listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app).await.expect("Server failed");
}
