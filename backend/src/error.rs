//! HTTP representation of backend API errors.
//!
//! This module separates internal application-layer errors from the public REST
//! API contract: responses expose only an HTTP status, a machine-readable error
//! code, and a safe user-facing message.
//!
//! The separation is a security boundary as much as a design one. An
//! application error may carry a driver message, a connection string fragment
//! or a poisoned-lock description; none of that may reach a client. Every
//! conversion here therefore *discards* the inner error's text and substitutes
//! a fixed, reviewed message.
//!
//! HTTP-представление ошибок API бэкенда.
//!
//! Модуль отделяет внутренние ошибки слоя приложения от публичного контракта
//! REST API: ответы раскрывают только HTTP-статус, машиночитаемый код ошибки и
//! безопасное сообщение для пользователя.
//!
//! Это разделение является не столько архитектурной, сколько защитной границей.
//! Ошибка приложения может нести сообщение драйвера, фрагмент строки
//! подключения или описание отравленной блокировки; ничего из этого не должно
//! попасть к клиенту. Поэтому каждое преобразование здесь *отбрасывает* текст
//! вложенной ошибки и подставляет фиксированное проверенное сообщение.

use application::error::{ApplicationError, RepositoryError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::vehicle_ownership::OwnershipError;
use serde::Serialize;

/// JSON body of an error response.
///
/// Тело JSON-ответа с ошибкой.
#[derive(Serialize)]
struct ErrorBody {
    /// Stable machine-readable error code for API clients.
    ///
    /// Clients are expected to branch on this rather than on `message`, which
    /// may be reworded without notice.
    ///
    /// Стабильный машиночитаемый код ошибки для клиентов API.
    ///
    /// Клиенты должны ветвиться по нему, а не по `message`, формулировка
    /// которого может измениться без предупреждения.
    error: String,
    /// Safe message that does not expose internal storage details.
    ///
    /// Безопасное сообщение, не раскрывающее внутренних деталей хранилища.
    message: String,
}

/// Error type that axum can convert directly into an HTTP response.
///
/// Used as the common error type in handlers. Because `From<ApplicationError>`
/// is implemented, handlers can return `Result<_, ApiError>` and use the `?`
/// operator on application-layer calls — the mapping to a status code happens
/// implicitly, in one reviewed place, instead of being repeated per route.
///
/// Тип ошибки, который axum может напрямую преобразовать в HTTP-ответ.
///
/// Используется как общий тип ошибки в обработчиках. Благодаря реализации
/// `From<ApplicationError>` обработчики могут возвращать `Result<_, ApiError>` и
/// применять оператор `?` к вызовам слоя приложения — сопоставление с кодом
/// состояния происходит неявно, в одном проверенном месте, а не повторяется в
/// каждом маршруте.
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    /// Creates a `500 Internal Server Error` response.
    ///
    /// Used for infrastructure failures whose details must not leak into the
    /// public API response.
    ///
    /// Создаёт ответ `500 Internal Server Error`.
    ///
    /// Используется для инфраструктурных сбоев, детали которых не должны
    /// попадать в публичный ответ API.
    pub fn internal_server_error(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorBody {
                error: "internal_server_error".to_string(),
                message: message.to_string(),
            },
        }
    }

    /// Creates a `409 Conflict` response.
    ///
    /// Suitable for optimistic-locking conflicts and concurrent changes to the
    /// same entity — cases where the request was valid but the current state
    /// refuses it, and a retry after reloading might succeed.
    ///
    /// Создаёт ответ `409 Conflict`.
    ///
    /// Подходит для конфликтов оптимистичной блокировки и конкурентных
    /// изменений одной сущности — случаев, когда запрос корректен, но текущее
    /// состояние его отклоняет, и повтор после перечитывания может завершиться
    /// успешно.
    pub fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "conflict".to_string(),
                message: message.to_string(),
            },
        }
    }

    /// Creates a `422 Unprocessable Entity` response.
    ///
    /// Used when the request is syntactically valid but violates application or
    /// domain business rules — distinct from `409`, in that retrying the same
    /// request unchanged can never succeed.
    ///
    /// Создаёт ответ `422 Unprocessable Entity`.
    ///
    /// Используется, когда запрос синтаксически корректен, но нарушает
    /// бизнес-правила приложения или домена — в отличие от `409`, повтор того
    /// же неизменного запроса здесь не может завершиться успешно.
    pub fn unprocessable_entity(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ErrorBody {
                error: "unprocessable_entity".to_string(),
                message: message.to_string(),
            },
        }
    }

    /// Creates a `404 Not Found` response.
    ///
    /// Used when the requested resource does not exist. Built by route handlers
    /// from an `Ok(None)` lookup, since absence is not an application error.
    ///
    /// Создаёт ответ `404 Not Found`.
    ///
    /// Используется, когда запрошенный ресурс не существует. Создаётся
    /// обработчиками маршрутов из результата поиска `Ok(None)`, поскольку
    /// отсутствие не является ошибкой приложения.
    pub fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ErrorBody {
                error: "not_found".to_string(),
                message: message.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    /// Converts the error into a JSON response shaped as
    /// `{ "error": "...", "message": "..." }`.
    ///
    /// Преобразует ошибку в JSON-ответ вида
    /// `{ "error": "...", "message": "..." }`.
    fn into_response(self) -> Response {
        let body = Json(self.body);
        (self.status, body).into_response()
    }
}

impl From<ApplicationError> for ApiError {
    /// Maps application-layer errors to public HTTP errors.
    ///
    /// Domain errors remain generalized: specific internal variants do not leak
    /// into the API contract until they have stable public error codes. Note
    /// that the inner error's own message is never forwarded — each arm picks a
    /// fixed string, so a driver or lock diagnostic cannot reach a client.
    ///
    /// The `match` is exhaustive on purpose. Adding a variant to
    /// `ApplicationError` or `OwnershipError` breaks this build, which forces a
    /// deliberate decision about the status code rather than letting a new
    /// failure mode fall through to a generic `500`.
    ///
    /// Сопоставляет ошибки слоя приложения с публичными HTTP-ошибками.
    ///
    /// Доменные ошибки остаются обобщёнными: конкретные внутренние варианты не
    /// попадают в контракт API, пока у них нет стабильных публичных кодов
    /// ошибок. Обратите внимание, что собственное сообщение вложенной ошибки
    /// никогда не передаётся дальше — каждая ветвь выбирает фиксированную
    /// строку, поэтому диагностика драйвера или блокировки не может дойти до
    /// клиента.
    ///
    /// `match` намеренно исчерпывающий. Добавление варианта в
    /// `ApplicationError` или `OwnershipError` ломает сборку, что вынуждает
    /// осознанно выбрать код состояния, а не позволяет новому виду отказа
    /// незаметно превратиться в обобщённый `500`.
    fn from(e: ApplicationError) -> Self {
        match e {
            ApplicationError::Repository(repo_err) => match repo_err {
                RepositoryError::VersionConflict { .. } => {
                    ApiError::conflict("Conflict occurred while processing the request.")
                }
                // The underlying message is dropped deliberately: it can name
                // internal storage details that must not reach an API client.
                //
                // Исходное сообщение намеренно отбрасывается: оно может
                // раскрывать внутренние детали хранилища, которые не должны
                // дойти до клиента API.
                RepositoryError::StorageFailure(_) => ApiError::internal_server_error(
                    "Internal server error occurred while accessing the repository.",
                ),
            },
            ApplicationError::Ownership(ownership_error) => match ownership_error {
                OwnershipError::ActiveOwnershipAlreadyExists => {
                    ApiError::conflict("Active ownership already exists.")
                }

                OwnershipError::PeriodEndBeforeStart => ApiError::unprocessable_entity(
                    "Ownership period end date is before start date.",
                ),

                OwnershipError::StatusDoesNotAllow(status) => ApiError::conflict(&format!(
                    "Ownership status '{}' does not allow this operation.",
                    status
                )),
            },
        }
    }
}
