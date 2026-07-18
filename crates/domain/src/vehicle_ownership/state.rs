//! Статус и вспомогательные типы агрегата `VehicleOwnership`.

use chrono::{DateTime, Utc};

/// Текущее состояние записи о владении.
///
/// `PendingVerification → Active` через `verify`.
/// `Active → Ended` через `end` (терминальное).
/// `Disputed`, `Rejected` реализуются отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipStatus {
    PendingVerification,
    Active,
    Ended,
}

/// Дискриминатор для сообщений об ошибках.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipStatusKind {
    PendingVerification,
    Active,
    Ended,
}

impl OwnershipStatus {
    /// Возвращает `true` если статус открытый (не завершённый).
    pub fn is_open(self) -> bool {
        match self {
            Self::PendingVerification | Self::Active => true,
            Self::Ended => false,
        }
    }

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

/// Тип владения автомобилем.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnershipType {
    Private,
    Company,
    Leasing,
    Fleet,
    Unknown,
}

/// Период владения: дата начала и опциональная дата завершения.
///
/// Является Value Object — создаётся в момент `start` и закрывается
/// при `end`. Инвариант: `ended_at >= started_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipPeriod {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl OwnershipPeriod {
    pub fn new(started_at: DateTime<Utc>) -> Self {
        Self {
            started_at,
            ended_at: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Закрывает период владения. Возвращает `None`, если `now < started_at`.
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

    #[test]
    fn open_statuses_are_classified_correctly() {
        assert!(OwnershipStatus::PendingVerification.is_open());
        assert!(OwnershipStatus::Active.is_open());
        assert!(!OwnershipStatus::Ended.is_open());
    }
}
