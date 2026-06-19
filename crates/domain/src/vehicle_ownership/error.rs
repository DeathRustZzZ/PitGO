//! Доменные ошибки агрегата `VehicleOwnership`.

use thiserror::Error;

use crate::vehicle_ownership::state::OwnershipStatusKind;

/// Ошибки бизнес-правил агрегата `VehicleOwnership`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OwnershipError {
    /// Попытка создать владение при уже существующем активном.
    #[error("для этого автомобиля уже существует активная запись о владении")]
    ActiveOwnershipAlreadyExists,

    /// Команда невозможна в текущем статусе.
    #[error("статус {0} не допускает данную операцию")]
    StatusDoesNotAllow(OwnershipStatusKind),

    /// Дата завершения периода раньше даты начала.
    #[error("дата завершения периода владения не может быть раньше даты начала")]
    PeriodEndBeforeStart,
}
