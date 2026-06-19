//! Снимок состояния для проверки кросс-агрегатных инвариантов при создании владения.

use crate::ids::VehicleId;

/// Снимок текущего состояния владений для конкретного автомобиля.
///
/// Создаётся application service перед вызовом `VehicleOwnership::start`.
/// Позволяет агрегату проверить инвариант «не более одного активного
/// владения» без прямого чтения других агрегатов.
///
/// Является capability-объектом: агрегат доверяет данным снимка.
/// Application service несёт ответственность за его актуальность.
#[derive(Debug, Clone)]
pub struct OwnershipEligibilitySnapshot {
    vehicle_id: VehicleId,
    has_active_ownership: bool,
}

impl OwnershipEligibilitySnapshot {
    pub fn new(vehicle_id: VehicleId, has_active_ownership: bool) -> Self {
        Self {
            vehicle_id,
            has_active_ownership,
        }
    }

    pub fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    /// Возвращает `true`, если для данного автомобиля нет активного владения.
    pub fn no_active_ownership_exists(&self) -> bool {
        !self.has_active_ownership
    }
}
