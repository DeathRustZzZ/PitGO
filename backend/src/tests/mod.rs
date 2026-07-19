//! Test modules of the backend crate.
//!
//! Tests live here rather than beside the code they exercise so that a module
//! like `error.rs` stays readable as a contract definition. This mirrors the
//! layout of the `application` crate; the domain crate keeps its tests next to
//! each aggregate instead, because those tests are part of how the aggregate's
//! invariants are specified.
//!
//! A consequence of the separation worth knowing: modules here can only reach
//! items that are visible crate-wide. Reaching into a private field is not an
//! option, which pushes these tests towards the public contract — that is the
//! point, not an inconvenience.
//!
//! Тестовые модули крейта backend.
//!
//! Тесты размещены здесь, а не рядом с проверяемым кодом, чтобы модуль вроде
//! `error.rs` оставался читаемым как определение контракта. Это повторяет
//! структуру крейта `application`; крейт domain, напротив, держит тесты рядом с
//! каждым агрегатом, поскольку там они являются частью описания его инвариантов.
//!
//! Важное следствие такого разделения: модули отсюда видят только элементы,
//! доступные в пределах всего крейта. Обратиться к приватному полю невозможно,
//! и это подталкивает тесты к публичному контракту — в чём и состоит замысел, а
//! не неудобство.

mod error;
