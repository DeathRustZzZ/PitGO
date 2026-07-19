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

/// Checks whether an aggregate being saved has a consistent version.
///
/// Returns `Ok(())` on a first insert (nothing stored yet) and on a valid
/// in-sequence update. Returns `AlreadyExists` when a freshly-created aggregate
/// (version 1) is saved while a stored entry already exists — that is a
/// duplicate create, not a stale update. Returns `VersionConflict` when an
/// existing entry's next expected version does not match the incoming one.
///
/// # Assumption
///
/// Every aggregate `create` raises exactly one event, so a freshly-created
/// aggregate always arrives at version 1. Adapters that call this helper must
/// honour that invariant: an adapter whose `create` raises more than one event
/// would misclassify a genuine stale update as a duplicate create whenever
/// the incoming version happens to be 1.
///
/// Проверяет, что агрегат, сохраняемый в репозитории, имеет согласованную версию.
///
/// Возвращает `Ok(())` при первой вставке (ещё ничего не сохранено) и при корректном обновлении по порядку.
/// Возвращает `AlreadyExists`, если агрегат, только что созданный (версия 1), сохраняется,
/// в то время как уже существует сохранённая запись — это дублирующее создание, а не устаревшее обновление.
/// Возвращает `VersionConflict`, если следующая ожидаемая версия существующей записи не совпадает с входящей
/// версией.
///
/// # Предположение
/// Каждое создание агрегата вызывает ровно одно событие, поэтому только что созданный агрегат всегда приходит
/// с версией 1. Адаптеры, вызывающие эту функцию, должны соблюдать это инвариант: адаптер, чей `create`
/// вызывает более одного события, неверно классифицировал бы подлинное устаревшее обновление как
/// дублирующее создание, если входящая версия случайно равна 1.
pub(crate) fn check_version(
    stored: Option<shared::aggregate::AggregateVersion>,
    incoming: shared::aggregate::AggregateVersion,
) -> Result<(), application::error::RepositoryError> {
    use application::error::RepositoryError;

    let Some(stored_version) = stored else {
        return Ok(());
    };
    if incoming.value() == 1 {
        return Err(RepositoryError::AlreadyExists);
    }
    let expected = stored_version.next();
    if expected != incoming {
        return Err(RepositoryError::VersionConflict {
            expected: expected.value(),
            actual: incoming.value(),
        });
    }
    Ok(())
}
