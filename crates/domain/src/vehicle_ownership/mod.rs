//! Vehicle ownership bounded context.
//!
//! Models *operational* ownership: which customer the platform treats as
//! responsible for a given vehicle. This is not a legal record of title — it
//! answers "who may book service for this car", not "who owns it in law".
//!
//! The central business rule is that a vehicle may have at most one open
//! ownership record at a time. That rule spans aggregates, so it is enforced in
//! layers: [`OwnershipEligibilitySnapshot`] lets the aggregate refuse an
//! obviously invalid start, the application service builds that snapshot from
//! the repository, and a partial unique index in the database is the final
//! arbiter under concurrency.
//!
//! # Layout
//!
//! - [`aggregate`] — the [`VehicleOwnership`] aggregate root and its commands.
//! - [`state`] — ownership status, ownership type and the `OwnershipPeriod` value object.
//! - [`event`] — domain events emitted by the aggregate.
//! - [`error`] — business-rule violations.
//! - [`snapshot`] — the cross-aggregate eligibility snapshot.
//!
//! Ограниченный контекст «Владение автомобилем».
//!
//! Моделирует *операционное* владение: какого клиента платформа считает
//! ответственным за конкретный автомобиль. Это не юридическая запись о
//! собственности — она отвечает на вопрос «кто может записать машину на
//! обслуживание», а не «кому она принадлежит по закону».
//!
//! Центральное бизнес-правило: у автомобиля одновременно может быть не более
//! одной открытой записи о владении. Правило охватывает несколько агрегатов,
//! поэтому обеспечивается послойно: [`OwnershipEligibilitySnapshot`] позволяет
//! агрегату отклонить заведомо неверное создание, сервис приложения строит
//! этот снимок по данным репозитория, а частичный уникальный индекс в базе
//! данных выступает окончательным арбитром при конкурентном доступе.
//!
//! # Состав
//!
//! - [`aggregate`] — корень агрегата [`VehicleOwnership`] и его команды.
//! - [`state`] — статус владения, тип владения и объект-значение `OwnershipPeriod`.
//! - [`event`] — доменные события, порождаемые агрегатом.
//! - [`error`] — нарушения бизнес-правил.
//! - [`snapshot`] — кросс-агрегатный снимок пригодности.

pub mod aggregate;
pub mod error;
pub mod event;
pub mod snapshot;
pub mod state;

pub use aggregate::VehicleOwnership;
pub use error::OwnershipError;
pub use event::VehicleOwnershipEvent;
pub use snapshot::OwnershipEligibilitySnapshot;
pub use state::{OwnershipStatus, OwnershipType};
