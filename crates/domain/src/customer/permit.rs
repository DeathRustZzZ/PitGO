//! Минимальная заглушка `ActivationPermit`.
//!
//! Полная реализация включает версии ContactBook и ConsentLedger,
//! `ActivationEligibilitySnapshot` и подпись. Текущий вариант достаточен
//! для доказательства DDD-паттернов и тестов агрегата.

use chrono::{DateTime, Utc};

use shared::aggregate::AggregateVersion;

use crate::customer::error::CustomerActivationError;
use crate::ids::CustomerId;

/// Разрешение на активацию клиента, выдаваемое `CustomerActivationPolicy`.
///
/// Агрегат проверяет только локальные условия: совпадение id, версии и срок.
/// Глобальные проверки (verified contact, consent) выполняются policy до
/// создания permit и повторно утверждаются `UnitOfWork` при commit.
#[derive(Debug, Clone)]
pub struct ActivationPermit {
    customer_id: CustomerId,
    customer_version: AggregateVersion,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl ActivationPermit {
    /// Создаётся только внутри `CustomerActivationPolicy` (или тестов).
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

    /// Локальные проверки, выполняемые агрегатом перед изменением состояния.
    ///
    /// Не читает ContactBook или ConsentLedger.
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
