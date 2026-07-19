//! In-memory adapter for the [`VehicleOwnershipRepository`] port.
//!
//! Адаптер в памяти для порта [`VehicleOwnershipRepository`].

use std::collections::HashMap;
use std::sync::Mutex;

use application::error::RepositoryError;
use application::ownership::ports::VehicleOwnershipRepository;
use domain::VehicleOwnershipId;
use domain::vehicle_ownership::VehicleOwnership;

/// In-memory implementation of [`VehicleOwnershipRepository`].
///
/// Intended for tests and for local runs before the PostgreSQL adapter exists.
/// State is lost on restart.
///
/// # Concurrency
///
/// `std::sync::Mutex` is used rather than `tokio::sync::Mutex` because no lock
/// guard is ever held across an `.await`: every method locks, does purely
/// synchronous work, and drops the guard before returning. Adding an `.await`
/// inside a locked section would invalidate that reasoning.
///
/// # Known limitation
///
/// Ownerships are keyed by `VehicleOwnershipId`, so the "at most one open
/// ownership per vehicle" rule cannot be expressed as a key constraint here the
/// way a partial unique index expresses it in PostgreSQL. The rule is instead
/// checked by [`Self::has_open_ownership`] before the write, which leaves a
/// read-then-write window: two concurrent starts can both observe a free
/// vehicle and both succeed. This is acceptable for a development stand-in and
/// is precisely the gap the database index is meant to close.
///
/// Реализация [`VehicleOwnershipRepository`] в памяти.
///
/// Предназначена для тестов и локальных запусков до появления адаптера
/// PostgreSQL. Состояние теряется при перезапуске.
///
/// # Конкурентность
///
/// Используется `std::sync::Mutex`, а не `tokio::sync::Mutex`, поскольку
/// охранник блокировки никогда не удерживается через `.await`: каждый метод
/// захватывает блокировку, выполняет исключительно синхронную работу и
/// освобождает её до возврата. Добавление `.await` внутрь заблокированного
/// участка сделало бы это рассуждение неверным.
///
/// # Известное ограничение
///
/// Владения индексируются по `VehicleOwnershipId`, поэтому правило «не более
/// одного открытого владения на автомобиль» здесь нельзя выразить ограничением
/// ключа так, как его выражает частичный уникальный индекс в PostgreSQL. Вместо
/// этого правило проверяется методом [`Self::has_open_ownership`] перед
/// записью, что оставляет окно между чтением и записью: два конкурентных
/// создания могут увидеть свободный автомобиль и оба завершиться успешно. Для
/// реализации периода разработки это приемлемо и представляет собой ровно тот
/// разрыв, который призван закрыть индекс базы данных.
pub struct InMemoryVehicleOwnershipRepository {
    vehicle_ownership: Mutex<HashMap<VehicleOwnershipId, VehicleOwnership>>,
}

impl InMemoryVehicleOwnershipRepository {
    /// Creates an empty repository.
    ///
    /// Создаёт пустой репозиторий.
    pub fn new() -> Self {
        Self {
            vehicle_ownership: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VehicleOwnershipRepository for InMemoryVehicleOwnershipRepository {
    /// Saves an ownership record, enforcing the optimistic-locking contract.
    ///
    /// A first insert always succeeds. A duplicate `start` for the same
    /// `ownership_id` (version 1 while an entry already exists) returns
    /// `AlreadyExists`. A stale update (the stored version's successor does
    /// not match the incoming version) returns `VersionConflict`.
    ///
    /// Сохраняет запись о владении, обеспечивая контракт оптимистичной
    /// блокировки.
    ///
    /// Первая вставка всегда проходит. Повторный `start` для того же
    /// `ownership_id` (версия 1, когда запись уже существует) возвращает
    /// `AlreadyExists`. Устаревшее обновление (следующая ожидаемая версия не
    /// совпадает со входящей) возвращает `VersionConflict`.
    async fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError> {
        // A poisoned lock means another thread panicked mid-write; reported as
        // a storage failure rather than unwrapped, so one panicking request
        // cannot bring down the process.
        //
        // Отравленная блокировка означает, что другой поток запаниковал во
        // время записи; сообщается как сбой хранилища, а не разворачивается
        // через unwrap, чтобы один запаниковавший запрос не обрушил процесс.
        let mut ownerships = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let stored = ownerships.get(&ownership.id()).map(|o| o.version());
        crate::check_version(stored, ownership.version())?;
        ownerships.insert(ownership.id(), ownership.clone());
        Ok(())
    }

    /// Finds a vehicle ownership by its id.
    ///
    /// Returns `Ok(Some(_))` when found and `Ok(None)` when absent; absence is
    /// an ordinary result, not an error.
    ///
    /// Находит владение автомобилем по его идентификатору.
    ///
    /// Возвращает `Ok(Some(_))`, если запись найдена, и `Ok(None)`, если её
    /// нет; отсутствие — обычный результат, а не ошибка.
    async fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError> {
        let ownership = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(ownership.get(&ownership_id).cloned())
    }

    /// Checks whether an open ownership exists for the given vehicle.
    ///
    /// Both predicates in the filter are load-bearing. `status().is_open()`
    /// treats `PendingVerification` as occupying the vehicle, so an unverified
    /// claim still blocks a second one — narrowing this to `Active` alone would
    /// reopen the original defect. `vehicle_id()` scopes the check to one car,
    /// so an open record on a different vehicle does not leak into this answer.
    ///
    /// The scan is linear over all records, which is fine for a development
    /// stand-in; the PostgreSQL adapter answers the same question with an
    /// indexed query.
    ///
    /// Проверяет, существует ли открытое владение для указанного автомобиля.
    ///
    /// Оба условия фильтра существенны. `status().is_open()` считает, что
    /// `PendingVerification` занимает автомобиль, поэтому неподтверждённое
    /// притязание всё равно блокирует второе — сужение проверки только до
    /// `Active` вновь открыло бы исходный дефект. `vehicle_id()` ограничивает
    /// проверку одной машиной, чтобы открытая запись по другому автомобилю не
    /// повлияла на ответ.
    ///
    /// Перебор линеен по всем записям, что приемлемо для реализации периода
    /// разработки; адаптер PostgreSQL отвечает на тот же вопрос индексированным
    /// запросом.
    async fn has_open_ownership(
        &self,
        vehicle_id: domain::VehicleId,
    ) -> Result<bool, RepositoryError> {
        let ownerships = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let has_open = ownerships
            .values()
            .any(|ownership| ownership.vehicle_id() == vehicle_id && ownership.status().is_open());

        Ok(has_open)
    }
}

impl Default for InMemoryVehicleOwnershipRepository {
    fn default() -> Self {
        Self::new()
    }
}
