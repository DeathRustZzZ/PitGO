//! PitGO domain layer.
//!
//! Holds the aggregates, value objects, domain events and business rules of the
//! platform. This crate is the innermost ring of the hexagonal architecture: it
//! depends only on `shared` and has zero infrastructure dependencies — no SQL,
//! no HTTP, no async runtime. That constraint is deliberate and load-bearing:
//! business rules that cannot reach a database cannot be quietly coupled to one,
//! and they stay testable with plain synchronous unit tests.
//!
//! Repository *ports* are traits; they are declared in the application layer,
//! and infrastructure supplies the adapters. Dependencies therefore always point
//! inward: `infrastructure → application → domain`.
//!
//! # Bounded contexts
//!
//! - [`customer`] — the customer lifecycle aggregate.
//! - [`vehicle`] — vehicle identity and lifecycle.
//! - [`vehicle_ownership`] — operational ownership linking a customer to a vehicle.
//!
//! Доменный слой PitGO.
//!
//! Содержит агрегаты, объекты-значения, доменные события и бизнес-правила
//! платформы. Этот крейт — внутреннее кольцо гексагональной архитектуры: он
//! зависит только от `shared` и не имеет инфраструктурных зависимостей — ни
//! SQL, ни HTTP, ни асинхронного рантайма. Это ограничение намеренно и несёт
//! нагрузку: бизнес-правила, которые не могут дотянуться до базы данных, не
//! могут быть незаметно с ней связаны и остаются тестируемыми обычными
//! синхронными юнит-тестами.
//!
//! *Порты* репозиториев — это трейты; они объявляются в слое приложения, а
//! инфраструктура поставляет адаптеры. Поэтому зависимости всегда направлены
//! внутрь: `infrastructure → application → domain`.
//!
//! # Ограниченные контексты
//!
//! - [`customer`] — агрегат жизненного цикла клиента.
//! - [`vehicle`] — identity и жизненный цикл автомобиля.
//! - [`vehicle_ownership`] — операционное владение, связывающее клиента и автомобиль.

pub mod customer;
pub mod ids;
pub mod vehicle;
pub mod vehicle_ownership;

pub use ids::*;
