//! Domain event primitives and the infrastructure envelope.
//!
//! Events exist in two shapes, and the split is intentional. Inside an
//! aggregate an event is a bare `PendingEvent`: the domain knows *what*
//! happened but not what position the event will occupy in the stream. Only the
//! persistence boundary can assign an event id and a final aggregate version,
//! so it wraps the payload into an `EventEnvelope` at write time.
//!
//! Примитивы доменных событий и инфраструктурного конверта.
//!
//! События существуют в двух формах, и это разделение намеренно. Внутри
//! агрегата событие — это «голый» `PendingEvent`: домен знает, *что*
//! произошло, но не знает, какую позицию событие займёт в потоке. Присвоить
//! идентификатор события и итоговую версию агрегата может только граница
//! персистентности, поэтому она оборачивает payload в `EventEnvelope` в момент
//! записи.

use chrono::{DateTime, Utc};

use crate::aggregate::AggregateVersion;
use crate::ids::{ActorId, CausationId, CorrelationId, EventId};

/// Domain event before it is enriched with infrastructure metadata.
///
/// Created inside the aggregate. Carries no `event_id` and no
/// `aggregate_version` — both are assigned when the event is written to the
/// store. Keeping them out here is what allows the aggregate to stay free of
/// any persistence knowledge.
///
/// Доменное событие до обогащения инфраструктурными метаданными.
///
/// Создаётся внутри агрегата. Не содержит `event_id` и `aggregate_version` —
/// оба присваиваются при записи события в хранилище. Именно их отсутствие
/// здесь позволяет агрегату оставаться свободным от знаний о персистентности.
#[derive(Debug, Clone)]
pub struct PendingEvent<E> {
    /// Event payload.
    ///
    /// Полезная нагрузка события.
    pub payload: E,
    /// Moment the event occurred, as observed by the domain.
    ///
    /// Момент возникновения события с точки зрения домена.
    pub occurred_at: DateTime<Utc>,
}

impl<E> PendingEvent<E> {
    /// Wraps a payload together with the time the event occurred.
    ///
    /// Оборачивает payload вместе с моментом возникновения события.
    pub fn new(payload: E, occurred_at: DateTime<Utc>) -> Self {
        Self {
            payload,
            occurred_at,
        }
    }
}

/// Stable event contract for infrastructure and audit consumers.
///
/// Created at the application/infrastructure boundary when an aggregate is
/// persisted. `EventEnvelopeFactory` is deliberately not part of `shared`: it
/// needs the clock, the id generator and the ambient action context, all of
/// which belong to the application or infrastructure layer. Keeping the factory
/// out preserves this crate's rule of holding data, not wiring.
///
/// Note that `occurred_at` and `stored_at` are separate fields: an event can be
/// raised long before it is durably written, and consumers reasoning about
/// business time must not accidentally read infrastructure time.
///
/// Стабильный контракт события для инфраструктуры и аудита.
///
/// Создаётся на границе application/infrastructure при сохранении агрегата.
/// `EventEnvelopeFactory` намеренно не входит в `shared`: ей нужны часы,
/// генератор идентификаторов и окружающий контекст действия, а всё это
/// принадлежит слою application или infrastructure. Вынесение фабрики
/// сохраняет правило этого крейта — хранить данные, а не связывание.
///
/// Обратите внимание, что `occurred_at` и `stored_at` — разные поля: событие
/// может быть порождено задолго до долговременной записи, и потребители,
/// рассуждающие о бизнес-времени, не должны случайно прочитать время
/// инфраструктуры.
#[derive(Debug, Clone)]
pub struct EventEnvelope<E> {
    /// Unique identifier of this event, assigned at write time.
    ///
    /// Уникальный идентификатор события, присваиваемый при записи.
    pub event_id: EventId,
    /// Name of the aggregate type that produced the event.
    ///
    /// Имя типа агрегата, породившего событие.
    pub aggregate_type: String,
    /// Identifier of the source aggregate, serialized as a string.
    ///
    /// Идентификатор агрегата-источника в строковом представлении.
    pub aggregate_id: String,
    /// Aggregate version this event produced.
    ///
    /// Версия агрегата, к которой привело данное событие.
    pub aggregate_version: AggregateVersion,
    /// Event type name used for routing and deserialization.
    ///
    /// Имя типа события, используемое для маршрутизации и десериализации.
    pub event_type: String,
    /// Typed event payload.
    ///
    /// Типизированная полезная нагрузка события.
    pub payload: E,
    /// Groups all events belonging to the same business process.
    ///
    /// Объединяет все события, принадлежащие одному бизнес-процессу.
    pub correlation_id: CorrelationId,
    /// Command or event that directly caused this one.
    ///
    /// Команда или событие, непосредственно вызвавшее данное.
    pub causation_id: CausationId,
    /// Actor on whose behalf the event was produced.
    ///
    /// Актор, от имени которого было порождено событие.
    pub actor_id: ActorId,
    /// Business time: when the event happened in the domain.
    ///
    /// Бизнес-время: когда событие произошло в домене.
    pub occurred_at: DateTime<Utc>,
    /// Infrastructure time: when the event was durably written.
    ///
    /// Время инфраструктуры: когда событие было долговременно записано.
    pub stored_at: DateTime<Utc>,
}
