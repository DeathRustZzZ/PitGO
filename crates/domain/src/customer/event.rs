//! Typed domain events of the `Customer` aggregate.
//!
//! Event payload structs carry a `V1` suffix because events are a published
//! contract: once written to the store they are read back by consumers that may
//! be older than the writer. A breaking change therefore introduces
//! `CustomerCreatedV2` alongside `V1` rather than editing `V1` in place.
//!
//! Типизированные доменные события агрегата `Customer`.
//!
//! Структуры полезной нагрузки событий имеют суффикс `V1`, потому что события —
//! это опубликованный контракт: будучи записанными в хранилище, они читаются
//! потребителями, которые могут быть старше писателя. Поэтому ломающее
//! изменение вводит `CustomerCreatedV2` рядом с `V1`, а не правит `V1` на месте.

use crate::ids::CustomerId;

/// A customer was created and is in the `Draft` status.
///
/// Клиент создан и находится в статусе `Draft`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerCreatedV1 {
    /// Identifier of the created customer.
    ///
    /// Идентификатор созданного клиента.
    pub customer_id: CustomerId,
}

/// A customer was moved into the `Active` status.
///
/// Клиент переведён в статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerActivatedV1 {
    /// Identifier of the activated customer.
    ///
    /// Идентификатор активированного клиента.
    pub customer_id: CustomerId,
}

/// All domain events of the `Customer` aggregate.
///
/// Each variant holds a concrete, typed payload. The `EventEnvelope` — with its
/// event id, aggregate version and audit metadata — is built at the persistence
/// boundary, not inside the aggregate, so the domain never has to know how or
/// when its events will be stored.
///
/// Перечень всех доменных событий агрегата `Customer`.
///
/// Каждый вариант содержит конкретный типизированный payload. `EventEnvelope`
/// с идентификатором события, версией агрегата и метаданными аудита создаётся
/// на границе персистентности, а не внутри агрегата, поэтому домену не нужно
/// знать, как и когда его события будут сохранены.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerEvent {
    /// See [`CustomerCreatedV1`].
    ///
    /// См. [`CustomerCreatedV1`].
    Created(CustomerCreatedV1),
    /// See [`CustomerActivatedV1`].
    ///
    /// См. [`CustomerActivatedV1`].
    Activated(CustomerActivatedV1),
}
