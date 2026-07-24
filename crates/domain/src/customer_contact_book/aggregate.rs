//! The `CustomerContactBook` aggregate — the customer's contact boundary.
//!
//! Агрегат `CustomerContactBook` — граница контактных данных клиента.

use chrono::{DateTime, Utc};
use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::{
    customer_contact_book::{
        error::CustomerContactBookError,
        event::{CustomerContactBookCreatedV1, CustomerContactBookEvent, PhoneAddedV1},
        value_objects::{phone_contact::PhoneContact, phone_number::PhoneNumber},
    },
    CustomerId,
};

/// Customer contact book aggregate root.
///
/// Owns customer-local contact state. The aggregate can exist without any
/// contacts; contacts are added later through named domain operations.
///
/// # Invariants
///
/// - Contact state changes only through aggregate methods.
/// - The version increases exactly once for every raised event.
/// - A newly created contact book contains no contacts.
///
/// Корень агрегата контактной книги клиента.
///
/// Владеет локальным состоянием контактов клиента. Агрегат может существовать
/// без контактов; они добавляются позже через именованные доменные операции.
///
/// # Инварианты
///
/// - Контактные данные изменяются только через методы агрегата.
/// - Версия увеличивается ровно один раз для каждого порождённого события.
/// - Новая контактная книга не содержит контактов.
#[derive(Debug)]
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone_contact: Option<PhoneContact>,
    // email_contact: EmailContact,
    // telegram_contact: TelegramContact,
    // primary_contact: PrimaryContact,
    aggregate_version: AggregateVersion,
    pending_events: Vec<PendingEvent<CustomerContactBookEvent>>,
}

impl CustomerContactBook {
    /// Creates an empty customer contact book.
    ///
    /// Creation raises a domain event, so the resulting aggregate version is
    /// `1`. The time is supplied explicitly to keep the domain deterministic.
    ///
    /// Создаёт пустую контактную книгу клиента.
    ///
    /// Создание порождает доменное событие, поэтому итоговая версия агрегата
    /// равна `1`. Время передаётся явно, чтобы домен оставался детерминированным.
    #[must_use]
    pub fn new(customer_id: CustomerId, now: DateTime<Utc>) -> Self {
        let mut contact_book = Self {
            customer_id,
            phone_contact: None,
            aggregate_version: AggregateVersion::INITIAL,
            pending_events: Vec::new(),
        };
        contact_book.raise(
            CustomerContactBookEvent::Created(CustomerContactBookCreatedV1 { customer_id }),
            now,
        );
        contact_book
    }

    /// Adds the first phone contact.
    ///
    /// Добавляет первый телефонный контакт.
    pub fn add_phone(
        &mut self,
        phone_number: PhoneNumber,
        now: DateTime<Utc>,
    ) -> Result<ChangeOutcome, CustomerContactBookError> {
        if self.phone_contact.is_some() {
            return Err(CustomerContactBookError::PhoneAlreadyExists);
        }

        let event_phone_number = phone_number.clone();

        self.phone_contact = Some(PhoneContact::new(phone_number));

        self.raise(
            CustomerContactBookEvent::PhoneAdded(PhoneAddedV1 {
                customer_id: self.customer_id,
                phone_number: event_phone_number,
            }),
            now,
        );

        Ok(ChangeOutcome::Changed)
    }

    /// Returns the buffered domain events and clears the buffer.
    ///
    /// Возвращает накопленные доменные события и очищает буфер.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<CustomerContactBookEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    /// Returns the customer identifier that owns this contact book.
    ///
    /// Возвращает идентификатор клиента, которому принадлежит контактная книга.
    #[must_use]
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    /// Returns the phone contact if one has been added.
    ///
    /// Возвращает телефонный контакт, если он добавлен.
    #[must_use]
    pub fn phone_contact(&self) -> Option<&PhoneContact> {
        self.phone_contact.as_ref()
    }

    /// Returns the current aggregate version for optimistic concurrency checks.
    ///
    /// Возвращает текущую версию агрегата для проверки оптимистичной блокировки.
    #[must_use]
    pub fn version(&self) -> AggregateVersion {
        self.aggregate_version
    }

    /// Buffers a domain event and advances the aggregate version.
    ///
    /// Добавляет доменное событие в буфер и увеличивает версию агрегата.
    fn raise(&mut self, event: CustomerContactBookEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.aggregate_version = self.aggregate_version.next();
    }
}

#[cfg(test)]
#[path = "tests/aggregate_tests.rs"]
mod aggregate_tests;
