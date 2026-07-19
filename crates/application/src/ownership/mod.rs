//! Vehicle ownership application module.
//!
//! Contains the use cases that start and manage ownership records. The
//! interesting responsibility here is supplying the domain with the
//! cross-aggregate fact it cannot fetch itself — whether the vehicle already
//! has an open ownership — and then letting the domain rule on it.
//!
//! Модуль приложения для владения автомобилем.
//!
//! Содержит сценарии использования, создающие записи о владении и управляющие
//! ими. Основная задача здесь — передать домену кросс-агрегатный факт, который
//! он не может получить сам (есть ли у автомобиля открытое владение), и затем
//! позволить домену вынести решение.

pub mod commands;
pub mod handlers;
pub mod ports;
