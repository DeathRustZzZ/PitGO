//! Доменные ошибки агрегата `Customer`.

use thiserror::Error;

use crate::customer::state::CustomerStatusKind;

/// Ошибки, возникающие при проверке `ActivationPermit` или состояния агрегата
/// в ходе команды `activate`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CustomerActivationError {
    #[error("срок действия ActivationPermit истёк")]
    PermitExpired,

    #[error("customer_id в ActivationPermit не совпадает с id агрегата")]
    PermitCustomerIdMismatch,

    #[error("версия агрегата в ActivationPermit не совпадает с текущей версией")]
    PermitVersionMismatch,

    #[error("статус {0} не допускает активацию")]
    StatusDoesNotAllow(CustomerStatusKind),
}

/// Корневой тип доменных ошибок агрегата `Customer`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CustomerError {
    #[error(transparent)]
    Activation(#[from] CustomerActivationError),
}
