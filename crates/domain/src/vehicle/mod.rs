//! Vehicle bounded context.
//!
//! Models the identity and lifecycle of a single vehicle: a vehicle is created
//! as a `Draft` and becomes `Active` once it carries a trustworthy identifier.
//!
//! Note the aggregate boundary: a vehicle knows nothing about who owns it.
//! Ownership is a separate aggregate root ([`crate::vehicle_ownership`])
//! because a vehicle outlives any individual ownership record and the two
//! change for entirely different reasons.
//!
//! # Layout
//!
//! - [`aggregate`] — the [`Vehicle`] aggregate root and its commands.
//! - [`state`] — lifecycle status and its transitions.
//! - [`event`] — domain events emitted by the aggregate.
//! - [`error`] — business-rule violations.
//! - [`permit`] — the [`VehicleActivationPermit`] capability object.
//!
//! Ограниченный контекст «Автомобиль».
//!
//! Моделирует identity и жизненный цикл конкретного автомобиля: автомобиль
//! создаётся в статусе `Draft` и становится `Active`, когда обладает надёжным
//! идентификатором.
//!
//! Обратите внимание на границу агрегата: автомобиль ничего не знает о своём
//! владельце. Владение — отдельный корень агрегата
//! ([`crate::vehicle_ownership`]), поскольку автомобиль живёт дольше любой
//! отдельной записи о владении, и изменяются они по совершенно разным причинам.
//!
//! # Состав
//!
//! - [`aggregate`] — корень агрегата [`Vehicle`] и его команды.
//! - [`state`] — статус жизненного цикла и его переходы.
//! - [`event`] — доменные события, порождаемые агрегатом.
//! - [`error`] — нарушения бизнес-правил.
//! - [`permit`] — capability-объект [`VehicleActivationPermit`].

pub mod aggregate;
pub mod error;
pub mod event;
pub mod permit;
pub mod state;

pub use aggregate::Vehicle;
pub use error::VehicleError;
pub use event::VehicleEvent;
pub use permit::VehicleActivationPermit;
pub use state::VehicleStatus;
