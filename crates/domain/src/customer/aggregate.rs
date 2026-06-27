//! Агрегат `Customer` — lifecycle-граница клиента в PitGO.

use chrono::{DateTime, Utc};

use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::customer::error::CustomerError;
use crate::customer::event::{CustomerActivatedV1, CustomerCreatedV1, CustomerEvent};
use crate::customer::permit::ActivationPermit;
use crate::customer::state::CustomerStatus;
use crate::ids::CustomerId;

/// Агрегат жизненного цикла клиента.
///
/// Хранит только lifecycle-состояние. Контакты, профиль, preferences и consent
/// выделены в отдельные агрегаты (`CustomerContactBook`, `CustomerProfile`, …).
///
/// Поля приватные: состояние изменяется только именованными командными методами.
///
/// Clone временная необходимость для тестов и репозитория в памяти. В реальном приложении !!!
/// Убрать после реализиции Postgres репозитория.
#[derive(Debug, Clone)]
pub struct Customer {
    id: CustomerId,
    status: CustomerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingEvent<CustomerEvent>>,
}

impl Customer {
    /// Создаёт нового клиента в статусе `Draft`.
    ///
    /// Единственный публичный конструктор. Записывает `CustomerCreated` event
    /// и устанавливает версию агрегата в 1.
    pub fn create(id: CustomerId, now: DateTime<Utc>) -> Self {
        let mut customer = Self {
            id,
            status: CustomerStatus::Draft,
            created_at: now,
            updated_at: now,
            version: AggregateVersion::INITIAL,
            pending_events: Vec::new(),
        };
        customer.raise(
            CustomerEvent::Created(CustomerCreatedV1 { customer_id: id }),
            now,
        );
        customer
    }

    /// Переводит клиента из `Draft` в `Active`.
    ///
    /// Проверяет локальные условия permit (id, версия, срок). Глобальные
    /// проверки eligibility выполнены policy до выдачи permit и повторно
    /// утверждаются `UnitOfWork` при commit.
    ///
    /// Возвращает `NoChange` без изменения состояния, если клиент уже `Active`.
    /// Возвращает ошибку для любого другого статуса.
    pub fn activate(
        &mut self,
        permit: ActivationPermit,
        now: DateTime<Utc>,
    ) -> Result<ChangeOutcome, CustomerError> {
        permit.validate_local(self.id, self.version, now)?;

        match &self.status {
            CustomerStatus::Draft => {
                self.status = CustomerStatus::Active;
                self.updated_at = now;
                self.raise(
                    CustomerEvent::Activated(CustomerActivatedV1 {
                        customer_id: self.id,
                    }),
                    now,
                );
                Ok(ChangeOutcome::Changed)
            }
            CustomerStatus::Active => Ok(ChangeOutcome::NoChange),
        }
    }

    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    ///
    /// Вызывается persistence-слоем после успешного сохранения агрегата.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<CustomerEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Геттеры (для тестов и repository rehydration) ─────────────────────

    pub fn id(&self) -> CustomerId {
        self.id
    }

    pub fn status(&self) -> &CustomerStatus {
        &self.status
    }

    pub fn version(&self) -> AggregateVersion {
        self.version
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ── Приватные вспомогательные методы ──────────────────────────────────

    /// Записывает событие в буфер и увеличивает версию агрегата.
    ///
    /// Каждое принятое событие увеличивает версию ровно на 1.
    fn raise(&mut self, event: CustomerEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod aggregate_tests;
