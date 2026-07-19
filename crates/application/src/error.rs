//! Error types of the application layer.
//!
//! Separates two fundamentally different kinds of failure: a *domain* refusal,
//! where the request was understood and rejected on business grounds, and an
//! *infrastructure* failure, where the request could not be carried out at all.
//! Callers act on them differently — the first is reported to the user, the
//! second may be retried or alerted on — so the type system keeps them apart.
//!
//! Типы ошибок слоя приложения.
//!
//! Разделяет два принципиально разных вида отказа: *доменный* отказ, когда
//! запрос понят и отклонён по бизнес-основаниям, и *инфраструктурный* сбой,
//! когда запрос вообще не удалось выполнить. Вызывающая сторона реагирует на
//! них по-разному — первое сообщается пользователю, второе может быть повторено
//! или отправлено в мониторинг, — поэтому система типов их различает.

use domain::vehicle_ownership::OwnershipError;
use thiserror::Error;

/// Top-level error type of the application layer.
///
/// Handlers return this so that a single `?` can propagate both repository
/// failures and domain refusals. Each variant is `#[error(transparent)]`: the
/// application layer adds no message of its own, because it has no information
/// the inner error lacks, and wrapping would only obscure the real cause.
///
/// The HTTP layer converts this into `ApiError`, which is where the decision
/// about status codes and client-visible wording is made.
///
/// Корневой тип ошибок слоя приложения.
///
/// Обработчики возвращают его, чтобы один `?` мог пробросить и сбои
/// репозитория, и доменные отказы. Каждый вариант помечен
/// `#[error(transparent)]`: слой приложения не добавляет собственного
/// сообщения, поскольку не располагает сведениями сверх вложенной ошибки, а
/// обёртка лишь скрыла бы настоящую причину.
///
/// HTTP-слой преобразует этот тип в `ApiError` — именно там принимается решение
/// о кодах состояния и видимых клиенту формулировках.
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// The operation could not be carried out due to a storage problem.
    ///
    /// See [`RepositoryError`] for the specific cause.
    ///
    /// Операцию не удалось выполнить из-за проблемы с хранилищем.
    ///
    /// Конкретную причину см. в [`RepositoryError`].
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    /// A vehicle-ownership business rule was violated.
    ///
    /// The request was well-formed and reached the domain, which refused it.
    /// Retrying will not help; the caller should report the refusal.
    ///
    /// Нарушено бизнес-правило владения автомобилем.
    ///
    /// Запрос был корректным и дошёл до домена, который его отклонил. Повтор не
    /// поможет; вызывающая сторона должна сообщить об отказе.
    #[error(transparent)]
    Ownership(#[from] OwnershipError),
}

/// Failures originating from a repository adapter.
///
/// Declared in the application layer rather than in `infrastructure` because it
/// is part of the port contract: every adapter — in-memory, PostgreSQL, or a
/// test mock — must express its failures in these terms. That is what keeps
/// handlers free of storage-specific error types and lets an adapter be
/// swapped without touching a single handler.
///
/// Сбои, исходящие от адаптера репозитория.
///
/// Объявлены в слое приложения, а не в `infrastructure`, поскольку являются
/// частью контракта порта: любой адаптер — в памяти, PostgreSQL или тестовый
/// мок — обязан выражать свои сбои в этих терминах. Именно это избавляет
/// обработчики от специфичных для хранилища типов ошибок и позволяет заменить
/// адаптер, не трогая ни одного обработчика.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryError {
    /// Optimistic-locking conflict: the aggregate changed under the writer.
    ///
    /// The stored version did not match the one the caller expected, meaning
    /// another writer committed in between. The write is refused rather than
    /// applied, since applying it would silently discard the other change.
    ///
    /// Also produced when the same aggregate is created twice: the second
    /// create arrives at version 1 while the store already holds version 1 and
    /// expects 2, so a duplicate surfaces as a conflict rather than needing a
    /// separate "already exists" variant.
    ///
    /// Callers should reload the aggregate and retry, or report `409 Conflict`.
    ///
    /// Конфликт оптимистичной блокировки: агрегат изменился «под» писателем.
    ///
    /// Сохранённая версия не совпала с ожидаемой вызывающей стороной, то есть в
    /// промежутке зафиксировался другой писатель. Запись отклоняется, а не
    /// применяется, поскольку её применение молча отбросило бы чужое изменение.
    ///
    /// Возникает также при повторном создании одного и того же агрегата:
    /// второе создание приходит с версией 1, тогда как в хранилище уже лежит
    /// версия 1 и ожидается 2 — поэтому дубликат проявляется как конфликт и не
    /// требует отдельного варианта «уже существует».
    ///
    /// Вызывающей стороне следует перечитать агрегат и повторить попытку либо
    /// сообщить `409 Conflict`.
    #[error("optimistic lock conflict: expected version {expected}, found {actual}")]
    VersionConflict {
        /// Version the repository required for the write to be safe.
        ///
        /// Версия, которую репозиторий требовал для безопасной записи.
        expected: u64,
        /// Version the caller actually presented.
        ///
        /// Версия, фактически предъявленная вызывающей стороной.
        actual: u64,
    },

    /// The underlying storage failed.
    ///
    /// Covers connection loss, a poisoned lock, a driver error — anything that
    /// prevented the operation from being attempted or completed. The message
    /// is diagnostic and must not be forwarded to API clients verbatim, as it
    /// can disclose internal details.
    ///
    /// Отказ нижележащего хранилища.
    ///
    /// Охватывает потерю соединения, отравленную блокировку, ошибку драйвера —
    /// всё, что помешало выполнить или завершить операцию. Сообщение носит
    /// диагностический характер и не должно передаваться клиентам API дословно,
    /// так как может раскрыть внутренние детали.
    #[error("storage failure: {0}")]
    StorageFailure(String),
}
