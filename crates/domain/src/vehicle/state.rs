//! Статус жизненного цикла агрегата `Vehicle`.

/// Текущее состояние автомобиля в платформе.
///
/// Допустимые переходы кодируются именованными методами агрегата.
/// `Archived`, `Disputed`, `Deleted` реализуются отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleStatus {
    Draft,
    Active,
}

/// Дискриминатор для сообщений об ошибках без заимствования `VehicleStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleStatusKind {
    Draft,
    Active,
}

impl VehicleStatus {
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
