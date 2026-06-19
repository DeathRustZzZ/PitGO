//! Минимальная заглушка `VehicleActivationPermit`.
//!
//! Полная реализация проверяет наличие надёжного идентификатора (VIN
//! или trusted external ref + license plate) через `VehicleActivationPolicy`.
//! Текущий вариант достаточен для тестирования агрегата без policy.

use chrono::{DateTime, Utc};

use crate::ids::VehicleId;
use crate::vehicle::error::VehicleActivationError;

/// Разрешение на активацию автомобиля, выдаваемое `VehicleActivationPolicy`.
///
/// Агрегат проверяет только локальные условия: совпадение vehicle_id и срок.
/// Проверка надёжного идентификатора выполняется policy до создания permit.
#[derive(Debug, Clone)]
pub struct VehicleActivationPermit {
    vehicle_id: VehicleId,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl VehicleActivationPermit {
    pub fn new(vehicle_id: VehicleId, issued_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> Self {
        Self {
            vehicle_id,
            issued_at,
            expires_at,
        }
    }

    pub fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    pub fn issued_at(&self) -> DateTime<Utc> {
        self.issued_at
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Локальные проверки, выполняемые агрегатом перед изменением состояния.
    pub fn validate_local(
        &self,
        vehicle_id: VehicleId,
        now: DateTime<Utc>,
    ) -> Result<(), VehicleActivationError> {
        if self.vehicle_id != vehicle_id {
            return Err(VehicleActivationError::PermitVehicleIdMismatch);
        }
        if now > self.expires_at {
            return Err(VehicleActivationError::PermitExpired);
        }
        Ok(())
    }
}
