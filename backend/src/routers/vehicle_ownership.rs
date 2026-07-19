//! HTTP routes for vehicle ownership operations.
//!
//! This module translates ownership API requests into application-layer
//! commands and returns transport-level responses.
//!
//! HTTP-маршруты для операций владения автомобилем.
//!
//! Модуль преобразует запросы API владения в команды слоя приложения и
//! возвращает ответы транспортного уровня.

use crate::AppState;
use crate::error::ApiError;
use application::ownership::commands::StartVehicleOwnershipCommand;
use application::ownership::handlers::StartVehicleOwnershipHandler;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use domain::vehicle_ownership::OwnershipType;
use serde::Deserialize;
use uuid::Uuid;

/// Request body for starting a vehicle ownership.
///
/// `vehicle_id` is absent here on purpose: it comes from the URL path, so the
/// resource being modified is identified in exactly one place and a body that
/// disagreed with the path could not arise.
///
/// Тело запроса на создание владения автомобилем.
///
/// `vehicle_id` здесь намеренно отсутствует: он берётся из пути URL, поэтому
/// изменяемый ресурс определяется ровно в одном месте, и тело запроса,
/// противоречащее пути, попросту не может возникнуть.
#[derive(Deserialize)]
pub struct CreateVehicleOwnershipRequest {
    /// Client-provided identifier for the new ownership record.
    ///
    /// Supplied by the caller so a retried request is idempotent.
    ///
    /// Идентификатор новой записи о владении, передаваемый клиентом.
    ///
    /// Передаётся вызывающей стороной, чтобы повторный запрос был идемпотентным.
    pub ownership_id: Uuid,
    /// Customer to be recorded as the owner.
    ///
    /// Клиент, который будет зафиксирован как владелец.
    pub owner_customer_id: Uuid,
    /// Kind of ownership relationship.
    ///
    /// Тип отношения владения.
    pub ownership_type: OwnershipTypeDto,
}

/// Wire representation of [`OwnershipType`].
///
/// A separate DTO rather than deserializing the domain enum directly. This
/// keeps the JSON contract — including its `snake_case` spelling — owned by the
/// transport layer, so renaming a domain variant cannot silently break API
/// clients. The conversion in [`OwnershipTypeDto::into_domain`] is where the
/// two vocabularies meet, and it fails to compile if either side gains a
/// variant the other lacks.
///
/// Представление [`OwnershipType`] «на проводе».
///
/// Отдельный DTO вместо прямой десериализации доменного перечисления. Это
/// оставляет JSON-контракт — вместе с его записью в `snake_case` — во владении
/// транспортного слоя, поэтому переименование доменного варианта не может
/// незаметно сломать клиентов API. Место встречи двух словарей —
/// [`OwnershipTypeDto::into_domain`]; преобразование не скомпилируется, если у
/// одной из сторон появится вариант, отсутствующий у другой.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTypeDto {
    /// Maps to [`OwnershipType::Private`].
    ///
    /// Соответствует [`OwnershipType::Private`].
    Private,
    /// Maps to [`OwnershipType::Company`].
    ///
    /// Соответствует [`OwnershipType::Company`].
    Company,
    /// Maps to [`OwnershipType::Leasing`].
    ///
    /// Соответствует [`OwnershipType::Leasing`].
    Leasing,
    /// Maps to [`OwnershipType::Fleet`].
    ///
    /// Соответствует [`OwnershipType::Fleet`].
    Fleet,
    /// Maps to [`OwnershipType::Unknown`].
    ///
    /// Соответствует [`OwnershipType::Unknown`].
    Unknown,
}

impl OwnershipTypeDto {
    /// Converts the wire value into its domain counterpart.
    ///
    /// Преобразует значение «с провода» в его доменный аналог.
    fn into_domain(self) -> OwnershipType {
        match self {
            Self::Private => OwnershipType::Private,
            Self::Company => OwnershipType::Company,
            Self::Leasing => OwnershipType::Leasing,
            Self::Fleet => OwnershipType::Fleet,
            Self::Unknown => OwnershipType::Unknown,
        }
    }
}

/// Handles `POST /vehicles/{vehicle_id}/ownerships`.
///
/// Builds a [`StartVehicleOwnershipCommand`] from the path and body, delegates
/// to the application layer, and maps any failure through [`ApiError`]. A
/// vehicle that already has an open ownership is refused by the domain and
/// surfaces here as `409 Conflict`.
///
/// Обрабатывает `POST /vehicles/{vehicle_id}/ownerships`.
///
/// Собирает [`StartVehicleOwnershipCommand`] из пути и тела запроса, делегирует
/// слою приложения и преобразует любой сбой через [`ApiError`]. Автомобиль, у
/// которого уже есть открытое владение, отклоняется доменом и проявляется здесь
/// как `409 Conflict`.
pub async fn create_vehicle_ownership(
    State(state): State<AppState>,
    Path(vehicle_id): Path<Uuid>,
    Json(body): Json<CreateVehicleOwnershipRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = StartVehicleOwnershipCommand {
        ownership_id: body.ownership_id.into(),
        vehicle_id: vehicle_id.into(),
        owner_customer_id: body.owner_customer_id.into(),
        ownership_type: body.ownership_type.into_domain(),
    };

    let repository = state.vehicle_ownership_repository;

    let handler = StartVehicleOwnershipHandler::new(repository);

    handler.handle(cmd).await?;
    Ok((
        StatusCode::CREATED,
        Json("Vehicle ownership started successfully".to_string()),
    ))
}
