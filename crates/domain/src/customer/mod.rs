//! Customer bounded context.
//!
//! Models the customer lifecycle: a customer is created as a `Draft` and is
//! promoted to `Active` once an activation permit proves they are eligible.
//!
//! The aggregate is deliberately narrow. Contact details, profile data,
//! preferences and consent each live in their own aggregate
//! (`CustomerContactBook`, `CustomerProfile`, …) so that editing a phone number
//! does not contend with an activation, and so the lifecycle invariants stay
//! small enough to reason about.
//!
//! # Layout
//!
//! - [`aggregate`] — the [`Customer`] aggregate root and its commands.
//! - [`state`] — lifecycle status and its transitions.
//! - [`event`] — domain events emitted by the aggregate.
//! - [`error`] — business-rule violations.
//! - [`permit`] — the [`ActivationPermit`] capability object.
//!
//! Ограниченный контекст «Клиент».
//!
//! Моделирует жизненный цикл клиента: клиент создаётся в статусе `Draft` и
//! переводится в `Active`, когда разрешение на активацию подтверждает его
//! пригодность.
//!
//! Агрегат намеренно узкий. Контактные данные, профиль, предпочтения и согласия
//! вынесены в собственные агрегаты (`CustomerContactBook`, `CustomerProfile`,
//! …), чтобы редактирование номера телефона не конкурировало с активацией, а
//! инварианты жизненного цикла оставались достаточно компактными для анализа.
//!
//! # Состав
//!
//! - [`aggregate`] — корень агрегата [`Customer`] и его команды.
//! - [`state`] — статус жизненного цикла и его переходы.
//! - [`event`] — доменные события, порождаемые агрегатом.
//! - [`error`] — нарушения бизнес-правил.
//! - [`permit`] — capability-объект [`ActivationPermit`].

pub mod aggregate;
pub mod error;
pub mod event;
pub mod permit;
pub mod state;

pub use aggregate::Customer;
pub use error::CustomerError;
pub use event::CustomerEvent;
pub use permit::ActivationPermit;
pub use state::CustomerStatus;
