//! Типизированные доменные события агрегата `Customer`.

use crate::ids::CustomerId;

/// Клиент создан и находится в статусе `Draft`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerCreatedV1 {
    pub customer_id: CustomerId,
}

/// Клиент переведён в статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerActivatedV1 {
    pub customer_id: CustomerId,
}

/// Перечень всех доменных событий агрегата `Customer`.
///
/// Каждый вариант содержит конкретный типизированный payload.
/// `EventEnvelope` создаётся на persistence-границе, а не внутри агрегата.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerEvent {
    Created(CustomerCreatedV1),
    Activated(CustomerActivatedV1),
}
