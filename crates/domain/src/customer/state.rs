//! Customer lifecycle status.
//!
//! Статус жизненного цикла клиента.

/// Current state of the `Customer` aggregate.
///
/// Permitted transitions are encoded by the aggregate's named command methods
/// plus exhaustive `match`, rather than by a separate transition-rules object.
/// The compiler then enforces completeness: adding a status below fails to
/// build until every command has decided what that status means for it, which
/// is a stronger guarantee than a runtime transition table.
///
/// Only the statuses of the current MVP slice are implemented. `Suspended`,
/// `Blocked` and `Deleted` are added by separate tasks.
///
/// Текущее состояние агрегата `Customer`.
///
/// Допустимые переходы кодируются именованными командными методами агрегата и
/// исчерпывающим `match`, а не отдельным объектом правил переходов. Тогда
/// полноту обеспечивает компилятор: добавление статуса ниже не соберётся, пока
/// каждая команда не определит, что этот статус для неё означает, — гарантия
/// более сильная, чем таблица переходов времени выполнения.
///
/// Реализованы только состояния текущего MVP-среза. `Suspended`, `Blocked` и
/// `Deleted` добавляются отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerStatus {
    /// Created but not yet activated; the initial status.
    ///
    /// Создан, но ещё не активирован; начальный статус.
    Draft,
    /// Activated and able to participate in business operations.
    ///
    /// Активирован и может участвовать в бизнес-операциях.
    Active,
}

/// Copyable discriminator of [`CustomerStatus`] for use in error messages.
///
/// Exists so that an error value can name the offending status without
/// borrowing from the aggregate: errors outlive the `&self` that produced them,
/// and a `Copy` discriminator sidesteps that lifetime problem entirely. It also
/// keeps the error type `Copy`-friendly even once statuses start carrying data.
///
/// Копируемый дискриминатор [`CustomerStatus`] для сообщений об ошибках.
///
/// Существует, чтобы значение ошибки могло назвать проблемный статус, не
/// заимствуя данные агрегата: ошибки живут дольше, чем породивший их `&self`, а
/// `Copy`-дискриминатор полностью снимает эту проблему времён жизни. Кроме
/// того, он сохраняет тип ошибки дружественным к `Copy` даже тогда, когда
/// статусы начнут нести данные.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerStatusKind {
    /// Discriminator of [`CustomerStatus::Draft`].
    ///
    /// Дискриминатор [`CustomerStatus::Draft`].
    Draft,
    /// Discriminator of [`CustomerStatus::Active`].
    ///
    /// Дискриминатор [`CustomerStatus::Active`].
    Active,
}

impl CustomerStatus {
    /// Returns the copyable discriminator of this status.
    ///
    /// Возвращает копируемый дискриминатор данного статуса.
    pub fn kind(&self) -> CustomerStatusKind {
        match self {
            Self::Draft => CustomerStatusKind::Draft,
            Self::Active => CustomerStatusKind::Active,
        }
    }
}

impl std::fmt::Display for CustomerStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Active => write!(f, "Active"),
        }
    }
}
