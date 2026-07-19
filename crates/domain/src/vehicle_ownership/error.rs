//! Domain errors of the `VehicleOwnership` aggregate.
//!
//! Доменные ошибки агрегата `VehicleOwnership`.

use thiserror::Error;

use crate::vehicle_ownership::state::OwnershipStatusKind;

/// Business-rule violations of the `VehicleOwnership` aggregate.
///
/// Every variant is a refusal, not a failure: the aggregate was asked to do
/// something the domain does not permit. Callers should translate these into a
/// client-visible rejection rather than retrying — retrying will not help,
/// because nothing about the request will have changed.
///
/// Ошибки бизнес-правил агрегата `VehicleOwnership`.
///
/// Каждый вариант — отказ, а не сбой: агрегат попросили сделать то, что домен
/// не допускает. Вызывающая сторона должна преобразовать их в видимое клиенту
/// отклонение, а не повторять запрос — повтор не поможет, поскольку в запросе
/// ничего не изменится.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OwnershipError {
    /// An ownership was started while an open one already exists for the vehicle.
    ///
    /// Raised from `start` when the eligibility snapshot reports the vehicle is
    /// occupied. Note that "open" includes `PendingVerification`, so this fires
    /// even when the existing claim has not been confirmed yet. Typically
    /// surfaced to the client as `409 Conflict`.
    ///
    /// Владение создаётся при уже существующем открытом владении на этот автомобиль.
    ///
    /// Возникает в `start`, когда снимок пригодности сообщает, что автомобиль
    /// занят. «Открытым» считается и `PendingVerification`, поэтому ошибка
    /// срабатывает даже при неподтверждённом существующем притязании. Обычно
    /// показывается клиенту как `409 Conflict`.
    #[error("для этого автомобиля уже существует активная запись о владении")]
    ActiveOwnershipAlreadyExists,

    /// The command is not permitted in the current status.
    ///
    /// Raised by `verify` and `end` for transitions the state machine forbids —
    /// for example ending a record that was never verified, or verifying one
    /// that has already terminated. Idempotent repeats are *not* reported here:
    /// they return `NoChange` instead.
    ///
    /// Команда невозможна в текущем статусе.
    ///
    /// Возникает в `verify` и `end` для переходов, запрещённых машиной
    /// состояний, — например, завершение записи, которая никогда не была
    /// подтверждена, или подтверждение уже завершённой. Идемпотентные повторы
    /// *не* попадают сюда: они возвращают `NoChange`.
    #[error("статус {0} не допускает данную операцию")]
    StatusDoesNotAllow(OwnershipStatusKind),

    /// The period's end date precedes its start date.
    ///
    /// Raised by `end` when the supplied timestamp is earlier than
    /// `started_at`. In practice this points at a clock problem — a skewed
    /// node, or a backdated command — rather than at user input, so it is worth
    /// investigating rather than simply reporting.
    ///
    /// Дата завершения периода раньше даты начала.
    ///
    /// Возникает в `end`, когда переданная временная метка раньше
    /// `started_at`. На практике это указывает на проблему с часами — сбитый
    /// узел или команду задним числом, — а не на пользовательский ввод,
    /// поэтому случай стоит расследовать, а не просто сообщать о нём.
    #[error("дата завершения периода владения не может быть раньше даты начала")]
    PeriodEndBeforeStart,
}
