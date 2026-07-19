//! Integration tests for the in-memory repository adapters.
//!
//! These exercise the adapters against the real aggregates rather than mocks,
//! so they verify the two things a unit test of either side alone cannot: that
//! the adapter's optimistic-locking arithmetic matches the versions aggregates
//! actually produce, and that the "one open ownership per vehicle" rule holds
//! when a handler and a real repository are wired together.
//!
//! Интеграционные тесты адаптеров репозиториев в памяти.
//!
//! Они проверяют адаптеры на настоящих агрегатах, а не на моках, поэтому
//! подтверждают то, чего не может подтвердить юнит-тест любой из сторон по
//! отдельности: что арифметика оптимистичной блокировки в адаптере
//! соответствует версиям, которые действительно порождают агрегаты, и что
//! правило «одно открытое владение на автомобиль» соблюдается при совместной
//! работе обработчика и настоящего репозитория.

mod customer_repository;
mod vehicle_ownership_repository;
mod vehicle_repository;
