//! Lifecycle status of the `Vehicle` aggregate.
//!
//! Статус жизненного цикла агрегата `Vehicle`.

/// Current state of a vehicle within the platform.
///
/// Permitted transitions are encoded by the aggregate's named command methods
/// plus exhaustive `match`, so adding a status below fails to compile until
/// every command has decided what that status means for it.
///
/// `Archived`, `Disputed` and `Deleted` are implemented by separate tasks.
///
/// Текущее состояние автомобиля в платформе.
///
/// Допустимые переходы кодируются именованными командными методами агрегата и
/// исчерпывающим `match`, поэтому добавление статуса ниже не соберётся, пока
/// каждая команда не определит, что этот статус для неё означает.
///
/// `Archived`, `Disputed` и `Deleted` реализуются отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleStatus {
    /// Registered but not yet activated; the initial status.
    ///
    /// Зарегистрирован, но ещё не активирован; начальный статус.
    Draft,
    /// Carries a trustworthy identifier and may be used in operations.
    ///
    /// Обладает надёжным идентификатором и может использоваться в операциях.
    Active,
}

/// Copyable discriminator of [`VehicleStatus`] for use in error messages.
///
/// Lets an error name the offending status without borrowing from the
/// aggregate — errors outlive the `&self` that produced them.
///
/// Копируемый дискриминатор [`VehicleStatus`] для сообщений об ошибках.
///
/// Позволяет ошибке назвать проблемный статус, не заимствуя данные агрегата:
/// ошибки живут дольше, чем породивший их `&self`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleStatusKind {
    /// Discriminator of [`VehicleStatus::Draft`].
    ///
    /// Дискриминатор [`VehicleStatus::Draft`].
    Draft,
    /// Discriminator of [`VehicleStatus::Active`].
    ///
    /// Дискриминатор [`VehicleStatus::Active`].
    Active,
}

impl VehicleStatus {
    /// Returns the copyable discriminator of this status.
    ///
    /// Возвращает копируемый дискриминатор данного статуса.
    pub fn kind(&self) -> VehicleStatusKind {
        match self {
            Self::Draft => VehicleStatusKind::Draft,
            Self::Active => VehicleStatusKind::Active,
        }
    }
}

impl std::fmt::Display for VehicleStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Active => write!(f, "Active"),
        }
    }
}
