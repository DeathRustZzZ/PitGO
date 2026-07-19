//! PitGO infrastructure layer.
//!
//! Supplies the *adapters* that implement the repository ports declared in
//! `application`. This is the outermost ring of the hexagon: it is the only
//! layer allowed to know about concrete storage technology, and nothing depends
//! on it in turn — `main` wires it in at startup, and every other layer sees
//! only the trait.
//!
//! # Current state
//!
//! All repositories here are in-memory `HashMap` implementations. They exist so
//! that the HTTP layer and the use cases can be exercised end-to-end before the
//! PostgreSQL adapters are written, and they deliberately implement the same
//! optimistic-locking contract the real adapters will, so that swapping them
//! out does not change observable behavior.
//!
//! Their one structural difference from a database is that they cannot enforce
//! a partial unique index. The "at most one open ownership per vehicle" rule is
//! therefore only checked read-then-write here, which is racy under true
//! concurrency — see [`vehicle_ownership_repository`].
//!
//! Инфраструктурный слой PitGO.
//!
//! Поставляет *адаптеры*, реализующие порты репозиториев, объявленные в
//! `application`. Это внешнее кольцо гексагона: только этому слою разрешено
//! знать о конкретной технологии хранения, и от него, в свою очередь, ничто не
//! зависит — `main` подключает его при старте, а все прочие слои видят лишь
//! трейт.
//!
//! # Текущее состояние
//!
//! Все репозитории здесь — реализации в памяти на основе `HashMap`. Они
//! существуют, чтобы HTTP-слой и сценарии использования можно было проверить
//! сквозным образом до написания адаптеров PostgreSQL, и намеренно реализуют
//! тот же контракт оптимистичной блокировки, что и будущие настоящие адаптеры,
//! чтобы их замена не изменила наблюдаемое поведение.
//!
//! Единственное структурное отличие от базы данных в том, что они не могут
//! обеспечить частичный уникальный индекс. Поэтому правило «не более одного
//! открытого владения на автомобиль» проверяется здесь только по схеме
//! «прочитать, затем записать», что подвержено гонке при настоящей
//! конкурентности — см. [`vehicle_ownership_repository`].

pub mod customer_repository;
pub mod tests;
pub mod vehicle_ownership_repository;
pub mod vehicle_repository;
