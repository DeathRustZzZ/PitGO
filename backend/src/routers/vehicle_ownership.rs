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
use application::ownership::handlers::{GetVehicleOwnershipHandler, StartVehicleOwnershipHandler};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use domain::vehicle_ownership::OwnershipType;
use domain::vehicle_ownership::state::OwnershipStatusKind;
use serde::{Deserialize, Serialize};
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
#[derive(Deserialize, Serialize)]
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

    fn from_domain(ownership_type: &OwnershipType) -> Self {
        // This explicit mapping is intentionally exhaustive. A new domain
        // variant must force a deliberate public-API decision rather than
        // silently changing the wire contract.
        //
        // Это явное сопоставление намеренно исчерпывающее. Новый вариант домена
        // должен потребовать осознанного решения для публичного API, а не
        // незаметно изменить контракт «на проводе».
        match ownership_type {
            OwnershipType::Private => Self::Private,
            OwnershipType::Company => Self::Company,
            OwnershipType::Leasing => Self::Leasing,
            OwnershipType::Fleet => Self::Fleet,
            OwnershipType::Unknown => Self::Unknown,
        }
    }
}

/// Response body for a vehicle ownership record.
///
/// This DTO owns the HTTP contract: enums are serialized as snake_case without
/// adding serde concerns to the domain model.
///
/// Этот DTO владеет HTTP-контрактом: enum-значения сериализуются в snake_case
/// без добавления serde-деталей в доменную модель.
#[derive(Serialize)]
pub struct VehicleOwnershipResponse {
    pub ownership_id: Uuid,
    pub vehicle_id: Uuid,
    pub owner_customer_id: Uuid,
    pub ownership_type: OwnershipTypeDto,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn ownership_status_to_wire(status: OwnershipStatusKind) -> String {
    // `Display` on the domain status produces PascalCase for diagnostics. The
    // API deliberately uses snake_case, matching the ownership-type request
    // DTO and keeping the wire vocabulary stable.
    //
    // `Display` доменного статуса даёт PascalCase для диагностики. API
    // намеренно использует snake_case: он согласован с request-DTO типа
    // владения и сохраняет словарь «на проводе» стабильным.
    match status {
        OwnershipStatusKind::PendingVerification => "pending_verification",
        OwnershipStatusKind::Active => "active",
        OwnershipStatusKind::Ended => "ended",
    }
    .to_string()
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

/// Handles `GET /vehicles/{vehicle_id}/ownerships/{ownership_id}`.
///
/// A found ownership is returned only when it belongs to the vehicle in the
/// nested URL. An absent ownership and an ownership of another vehicle both
/// produce the same `404`: the resource does not exist at that URL, and the
/// uniform response prevents ownership enumeration across vehicles.
///
/// Обрабатывает `GET /vehicles/{vehicle_id}/ownerships/{ownership_id}`.
///
/// Найденное владение возвращается только для автомобиля из вложенного URL.
/// Отсутствующая запись и запись другого автомобиля одинаково дают `404`:
/// ресурса по этому URL нет, а единый ответ не позволяет перебирать владения
/// других автомобилей.
pub async fn get_vehicle_ownership(
    State(state): State<AppState>,
    Path((vehicle_id, ownership_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VehicleOwnershipResponse>, ApiError> {
    let handler = GetVehicleOwnershipHandler::new(state.vehicle_ownership_repository);

    let Some(ownership) = handler.handle(ownership_id.into()).await? else {
        return Err(ApiError::ownership_not_found());
    };

    // `vehicle_id` is part of the resource address, not a domain rule. Keep
    // this scope check in the HTTP adapter and deliberately reuse the missing
    // response so that another vehicle's ownership is not disclosed.
    //
    // `vehicle_id` — часть адреса ресурса, а не доменное правило. Проверка
    // области остаётся в HTTP-адаптере и намеренно использует тот же ответ об
    // отсутствии, чтобы не раскрывать владение другого автомобиля.
    if ownership.vehicle_id() != vehicle_id.into() {
        return Err(ApiError::ownership_not_found());
    }

    let period = ownership.period();
    Ok(Json(VehicleOwnershipResponse {
        ownership_id: ownership.id().into(),
        vehicle_id: ownership.vehicle_id().into(),
        owner_customer_id: ownership.owner_customer_id().into(),
        ownership_type: OwnershipTypeDto::from_domain(ownership.ownership_type()),
        status: ownership_status_to_wire(ownership.status().kind()),
        started_at: period.started_at,
        ended_at: period.ended_at,
        created_at: ownership.created_at(),
        updated_at: ownership.updated_at(),
    }))
}
