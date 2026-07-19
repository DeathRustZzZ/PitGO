//! Minimal stub of `VehicleActivationPermit`.
//!
//! The full implementation verifies, through `VehicleActivationPolicy`, that
//! the vehicle carries a trustworthy identifier — a VIN, or a trusted external
//! reference together with a license plate. The current variant is enough to
//! test the aggregate without a policy in place.
//!
//! Минимальная заглушка `VehicleActivationPermit`.
//!
//! Полная реализация проверяет через `VehicleActivationPolicy` наличие
//! надёжного идентификатора — VIN либо доверенной внешней ссылки вместе с
//! государственным номером. Текущий вариант достаточен для тестирования
//! агрегата без policy.

use chrono::{DateTime, Utc};

use crate::ids::VehicleId;
use crate::vehicle::error::VehicleActivationError;

/// Permission to activate a vehicle, issued by `VehicleActivationPolicy`.
///
/// A capability object: holding one is the aggregate's evidence that the
/// identifier check has already been performed. The aggregate validates only
/// the local conditions — matching `vehicle_id` and expiry.
///
/// Unlike [`crate::customer::ActivationPermit`], this permit does not pin an
/// aggregate version. A vehicle's identifying facts do not change in ways that
/// would invalidate the eligibility decision, so a version binding would cause
/// spurious rejections without buying any additional safety.
///
/// Fields are private so a permit cannot be forged or edited after issue.
///
/// Разрешение на активацию автомобиля, выдаваемое `VehicleActivationPolicy`.
///
/// Capability-объект: обладание им служит для агрегата доказательством, что
/// проверка идентификатора уже выполнена. Агрегат проверяет только локальные
/// условия — совпадение `vehicle_id` и срок действия.
///
/// В отличие от [`crate::customer::ActivationPermit`], этот permit не
/// привязывается к версии агрегата. Идентифицирующие сведения об автомобиле не
/// меняются так, чтобы обесценить решение о пригодности, поэтому привязка к
/// версии приводила бы к ложным отказам, не давая дополнительной надёжности.
///
/// Поля приватные, чтобы permit нельзя было подделать или изменить после выдачи.
#[derive(Debug, Clone)]
pub struct VehicleActivationPermit {
    vehicle_id: VehicleId,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl VehicleActivationPermit {
    /// Issues a permit. Intended to be called only from
    /// `VehicleActivationPolicy` (or from tests).
    ///
    /// Выдаёт permit. Предназначен для вызова только из
    /// `VehicleActivationPolicy` (или из тестов).
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

    /// Local checks performed by the aggregate before it changes state.
    ///
    /// Performs no I/O and consults no other aggregate — that is the point of
    /// the permit pattern: the aggregate stays a pure synchronous unit while
    /// still refusing to act on expired or misdirected evidence.
    ///
    /// Локальные проверки, выполняемые агрегатом перед изменением состояния.
    ///
    /// Не выполняет ввод-вывод и не обращается к другим агрегатам — в этом и
    /// состоит смысл паттерна permit: агрегат остаётся чистой синхронной
    /// единицей и при этом отказывается действовать по просроченным или чужим
    /// данным.
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
