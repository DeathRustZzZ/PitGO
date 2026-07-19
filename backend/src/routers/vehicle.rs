//! HTTP routes for vehicle operations.
//!
//! This module translates vehicle API requests into application-layer commands
//! and returns transport-level responses.
//!
//! HTTP-маршруты для операций с автомобилями.
//!
//! Модуль преобразует запросы API автомобилей в команды слоя приложения и
//! возвращает ответы транспортного уровня.

use crate::AppState;
use crate::error::ApiError;
use application::vehicle::commands::CreateVehicleCommand;
use application::vehicle::handlers::{CreateVehicleHandler, GetVehicleHandler};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request body for creating a new vehicle.
///
/// Тело запроса на создание нового автомобиля.
#[derive(Deserialize)]
pub struct CreateVehicleRequest {
    /// Client-provided vehicle identifier.
    ///
    /// Supplied by the caller rather than generated server-side, which makes a
    /// retried create idempotent: the repository's version check rejects the
    /// duplicate instead of producing a second vehicle.
    ///
    /// Идентификатор автомобиля, передаваемый вызывающей стороной.
    ///
    /// Передаётся клиентом, а не генерируется на сервере, что делает повторное
    /// создание идемпотентным: проверка версии в репозитории отклоняет дубликат
    /// вместо создания второго автомобиля.
    pub vehicle_id: Uuid,
}

/// Response body for a vehicle.
///
/// A dedicated wire type rather than the `Vehicle` aggregate: the aggregate's
/// fields are private, and keeping the response separate means the API shape is
/// chosen deliberately instead of following whatever the domain happens to hold.
///
/// Тело ответа с данными автомобиля.
///
/// Отдельный тип «для провода», а не агрегат `Vehicle`: поля агрегата
/// приватны, а обособленный ответ означает, что форма API выбирается осознанно,
/// а не следует за тем, что домен хранит в данный момент.
#[derive(Serialize)]
pub struct VehicleResponse {
    /// Identifier of the vehicle.
    ///
    /// Идентификатор автомобиля.
    pub vehicle_id: Uuid,
    /// Lifecycle status rendered as a string, e.g. `"Draft"` or `"Active"`.
    ///
    /// Статус жизненного цикла в виде строки, например `"Draft"` или `"Active"`.
    pub status: String,
    /// When the vehicle was created.
    ///
    /// Момент создания автомобиля.
    pub created_at: DateTime<Utc>,
    /// When the vehicle was last modified.
    ///
    /// Момент последнего изменения автомобиля.
    pub updated_at: DateTime<Utc>,
}

/// Handles `POST /vehicles`.
///
/// Builds a `CreateVehicleCommand`, delegates business behavior to the
/// application layer, and maps any application error into `ApiError`. A repeat
/// create for the same id surfaces as `409 Conflict` via the optimistic-locking
/// check.
///
/// Обрабатывает `POST /vehicles`.
///
/// Собирает `CreateVehicleCommand`, делегирует бизнес-поведение слою приложения
/// и преобразует любую ошибку приложения в `ApiError`. Повторное создание с тем
/// же идентификатором проявляется как `409 Conflict` благодаря проверке
/// оптимистичной блокировки.
pub async fn create_vehicle(
    State(state): State<AppState>,
    Json(body): Json<CreateVehicleRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = CreateVehicleCommand {
        vehicle_id: body.vehicle_id.into(),
    };

    let repository = state.vehicle_repository;

    let handler = CreateVehicleHandler::new(repository);

    handler.handle(cmd).await?;
    Ok((
        StatusCode::CREATED,
        Json("Vehicle created successfully".to_string()),
    ))
}

/// Handles `GET /vehicles/{id}`.
///
/// Translating a missing vehicle into `404` happens here rather than in the
/// application layer: absence is an ordinary lookup result, and only the
/// transport layer is in a position to call it an HTTP error.
///
/// Обрабатывает `GET /vehicles/{id}`.
///
/// Преобразование отсутствующего автомобиля в `404` выполняется здесь, а не в
/// слое приложения: отсутствие — обычный результат поиска, и только
/// транспортный слой вправе назвать его HTTP-ошибкой.
pub async fn get_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, ApiError> {
    let handler = GetVehicleHandler::new(state.vehicle_repository);

    match handler.handle(id.into()).await? {
        Some(vehicle) => Ok(Json(VehicleResponse {
            vehicle_id: vehicle.id().into(),
            status: vehicle.status().kind().to_string(),
            created_at: vehicle.created_at(),
            updated_at: vehicle.updated_at(),
        })),
        None => Err(ApiError::not_found("Vehicle not found.")),
    }
}
