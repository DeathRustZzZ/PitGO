//! Примитивы доменных событий и инфраструктурного конверта.

use chrono::{DateTime, Utc};

use crate::aggregate::AggregateVersion;
use crate::ids::{ActorId, CausationId, CorrelationId, EventId};

/// Доменное событие до обогащения инфраструктурными метаданными.
///
/// Создаётся внутри агрегата. Не содержит event_id и aggregate_version —
/// они присваиваются при записи в хранилище.
#[derive(Debug, Clone)]
pub struct PendingEvent<E> {
    /// Полезная нагрузка события.
    pub payload: E,
    /// Момент возникновения события.
    pub occurred_at: DateTime<Utc>,
}

impl<E> PendingEvent<E> {
    pub fn new(payload: E, occurred_at: DateTime<Utc>) -> Self {
        Self {
            payload,
            occurred_at,
        }
    }
}

/// Стабильный контракт события для инфраструктуры и аудита.
///
/// Создаётся на границе application/infrastructure при сохранении агрегата.
/// `EventEnvelopeFactory` намеренно не входит в shared — она принадлежит
/// application- или infrastructure-слою.
#[derive(Debug, Clone)]
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub aggregate_version: AggregateVersion,
    pub event_type: String,
    pub payload: E,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
    pub actor_id: ActorId,
    pub occurred_at: DateTime<Utc>,
    pub stored_at: DateTime<Utc>,
}
