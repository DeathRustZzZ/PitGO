//! Shared kernel of the PitGO platform.
//!
//! Contains cross-cutting primitives reused by every bounded context:
//! aggregate versioning, domain-event envelopes, audit context and type-safe
//! identifiers.
//!
//! This crate sits at the very bottom of the dependency graph and must never
//! depend on `domain`, `application` or `infrastructure`. Anything placed here
//! is, by definition, a concept that no single bounded context owns — keeping
//! that rule strict is what prevents the crate from decaying into a dumping
//! ground for unrelated helpers.
//!
//! Разделяемое ядро платформы PitGO.
//!
//! Содержит сквозные примитивы, используемые всеми ограниченными контекстами:
//! версионирование агрегатов, конверты доменных событий, аудит-контекст и
//! типобезопасные идентификаторы.
//!
//! Крейт находится в самом низу графа зависимостей и никогда не должен зависеть
//! от `domain`, `application` или `infrastructure`. Всё, что размещается здесь,
//! по определению является понятием, не принадлежащим ни одному отдельному
//! контексту — строгое соблюдение этого правила не даёт крейту превратиться в
//! свалку несвязанных утилит.

pub mod aggregate;
pub mod audit;
pub mod event;
pub mod ids;

pub use ids::{ActorId, CausationId, CorrelationId, EventId};
