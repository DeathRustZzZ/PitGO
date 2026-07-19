//! Domain errors of the `Vehicle` aggregate.
//!
//! Доменные ошибки агрегата `Vehicle`.

use thiserror::Error;

use crate::vehicle::state::VehicleStatusKind;

/// Errors raised while validating a `VehicleActivationPermit` inside the
/// `activate` command.
///
/// All variants signal a rejected activation attempt, never an infrastructure
/// failure — a caller should surface a business-level refusal rather than retry.
///
/// Ошибки при проверке `VehicleActivationPermit` внутри команды `activate`.
///
/// Все варианты сигнализируют об отклонённой попытке активации, а не об
/// инфраструктурном сбое — вызывающая сторона должна показать отказ на уровне
/// бизнес-логики, а не повторять попытку.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VehicleActivationError {
    /// The permit was issued for a different vehicle.
    ///
    /// Indicates a wiring bug in the caller rather than a business condition:
    /// a permit must never travel between aggregates.
    ///
    /// Permit выдан для другого автомобиля.
    ///
    /// Указывает на ошибку связывания у вызывающей стороны, а не на бизнес-
    /// условие: permit никогда не должен переходить между агрегатами.
    #[error("VehicleActivationPermit выдан для другого vehicle_id")]
    PermitVehicleIdMismatch,

    /// The permit's validity window has passed.
    ///
    /// A permit is short-lived on purpose: it certifies that a trustworthy
    /// identifier was present when it was issued, and that claim must not be
    /// trusted indefinitely. Handled by reissuing the permit through the policy.
    ///
    /// Срок действия permit истёк.
    ///
    /// Permit намеренно короткоживущий: он удостоверяет наличие надёжного
    /// идентификатора на момент выдачи, и этому утверждению нельзя доверять
    /// бесконечно. Обрабатывается повторной выдачей permit через policy.
    #[error("срок действия VehicleActivationPermit истёк")]
    PermitExpired,

    /// The current lifecycle status does not permit activation.
    ///
    /// An already-`Active` vehicle does *not* produce this error: that case is
    /// idempotent and returns `NoChange` instead.
    ///
    /// Текущий статус жизненного цикла не допускает активацию.
    ///
    /// Уже активный автомобиль *не* приводит к этой ошибке — такой случай
    /// идемпотентен и возвращает `NoChange`.
    #[error("статус {0} не допускает активацию")]
    StatusDoesNotAllow(VehicleStatusKind),
}

/// Root domain error type of the `Vehicle` aggregate.
///
/// Groups the per-command error enums behind one type so callers can handle
/// "a vehicle rule was violated" uniformly while still matching on the specific
/// cause when they need to.
///
/// Корневой тип доменных ошибок агрегата `Vehicle`.
///
/// Объединяет покомандные перечисления ошибок под одним типом, чтобы
/// вызывающая сторона могла единообразно обрабатывать «нарушено правило
/// автомобиля», сохраняя возможность сопоставления с конкретной причиной.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VehicleError {
    /// An activation rule was violated. See [`VehicleActivationError`].
    ///
    /// Нарушено правило активации. См. [`VehicleActivationError`].
    #[error(transparent)]
    Activation(#[from] VehicleActivationError),
}
