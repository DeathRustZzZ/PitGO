//! Unit tests for the application layer's use-case handlers.
//!
//! Each module defines a mock repository implementing the relevant port. The
//! mocks are what make these tests possible without a database at all — the
//! concrete payoff of declaring ports as traits in this layer rather than
//! depending on an adapter.
//!
//! Being able to inject failures is the other reason: a mock can be told to
//! return a `RepositoryError` on demand, so the tests can verify that handlers
//! propagate storage failures instead of swallowing them — a path that is
//! awkward to trigger against a real store.
//!
//! Юнит-тесты обработчиков сценариев использования слоя приложения.
//!
//! Каждый модуль определяет мок-репозиторий, реализующий соответствующий порт.
//! Именно моки позволяют выполнять эти тесты вообще без базы данных — это
//! конкретная выгода от объявления портов в виде трейтов в этом слое вместо
//! зависимости от адаптера.
//!
//! Вторая причина — возможность внедрять сбои: моку можно предписать вернуть
//! `RepositoryError` по требованию, поэтому тесты могут убедиться, что
//! обработчики пробрасывают сбои хранилища, а не поглощают их, — путь, который
//! неудобно воспроизводить на настоящем хранилище.

mod customer;
mod ownership;
mod vehicle;
