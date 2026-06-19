//! Доменные ошибки агрегата `Vehicle`.

use thiserror::Error;

use crate::vehicle::state::VehicleStatusKind;

/// Ошибки при проверке `VehicleActivationPermit` внутри команды `activate`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VehicleActivationError {
    #[error("VehicleActivationPermit выдан для другого vehicle_id")]
    PermitVehicleIdMismatch,

    #[error("срок действия VehicleActivationPermit истёк")]
    PermitExpired,

    #[error("статус {0} не допускает активацию")]
    StatusDoesNotAllow(VehicleStatusKind),
}

/// Корневой тип доменных ошибок агрегата `Vehicle`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VehicleError {
    #[error(transparent)]
    Activation(#[from] VehicleActivationError),
}
