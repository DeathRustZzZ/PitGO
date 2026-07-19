//! HTTP routes for customer operations.
//!
//! This module translates customer API requests into application-layer commands
//! and returns transport-level responses.
//!
//! HTTP-маршруты для операций с клиентами.
//!
//! Модуль преобразует запросы API клиентов в команды слоя приложения и
//! возвращает ответы транспортного уровня.

use application::customer::commands::CreateCustomerCommand;
use application::customer::handlers::{CreateCustomerHandler, GetCustomerHandler};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;

/// Request body for creating a new customer.
///
/// Тело запроса на создание нового клиента.
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    /// Client-provided customer identifier.
    ///
    /// Supplied by the caller rather than generated server-side, which makes a
    /// retried create idempotent: the repository's version check rejects the
    /// duplicate instead of producing a second customer.
    ///
    /// Идентификатор клиента, передаваемый вызывающей стороной.
    ///
    /// Передаётся клиентом, а не генерируется на сервере, что делает повторное
    /// создание идемпотентным: проверка версии в репозитории отклоняет дубликат
    /// вместо создания второго клиента.
    pub customer_id: Uuid,
}

/// Response body for a customer.
///
/// A dedicated wire type rather than the `Customer` aggregate: the aggregate's
/// fields are private, and keeping the response separate means the API shape is
/// chosen deliberately instead of following whatever the domain happens to hold.
///
/// Тело ответа с данными клиента.
///
/// Отдельный тип «для провода», а не агрегат `Customer`: поля агрегата
/// приватны, а обособленный ответ означает, что форма API выбирается осознанно,
/// а не следует за тем, что домен хранит в данный момент.
#[derive(Serialize)]
pub struct CustomerResponse {
    /// Identifier of the customer.
    ///
    /// Идентификатор клиента.
    pub customer_id: Uuid,
    /// Lifecycle status rendered as a string, e.g. `"Draft"` or `"Active"`.
    ///
    /// Статус жизненного цикла в виде строки, например `"Draft"` или `"Active"`.
    pub status: String,
    /// When the customer was created.
    ///
    /// Момент создания клиента.
    pub created_at: DateTime<Utc>,
    /// When the customer was last modified.
    ///
    /// Момент последнего изменения клиента.
    pub updated_at: DateTime<Utc>,
}

/// Handles `POST /customers`.
///
/// Builds a `CreateCustomerCommand`, delegates business behavior to the
/// application layer, and maps any application error into `ApiError`. A repeat
/// create for the same id surfaces as `409 Conflict` via the optimistic-locking
/// check.
///
/// Обрабатывает `POST /customers`.
///
/// Собирает `CreateCustomerCommand`, делегирует бизнес-поведение слою
/// приложения и преобразует любую ошибку приложения в `ApiError`. Повторное
/// создание с тем же идентификатором проявляется как `409 Conflict` благодаря
/// проверке оптимистичной блокировки.
pub async fn create_customer(
    State(state): State<AppState>,
    Json(body): Json<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = CreateCustomerCommand {
        customer_id: body.customer_id.into(),
    };

    let repository = state.customer_repository;

    let handler = CreateCustomerHandler::new(repository);

    handler.handle(cmd).await?;
    Ok((
        StatusCode::CREATED,
        Json("Customer created successfully".to_string()),
    ))
}

/// Handles `GET /customers/{id}`.
///
/// Translating a missing customer into `404` happens here rather than in the
/// application layer: absence is an ordinary lookup result, and only the
/// transport layer is in a position to call it an HTTP error.
///
/// Обрабатывает `GET /customers/{id}`.
///
/// Преобразование отсутствующего клиента в `404` выполняется здесь, а не в слое
/// приложения: отсутствие — обычный результат поиска, и только транспортный
/// слой вправе назвать его HTTP-ошибкой.
pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, ApiError> {
    let handler = GetCustomerHandler::new(state.customer_repository);

    match handler.handle(id.into()).await? {
        Some(customer) => Ok(Json(CustomerResponse {
            customer_id: customer.id().into(),
            status: customer.status().kind().to_string(),
            created_at: customer.created_at(),
            updated_at: customer.updated_at(),
        })),
        None => Err(ApiError::customer_not_found()),
    }
}
