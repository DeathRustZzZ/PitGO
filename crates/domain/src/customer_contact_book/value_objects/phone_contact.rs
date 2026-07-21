use chrono::{DateTime, Utc};
use shared::aggregate::ChangeOutcome;

use crate::customer_contact_book::value_objects::phone_number::PhoneNumber;

/// Контактный телефон клиента и статус его подтверждения.
///
/// A customer's phone contact together with its verification status.
#[derive(Debug)]
pub struct PhoneContact {
    number: PhoneNumber,

    verification: VerificationStatus,
}

/// Доверие к контактному номеру.
///
/// The trust state of a phone contact.
#[derive(Debug)]
pub enum VerificationStatus {
    Unverified,

    /// Номер был подтверждён в указанное время.
    ///
    /// The number was verified at the given time.
    Verified {
        verified_at: DateTime<Utc>,
    },
}

impl PhoneContact {
    /// Создаёт новый неподтверждённый контакт.
    ///
    /// Creates a new unverified phone contact.
    #[must_use]
    pub fn new(number: PhoneNumber) -> Self {
        Self {
            number,
            verification: VerificationStatus::Unverified,
        }
    }

    /// Подтверждает контакт, если он ещё не был подтверждён.
    ///
    /// Verifies the contact if it has not been verified already.
    ///
    /// Повторное подтверждение ничего не меняет: это делает операцию
    /// безопасной для повторной доставки команды.
    ///
    /// Repeating verification changes nothing, making the operation safe
    /// to retry after a duplicate command delivery.
    pub fn verify(&mut self, now: DateTime<Utc>) -> ChangeOutcome {
        match &self.verification {
            VerificationStatus::Unverified => {
                self.verification = VerificationStatus::Verified { verified_at: now };
                ChangeOutcome::Changed
            }
            VerificationStatus::Verified { .. } => ChangeOutcome::NoChange,
        }
    }

    /// Возвращает нормализованный номер телефона.
    ///
    /// Returns the normalized phone number.
    pub fn number(&self) -> &PhoneNumber {
        &self.number
    }

    /// Возвращает текущий статус подтверждения.
    ///
    /// Returns the current verification status.
    pub fn verification(&self) -> &VerificationStatus {
        &self.verification
    }
}
