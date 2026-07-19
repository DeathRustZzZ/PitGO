//! Status and supporting types of the `VehicleOwnership` aggregate.
//!
//! Статус и вспомогательные типы агрегата `VehicleOwnership`.

use chrono::{DateTime, Utc};

/// Current state of an ownership record.
///
/// ```text
/// PendingVerification ─(verify)─→ Active ─(end)─→ Ended
/// ```
///
/// `Ended` is terminal. `Disputed` and `Rejected` are implemented by separate
/// tasks.
///
/// Текущее состояние записи о владении.
///
/// ```text
/// PendingVerification ─(verify)─→ Active ─(end)─→ Ended
/// ```
///
/// `Ended` — терминальное состояние. `Disputed` и `Rejected` реализуются
/// отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipStatus {
    /// Claimed but not yet confirmed. Already occupies the vehicle.
    ///
    /// Заявлено, но ещё не подтверждено. Уже занимает автомобиль.
    PendingVerification,
    /// Confirmed and in effect.
    ///
    /// Подтверждено и действует.
    Active,
    /// Terminated; terminal state. Does not occupy the vehicle.
    ///
    /// Завершено; терминальное состояние. Не занимает автомобиль.
    Ended,
}

/// Copyable discriminator of [`OwnershipStatus`] for use in error messages.
///
/// Lets an error name the offending status without borrowing from the
/// aggregate — errors outlive the `&self` that produced them.
///
/// Копируемый дискриминатор [`OwnershipStatus`] для сообщений об ошибках.
///
/// Позволяет ошибке назвать проблемный статус, не заимствуя данные агрегата:
/// ошибки живут дольше, чем породивший их `&self`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipStatusKind {
    /// Discriminator of [`OwnershipStatus::PendingVerification`].
    ///
    /// Дискриминатор [`OwnershipStatus::PendingVerification`].
    PendingVerification,
    /// Discriminator of [`OwnershipStatus::Active`].
    ///
    /// Дискриминатор [`OwnershipStatus::Active`].
    Active,
    /// Discriminator of [`OwnershipStatus::Ended`].
    ///
    /// Дискриминатор [`OwnershipStatus::Ended`].
    Ended,
}

impl OwnershipStatus {
    /// Returns `true` if the status is open, i.e. not yet terminated.
    ///
    /// This predicate carries the core business rule of the context: an open
    /// record *occupies* the vehicle and blocks a second ownership from being
    /// started. Crucially `PendingVerification` counts as open — an unverified
    /// claim still reserves the vehicle, otherwise two customers could each
    /// start a pending claim on the same car and both would be accepted.
    ///
    /// Возвращает `true`, если статус открытый, то есть ещё не завершённый.
    ///
    /// Этот предикат несёт основное бизнес-правило контекста: открытая запись
    /// *занимает* автомобиль и блокирует создание второго владения. Существенно,
    /// что `PendingVerification` считается открытым — неподтверждённое
    /// притязание всё равно резервирует автомобиль, иначе два клиента могли бы
    /// создать по ожидающей записи на одну машину, и обе были бы приняты.
    pub fn is_open(&self) -> bool {
        match self {
            Self::PendingVerification | Self::Active => true,
            Self::Ended => false,
        }
    }

    /// Returns the copyable discriminator of this status.
    ///
    /// Возвращает копируемый дискриминатор данного статуса.
    pub fn kind(&self) -> OwnershipStatusKind {
        match self {
            Self::PendingVerification => OwnershipStatusKind::PendingVerification,
            Self::Active => OwnershipStatusKind::Active,
            Self::Ended => OwnershipStatusKind::Ended,
        }
    }
}

impl std::fmt::Display for OwnershipStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingVerification => write!(f, "PendingVerification"),
            Self::Active => write!(f, "Active"),
            Self::Ended => write!(f, "Ended"),
        }
    }
}

/// Kind of vehicle ownership.
///
/// Classifies the relationship for reporting and for future rules that differ
/// by ownership kind — a leasing arrangement and a private owner may warrant
/// different verification requirements. `Unknown` exists for records imported
/// from external systems where the kind was not supplied; it is a legitimate
/// value, not an error placeholder.
///
/// Тип владения автомобилем.
///
/// Классифицирует отношение для отчётности и для будущих правил, различающихся
/// по типу владения: лизинг и частный владелец могут требовать разных условий
/// подтверждения. `Unknown` предусмотрен для записей, импортированных из
/// внешних систем, где тип не был указан; это допустимое значение, а не
/// заглушка ошибки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipType {
    /// Private individual owner.
    ///
    /// Частный владелец — физическое лицо.
    Private,
    /// Owned by a company.
    ///
    /// Принадлежит компании.
    Company,
    /// Held under a leasing agreement.
    ///
    /// Находится в лизинге.
    Leasing,
    /// Part of a managed vehicle fleet.
    ///
    /// Входит в управляемый автопарк.
    Fleet,
    /// Kind not supplied, typically for imported records.
    ///
    /// Тип не указан, обычно для импортированных записей.
    Unknown,
}

/// Ownership period: a start date and an optional end date.
///
/// A value object — created when the ownership starts and closed when it ends.
/// Invariant: `ended_at >= started_at`.
///
/// Период владения: дата начала и опциональная дата завершения.
///
/// Объект-значение: создаётся при начале владения и закрывается при его
/// завершении. Инвариант: `ended_at >= started_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipPeriod {
    /// Moment the ownership started.
    ///
    /// Момент начала владения.
    pub started_at: DateTime<Utc>,
    /// Moment the ownership ended; `None` while the period is still open.
    ///
    /// Момент завершения владения; `None`, пока период открыт.
    pub ended_at: Option<DateTime<Utc>>,
}

impl OwnershipPeriod {
    /// Opens a new period starting at the given moment.
    ///
    /// Открывает новый период, начинающийся в указанный момент.
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            started_at,
            ended_at: None,
        }
    }

    /// Returns `true` if the period is still open.
    ///
    /// Возвращает `true`, если период ещё открыт.
    pub fn is_open(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Closes the period. Returns `None` if `now < started_at`.
    ///
    /// Takes `self` by value and returns a new instance rather than mutating in
    /// place: as a value object a period is immutable, and returning `Option`
    /// means an invalid closing attempt yields no value at all instead of
    /// leaving a half-updated period behind. The caller decides how to report
    /// the failure — [`super::error::OwnershipError::PeriodEndBeforeStart`] in
    /// the aggregate's case.
    ///
    /// Закрывает период владения. Возвращает `None`, если `now < started_at`.
    ///
    /// Принимает `self` по значению и возвращает новый экземпляр, а не изменяет
    /// текущий: как объект-значение период неизменяем, а возврат `Option`
    /// означает, что неверная попытка закрытия не даёт значения вовсе, вместо
    /// того чтобы оставить период наполовину обновлённым. Способ сообщения об
    /// ошибке выбирает вызывающая сторона — в случае агрегата это
    /// [`super::error::OwnershipError::PeriodEndBeforeStart`].
    pub fn close(self, now: DateTime<Utc>) -> Option<Self> {
        if now < self.started_at {
            return None;
        }
        Some(Self {
            ended_at: Some(now),
            ..self
        })
    }
}

#[cfg(test)]
mod tests {
    use super::OwnershipStatus;

    /// Guards the rule that a pending claim already occupies the vehicle.
    /// If `PendingVerification` were ever reclassified as not-open, two
    /// customers could start concurrent claims on the same car.
    ///
    /// Защищает правило, по которому ожидающее притязание уже занимает
    /// автомобиль. Если бы `PendingVerification` перестал считаться открытым,
    /// два клиента могли бы создать конкурирующие притязания на одну машину.
    #[test]
    fn open_statuses_are_classified_correctly() {
        assert!(OwnershipStatus::PendingVerification.is_open());
        assert!(OwnershipStatus::Active.is_open());
        assert!(!OwnershipStatus::Ended.is_open());
    }
}
