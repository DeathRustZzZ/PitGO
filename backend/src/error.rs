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
//! # Error code table
//!
//! | Source                                          | HTTP status | `error` code                       |
//! |--------------------------------------------------|-------------|-------------------------------------|
//! | `RepositoryError::VersionConflict`                | 409         | `version_conflict`                  |
//! | `RepositoryError::AlreadyExists`                  | 409         | `already_exists`                    |
//! | `RepositoryError::StorageFailure`                 | 500         | `internal`                          |
//! | `OwnershipError::ActiveOwnershipAlreadyExists`    | 409         | `active_ownership_already_exists`   |
//! | `OwnershipError::StatusDoesNotAllow`               | 409         | `ownership_status_does_not_allow`   |
//! | `OwnershipError::PeriodEndBeforeStart`             | 422         | `ownership_period_invalid`          |
//! | not found (customer, from routers/customer.rs)    | 404         | `customer_not_found`                |
//! | not found (vehicle, from routers/vehicle.rs)       | 404         | `vehicle_not_found`                 |
//!
//! HTTP-представление ошибок API бэкенда.
//!
//! Модуль отделяет внутренние ошибки слоя приложения от публичного контракта
//! REST API: ответы раскрывают только HTTP-статус, машиночитаемый код ошибки и
//! безопасное сообщение для пользователя.
//!
//! # Таблица кодов ошибок
//!
//! | Источник                                          | HTTP-статус | код `error`                        |
//! |----------------------------------------------------|-------------|--------------------------------------|
//! | `RepositoryError::VersionConflict`                  | 409         | `version_conflict`                   |
//! | `RepositoryError::AlreadyExists`                    | 409         | `already_exists`                     |
//! | `RepositoryError::StorageFailure`                   | 500         | `internal`                           |
//! | `OwnershipError::ActiveOwnershipAlreadyExists`      | 409         | `active_ownership_already_exists`    |
//! | `OwnershipError::StatusDoesNotAllow`                 | 409         | `ownership_status_does_not_allow`    |
//! | `OwnershipError::PeriodEndBeforeStart`               | 422         | `ownership_period_invalid`           |
//! | not found (клиент, из routers/customer.rs)          | 404         | `customer_not_found`                 |
//! | not found (автомобиль, из routers/vehicle.rs)        | 404         | `vehicle_not_found`                  |
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
use domain::vehicle_ownership::state::OwnershipStatusKind;
use serde::Serialize;

/// JSON body of an error response.
///
/// Тело JSON-ответа с ошибкой.
#[derive(Serialize)]
struct ErrorBody {
    /// Stable machine-readable error code for API clients.
    ///
    /// Clients are expected to branch on this rather than on `message`, which
    /// may be reworded without notice. Deliberately not derived automatically
    /// from the internal error enum (e.g. via `Display`/`strum`): doing so
    /// would tie this public contract to internal variant names, so every code
    /// is instead a fixed literal chosen in this file.
    ///
    /// Стабильный машиночитаемый код ошибки для клиентов API.
    ///
    /// Клиенты должны ветвиться по нему, а не по `message`, формулировка
    /// которого может измениться без предупреждения. Намеренно не выводится
    /// автоматически из внутреннего перечисления ошибок (например, через
    /// `Display`/`strum`): это связало бы публичный контракт с именами
    /// внутренних вариантов, поэтому каждый код — фиксированный литерал,
    /// выбранный в этом файле.
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
/// There are no generic constructors (no `conflict(message)`,
/// `not_found(message)`, …): every factory below is specific to one error case
/// and bakes in both the status and the code. This makes it impossible to
/// build an `ApiError` with a status/code pair that was not deliberately
/// chosen for that case.
///
/// Тип ошибки, который axum может напрямую преобразовать в HTTP-ответ.
///
/// Используется как общий тип ошибки в обработчиках. Благодаря реализации
/// `From<ApplicationError>` обработчики могут возвращать `Result<_, ApiError>` и
/// применять оператор `?` к вызовам слоя приложения — сопоставление с кодом
/// состояния происходит неявно, в одном проверенном месте, а не повторяется в
/// каждом маршруте.
///
/// Здесь нет обобщённых конструкторов (нет `conflict(message)`,
/// `not_found(message)`, …): каждая фабрика ниже специфична для одного случая
/// ошибки и несёт в себе и статус, и код. Это делает невозможным создание
/// `ApiError` со связкой статус/код, не выбранной осознанно для этого случая.
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    /// Creates the `500 Internal Server Error` / `internal` response for a
    /// storage failure.
    ///
    /// The failing repository's own message is never forwarded — it may
    /// contain driver diagnostics or connection details.
    ///
    /// Создаёт ответ `500 Internal Server Error` / `internal` для сбоя
    /// хранилища.
    ///
    /// Собственное сообщение сбойного репозитория никогда не передаётся дальше
    /// — оно может содержать диагностику драйвера или детали подключения.
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorBody {
                error: "internal".to_string(),
                message: "Internal server error occurred while processing the request.".to_string(),
            },
        }
    }

    /// Creates the `409 Conflict` / `version_conflict` response.
    ///
    /// Covers optimistic-locking conflicts only: an existing aggregate was
    /// modified concurrently, so the caller's version is stale. A duplicate
    /// create is a different failure and maps to `already_exists` instead —
    /// the distinction matters to clients, because a stale write is worth
    /// retrying after a reload while a taken id never is.
    ///
    /// Создаёт ответ `409 Conflict` / `version_conflict`.
    ///
    /// Покрывает только конфликты оптимистичной блокировки: существующий
    /// агрегат был изменён конкурентно, поэтому версия у вызывающей стороны
    /// устарела. Повторное создание — другой вид отказа, он отображается в
    /// `already_exists`. Различие существенно для клиентов: устаревшую запись
    /// имеет смысл повторить после перечитывания, а занятый идентификатор —
    /// никогда.
    fn version_conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "version_conflict".to_string(),
                message: "Conflict occurred while processing the request.".to_string(),
            },
        }
    }

    /// Creates the `409 Conflict` / `active_ownership_already_exists` response.
    ///
    /// Создаёт ответ `409 Conflict` / `active_ownership_already_exists`.
    fn active_ownership_already_exists() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "active_ownership_already_exists".to_string(),
                message: "Active ownership already exists for this vehicle.".to_string(),
            },
        }
    }

    /// Creates the `409 Conflict` / `ownership_status_does_not_allow` response.
    ///
    /// The current status is interpolated into `message` since ownership
    /// statuses are public, but `error` stays a fixed literal regardless of
    /// which status was rejected — clients must branch on the code, not on the
    /// wording.
    ///
    /// Создаёт ответ `409 Conflict` / `ownership_status_does_not_allow`.
    ///
    /// Текущий статус подставляется в `message`, так как статусы владения
    /// публичны, но `error` остаётся фиксированным литералом независимо от
    /// того, какой статус был отклонён — клиенты должны ветвиться по коду, а
    /// не по формулировке.
    fn ownership_status_does_not_allow(status: OwnershipStatusKind) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "ownership_status_does_not_allow".to_string(),
                message: format!("Ownership status '{status}' does not allow this operation."),
            },
        }
    }

    /// Creates the `422 Unprocessable Entity` / `ownership_period_invalid`
    /// response.
    ///
    /// Создаёт ответ `422 Unprocessable Entity` / `ownership_period_invalid`.
    fn ownership_period_invalid() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ErrorBody {
                error: "ownership_period_invalid".to_string(),
                message: "Ownership period end date is before start date.".to_string(),
            },
        }
    }

    /// Creates the `404 Not Found` / `customer_not_found` response.
    ///
    /// Built by the customer router from an `Ok(None)` lookup, since absence is
    /// not an application error.
    ///
    /// Создаёт ответ `404 Not Found` / `customer_not_found`.
    ///
    /// Создаётся маршрутом клиента из результата поиска `Ok(None)`, поскольку
    /// отсутствие не является ошибкой приложения.
    pub fn customer_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ErrorBody {
                error: "customer_not_found".to_string(),
                message: "Customer not found.".to_string(),
            },
        }
    }

    /// Creates the `404 Not Found` / `vehicle_not_found` response.
    ///
    /// Built by the vehicle router from an `Ok(None)` lookup, since absence is
    /// not an application error.
    ///
    /// Создаёт ответ `404 Not Found` / `vehicle_not_found`.
    ///
    /// Создаётся маршрутом автомобиля из результата поиска `Ok(None)`,
    /// поскольку отсутствие не является ошибкой приложения.
    pub fn vehicle_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ErrorBody {
                error: "vehicle_not_found".to_string(),
                message: "Vehicle not found.".to_string(),
            },
        }
    }

    /// Creates the `409 Conflict` / `already_exists` response.
    ///
    /// Returned when a repository detects that an aggregate with the same id
    /// was already persisted. Unlike `version_conflict` (a concurrent-write
    /// collision on an existing aggregate), this code signals that the id
    /// itself is taken — retrying the same create will never succeed.
    ///
    /// Создаёт ответ `409 Conflict` / `already_exists`.
    ///
    /// Возвращается, когда репозиторий обнаруживает, что агрегат с тем же
    /// идентификатором уже был сохранён. В отличие от `version_conflict`
    /// (коллизии конкурентной записи существующего агрегата), этот код
    /// сигнализирует о том, что сам идентификатор занят — повтор того же
    /// создания никогда не завершится успехом.
    fn already_exists() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "already_exists".to_string(),
                message: "An aggregate with this identifier already exists.".to_string(),
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
    /// deliberate decision about the status code and error code rather than
    /// letting a new failure mode fall through to a generic `500`.
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
    /// осознанно выбрать код состояния и код ошибки, а не позволяет новому виду
    /// отказа незаметно превратиться в обобщённый `500`.
    fn from(e: ApplicationError) -> Self {
        match e {
            ApplicationError::Repository(repo_err) => match repo_err {
                RepositoryError::VersionConflict { .. } => ApiError::version_conflict(),

                // The underlying message is dropped deliberately: it can name
                // internal storage details that must not reach an API client.
                //
                // Исходное сообщение намеренно отбрасывается: оно может
                // раскрывать внутренние детали хранилища, которые не должны
                // дойти до клиента API.
                RepositoryError::AlreadyExists => ApiError::already_exists(),

                // The underlying message is dropped deliberately: it can name
                // internal storage details that must not reach an API client.
                //
                // Исходное сообщение намеренно отбрасывается: оно может
                // раскрывать внутренние детали хранилища, которые не должны
                // дойти до клиента API.
                RepositoryError::StorageFailure(_) => ApiError::internal(),
            },
            ApplicationError::Ownership(ownership_error) => match ownership_error {
                OwnershipError::ActiveOwnershipAlreadyExists => {
                    ApiError::active_ownership_already_exists()
                }

                OwnershipError::PeriodEndBeforeStart => ApiError::ownership_period_invalid(),

                OwnershipError::StatusDoesNotAllow(status) => {
                    ApiError::ownership_status_does_not_allow(status)
                }
            },
        }
    }
}
