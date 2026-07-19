//! Minimal stub of `ActivationPermit`.
//!
//! The full implementation will additionally carry the `ContactBook` and
//! `ConsentLedger` versions, an `ActivationEligibilitySnapshot` and a
//! signature. The current variant is enough to demonstrate the DDD patterns and
//! to test the aggregate without a policy in place.
//!
//! Минимальная заглушка `ActivationPermit`.
//!
//! Полная реализация дополнительно включает версии `ContactBook` и
//! `ConsentLedger`, `ActivationEligibilitySnapshot` и подпись. Текущий вариант
//! достаточен для демонстрации DDD-паттернов и тестирования агрегата без policy.

use chrono::{DateTime, Utc};

use shared::aggregate::AggregateVersion;

use crate::customer::error::CustomerActivationError;
use crate::ids::CustomerId;

/// Permission to activate a customer, issued by `CustomerActivationPolicy`.
///
/// This is a capability object: possessing one is the aggregate's evidence that
/// the expensive, cross-aggregate eligibility checks have already been
/// performed. The aggregate validates only the local conditions — matching id,
/// matching version and expiry. The global checks (verified contact, recorded
/// consent) run in the policy before the permit is created and are re-asserted
/// by `UnitOfWork` at commit.
///
/// Fields are private so that a permit cannot be forged or edited after issue;
/// the only way to obtain one is through [`ActivationPermit::new`].
///
/// Разрешение на активацию клиента, выдаваемое `CustomerActivationPolicy`.
///
/// Это capability-объект: обладание им служит для агрегата доказательством, что
/// дорогие кросс-агрегатные проверки пригодности уже выполнены. Агрегат
/// проверяет только локальные условия — совпадение идентификатора, совпадение
/// версии и срок действия. Глобальные проверки (подтверждённый контакт,
/// зафиксированное согласие) выполняются в policy до создания permit и
/// повторно утверждаются `UnitOfWork` при commit.
///
/// Поля приватные, чтобы permit нельзя было подделать или изменить после
/// выдачи; единственный способ его получить — [`ActivationPermit::new`].
#[derive(Debug, Clone)]
pub struct ActivationPermit {
    customer_id: CustomerId,
    customer_version: AggregateVersion,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl ActivationPermit {
    /// Issues a permit. Intended to be called only from
    /// `CustomerActivationPolicy` (or from tests).
    ///
    /// `customer_version` pins the permit to one exact aggregate state: if the
    /// customer changes before the permit is redeemed, activation is refused.
    ///
    /// Выдаёт permit. Предназначен для вызова только из
    /// `CustomerActivationPolicy` (или из тестов).
    ///
    /// `customer_version` привязывает permit к одному конкретному состоянию
    /// агрегата: если клиент изменится до предъявления permit, активация будет
    /// отклонена.
    pub fn new(
        customer_id: CustomerId,
        customer_version: AggregateVersion,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            customer_id,
            customer_version,
            issued_at,
            expires_at,
        }
    }

    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    pub fn customer_version(&self) -> AggregateVersion {
        self.customer_version
    }

    pub fn issued_at(&self) -> DateTime<Utc> {
        self.issued_at
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Local checks performed by the aggregate before it changes state.
    ///
    /// Reads neither the `ContactBook` nor the `ConsentLedger` — that is the
    /// whole point of the permit pattern. The aggregate stays a pure,
    /// synchronous, dependency-free unit while still refusing to act on stale
    /// or misdirected evidence.
    ///
    /// Локальные проверки, выполняемые агрегатом перед изменением состояния.
    ///
    /// Не читает ни `ContactBook`, ни `ConsentLedger` — в этом и состоит смысл
    /// паттерна permit. Агрегат остаётся чистой синхронной единицей без
    /// зависимостей и при этом отказывается действовать по устаревшим или
    /// чужим данным.
    pub fn validate_local(
        &self,
        customer_id: CustomerId,
        current_version: AggregateVersion,
        now: DateTime<Utc>,
    ) -> Result<(), CustomerActivationError> {
        if now > self.expires_at {
            return Err(CustomerActivationError::PermitExpired);
        }
        if self.customer_id != customer_id {
            return Err(CustomerActivationError::PermitCustomerIdMismatch);
        }
        if self.customer_version != current_version {
            return Err(CustomerActivationError::PermitVersionMismatch);
        }
        Ok(())
    }
}
