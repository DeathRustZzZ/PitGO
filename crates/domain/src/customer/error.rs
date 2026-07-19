//! Domain errors of the `Customer` aggregate.
//!
//! Доменные ошибки агрегата `Customer`.

use thiserror::Error;

use crate::customer::state::CustomerStatusKind;

/// Errors raised while validating an `ActivationPermit` or the aggregate state
/// during the `activate` command.
///
/// All variants signal a rejected activation attempt. None of them indicate an
/// infrastructure failure — a caller seeing one of these should surface a
/// business-level refusal rather than retry.
///
/// Ошибки, возникающие при проверке `ActivationPermit` или состояния агрегата
/// в ходе команды `activate`.
///
/// Все варианты сигнализируют об отклонённой попытке активации. Ни один из них
/// не означает инфраструктурный сбой — вызывающая сторона должна показать отказ
/// на уровне бизнес-логики, а не повторять попытку.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CustomerActivationError {
    /// The permit's validity window has passed.
    ///
    /// A permit is short-lived on purpose: it certifies eligibility that was
    /// true when it was issued, and that claim must not be trusted
    /// indefinitely. Handled by reissuing the permit through the policy.
    ///
    /// Срок действия permit истёк.
    ///
    /// Permit намеренно короткоживущий: он удостоверяет пригодность, истинную
    /// на момент выдачи, и этому утверждению нельзя доверять бесконечно.
    /// Обрабатывается повторной выдачей permit через policy.
    #[error("срок действия ActivationPermit истёк")]
    PermitExpired,

    /// The permit was issued for a different customer.
    ///
    /// Indicates a wiring bug in the caller rather than a business condition:
    /// a permit must never travel between aggregates.
    ///
    /// Permit выдан для другого клиента.
    ///
    /// Указывает на ошибку связывания у вызывающей стороны, а не на бизнес-
    /// условие: permit никогда не должен переходить между агрегатами.
    #[error("customer_id в ActivationPermit не совпадает с id агрегата")]
    PermitCustomerIdMismatch,

    /// The customer changed after the permit was issued.
    ///
    /// The permit certifies eligibility for one exact aggregate version; any
    /// subsequent change may have invalidated the evidence behind it, so the
    /// activation is refused and the policy must re-evaluate.
    ///
    /// Клиент изменился после выдачи permit.
    ///
    /// Permit удостоверяет пригодность для одной конкретной версии агрегата;
    /// любое последующее изменение могло обесценить лежащие в его основе
    /// данные, поэтому активация отклоняется, а policy должна выполнить
    /// проверку заново.
    #[error("версия агрегата в ActivationPermit не совпадает с текущей версией")]
    PermitVersionMismatch,

    /// The current lifecycle status does not permit activation.
    ///
    /// Note that an already-`Active` customer does *not* produce this error:
    /// that case is idempotent and returns `NoChange` instead.
    ///
    /// Текущий статус жизненного цикла не допускает активацию.
    ///
    /// Обратите внимание: уже активный клиент *не* приводит к этой ошибке —
    /// такой случай идемпотентен и возвращает `NoChange`.
    #[error("статус {0} не допускает активацию")]
    StatusDoesNotAllow(CustomerStatusKind),
}

/// Root domain error type of the `Customer` aggregate.
///
/// Groups the per-command error enums behind one type so that callers can
/// handle "a customer rule was violated" uniformly, while still being able to
/// match on the specific cause when they care.
///
/// Корневой тип доменных ошибок агрегата `Customer`.
///
/// Объединяет покомандные перечисления ошибок под одним типом, чтобы
/// вызывающая сторона могла единообразно обрабатывать «нарушено правило
/// клиента», сохраняя при этом возможность сопоставления с конкретной причиной.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CustomerError {
    /// An activation rule was violated. See [`CustomerActivationError`].
    ///
    /// Нарушено правило активации. См. [`CustomerActivationError`].
    #[error(transparent)]
    Activation(#[from] CustomerActivationError),
}
