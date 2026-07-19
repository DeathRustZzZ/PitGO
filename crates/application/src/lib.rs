//! PitGO application layer.
//!
//! Orchestrates use cases. A handler here loads aggregates, hands them the
//! facts they cannot fetch themselves, invokes a domain command and persists
//! the result. What it deliberately does *not* do is decide business questions:
//! every rule lives in `domain`, so that the rules stay testable without a
//! runtime and cannot be duplicated — and drift — between layers.
//!
//! # Ports and adapters
//!
//! This layer declares repository *ports* as traits (`*/ports.rs`).
//! `infrastructure` supplies the adapters that implement them. The port lives
//! here, next to its consumer, rather than in `infrastructure` next to its
//! implementation — that inversion is what lets the application depend on an
//! interface it owns instead of on a database driver, and what lets tests swap
//! in a mock without touching production code.
//!
//! # Structure
//!
//! Each bounded context follows the same three-file shape:
//!
//! - `commands.rs` — inert request data, no behavior.
//! - `ports.rs` — repository traits this context needs.
//! - `handlers.rs` — the use-case orchestration itself.
//!
//! Слой приложения PitGO.
//!
//! Оркеструет сценарии использования. Обработчик здесь загружает агрегаты,
//! передаёт им факты, которые они не могут получить сами, вызывает доменную
//! команду и сохраняет результат. Чего он намеренно *не* делает — так это не
//! решает бизнес-вопросы: все правила живут в `domain`, чтобы оставаться
//! тестируемыми без рантайма и не дублироваться — а значит, и не расходиться —
//! между слоями.
//!
//! # Порты и адаптеры
//!
//! Этот слой объявляет *порты* репозиториев в виде трейтов (`*/ports.rs`).
//! `infrastructure` поставляет реализующие их адаптеры. Порт находится здесь,
//! рядом с потребителем, а не в `infrastructure` рядом с реализацией: именно
//! эта инверсия позволяет приложению зависеть от собственного интерфейса, а не
//! от драйвера базы данных, и позволяет тестам подставить мок, не трогая
//! продуктивный код.
//!
//! # Структура
//!
//! Каждый ограниченный контекст имеет одинаковую структуру из трёх файлов:
//!
//! - `commands.rs` — инертные данные запроса без поведения.
//! - `ports.rs` — трейты репозиториев, нужные этому контексту.
//! - `handlers.rs` — собственно оркестрация сценария использования.

pub mod customer;
pub mod error;
pub mod ownership;
pub mod vehicle;

#[cfg(test)]
mod tests;
