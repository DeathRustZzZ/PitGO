//! HTTP route handlers, grouped by bounded context.
//!
//! Each module here is a thin transport adapter: it deserializes a request into
//! an application command, invokes the matching handler, and turns the result
//! into a status code and body. Request and response types are defined
//! per-module rather than reusing domain types, so that an internal refactor
//! cannot silently change the public API shape.
//!
//! Обработчики HTTP-маршрутов, сгруппированные по ограниченным контекстам.
//!
//! Каждый модуль здесь — тонкий транспортный адаптер: он десериализует запрос в
//! команду приложения, вызывает соответствующий обработчик и превращает
//! результат в код состояния и тело ответа. Типы запросов и ответов
//! определяются в каждом модуле отдельно, а не переиспользуют доменные типы,
//! чтобы внутренний рефакторинг не мог незаметно изменить форму публичного API.

pub mod customer;
pub mod vehicle;
pub mod vehicle_ownership;
