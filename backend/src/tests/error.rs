//! Behavioral tests of the `ApplicationError` → `ApiError` contract.
//!
//! Each test drives `into_response()` and parses the JSON body, the same way a
//! real API client would. Status and code are read off the produced HTTP
//! response rather than off `ApiError`'s private fields, so what is asserted is
//! the wire contract and not the internal representation. Living outside
//! `error.rs` enforces that discipline: the private fields are simply not
//! reachable from here.
//!
//! Every row of the error-code table in `crate::error` has a test below. Adding
//! a row without adding a test leaves a public code unverified.
//!
//! Поведенческие тесты контракта `ApplicationError` → `ApiError`.
//!
//! Каждый тест прогоняет `into_response()` и разбирает JSON-тело — так же, как
//! это делал бы настоящий клиент API. Статус и код читаются из полученного
//! HTTP-ответа, а не из приватных полей `ApiError`, поэтому проверяется контракт
//! «на проводе», а не внутреннее представление. Размещение вне `error.rs`
//! закрепляет этот подход: приватные поля отсюда попросту недоступны.
//!
//! Каждой строке таблицы кодов ошибок в `crate::error` соответствует тест ниже.
//! Добавление строки без теста оставляет публичный код непроверенным.

use crate::error::ApiError;
use application::error::{ApplicationError, RepositoryError};
use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::vehicle_ownership::OwnershipError;
use domain::vehicle_ownership::state::OwnershipStatusKind;
use serde::Deserialize;

/// The error body as a client actually receives it.
///
/// Declared separately from `ErrorBody` in `crate::error` on purpose: that type
/// is private, and duplicating the shape here means a field rename in the
/// production type surfaces as a failing test instead of silently compiling.
///
/// Тело ошибки в том виде, в каком его получает клиент.
///
/// Объявлено отдельно от `ErrorBody` из `crate::error` намеренно: тот тип
/// приватный, а дублирование формы здесь означает, что переименование поля в
/// продуктивном типе проявится как падающий тест, а не скомпилируется молча.
#[derive(Deserialize)]
struct WireErrorBody {
    error: String,
    message: String,
}

/// Renders an `ApiError` the way axum would and decodes the result.
///
/// The status is taken from the response, not from the error value, so this
/// helper verifies the same bytes and headers a client would observe.
///
/// Преобразует `ApiError` так же, как это сделал бы axum, и разбирает результат.
///
/// Статус берётся из ответа, а не из значения ошибки, поэтому хелпер проверяет
/// те же байты и заголовки, которые увидел бы клиент.
async fn decode(err: ApiError) -> (StatusCode, WireErrorBody) {
    let response = err.into_response();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should be readable");
    let decoded: WireErrorBody =
        serde_json::from_slice(&body).expect("body should be valid JSON matching the contract");
    (status, decoded)
}

#[tokio::test]
async fn version_conflict_maps_to_409() {
    let err = ApiError::from(ApplicationError::Repository(
        RepositoryError::VersionConflict {
            expected: 2,
            actual: 1,
        },
    ));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "version_conflict");
}

/// A duplicate create and a stale write share status 409 but must not share a
/// code: a client retries the second and never the first.
///
/// Повторное создание и устаревшая запись имеют общий статус 409, но не должны
/// иметь общий код: второе клиент повторяет, первое — никогда.
#[tokio::test]
async fn already_exists_maps_to_409() {
    let err = ApiError::from(ApplicationError::Repository(RepositoryError::AlreadyExists));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "already_exists");
}

#[tokio::test]
async fn storage_failure_maps_to_500_internal() {
    let err = ApiError::from(ApplicationError::Repository(
        RepositoryError::StorageFailure("secret detail".to_string()),
    ));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.error, "internal");
}

/// Guards the security boundary: a repository's diagnostic text may name
/// internal storage details, and none of it may reach an API client.
///
/// Защищает границу безопасности: диагностический текст репозитория может
/// называть внутренние детали хранилища, и ничто из этого не должно дойти до
/// клиента API.
#[tokio::test]
async fn storage_failure_message_does_not_leak_internal_detail() {
    let err = ApiError::from(ApplicationError::Repository(
        RepositoryError::StorageFailure("secret detail".to_string()),
    ));

    let (_, body) = decode(err).await;

    assert!(
        !body.message.contains("secret detail"),
        "message must not leak the repository's internal diagnostic text"
    );
}

#[tokio::test]
async fn active_ownership_already_exists_maps_to_409() {
    let err = ApiError::from(ApplicationError::Ownership(
        OwnershipError::ActiveOwnershipAlreadyExists,
    ));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "active_ownership_already_exists");
}

/// The status name is interpolated into `message`, but the code must stay a
/// fixed literal — clients branch on the code, which cannot vary per status.
///
/// Имя статуса подставляется в `message`, но код обязан оставаться фиксированным
/// литералом: клиенты ветвятся по коду, и он не может меняться вместе со
/// статусом.
#[tokio::test]
async fn ownership_status_does_not_allow_maps_to_409() {
    let err = ApiError::from(ApplicationError::Ownership(
        OwnershipError::StatusDoesNotAllow(OwnershipStatusKind::Ended),
    ));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.error, "ownership_status_does_not_allow");
}

#[tokio::test]
async fn ownership_period_before_start_maps_to_422() {
    let err = ApiError::from(ApplicationError::Ownership(
        OwnershipError::PeriodEndBeforeStart,
    ));

    let (status, body) = decode(err).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body.error, "ownership_period_invalid");
}

#[tokio::test]
async fn customer_not_found_maps_to_404() {
    let (status, body) = decode(ApiError::customer_not_found()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "customer_not_found");
}

#[tokio::test]
async fn vehicle_not_found_maps_to_404() {
    let (status, body) = decode(ApiError::vehicle_not_found()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "vehicle_not_found");
}

#[tokio::test]
async fn ownership_not_found_maps_to_404() {
    let (status, body) = decode(ApiError::ownership_not_found()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.error, "ownership_not_found");
}
