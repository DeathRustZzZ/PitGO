//! In-memory adapter for the [`VehicleRepository`] port.
//!
//! Адаптер в памяти для порта [`VehicleRepository`].

use application::error::RepositoryError;
use application::vehicle::ports::VehicleRepository;
use domain::VehicleId;
use domain::vehicle::Vehicle;
use std::collections::HashMap;
use std::sync::Mutex;

/// In-memory implementation of [`VehicleRepository`].
///
/// Intended for tests and for local runs before the PostgreSQL adapter exists.
/// State is lost on restart.
///
/// `std::sync::Mutex` is used rather than `tokio::sync::Mutex` because no lock
/// guard is ever held across an `.await`: every method locks, does purely
/// synchronous work, and drops the guard before returning. Adding an `.await`
/// inside a locked section would invalidate that reasoning and risk stalling a
/// Tokio worker thread.
///
/// Реализация [`VehicleRepository`] в памяти.
///
/// Предназначена для тестов и локальных запусков до появления адаптера
/// PostgreSQL. Состояние теряется при перезапуске.
///
/// Используется `std::sync::Mutex`, а не `tokio::sync::Mutex`, поскольку
/// охранник блокировки никогда не удерживается через `.await`: каждый метод
/// захватывает блокировку, выполняет исключительно синхронную работу и
/// освобождает её до возврата. Добавление `.await` внутрь заблокированного
/// участка сделало бы это рассуждение неверным и создало бы риск остановки
/// рабочего потока Tokio.
pub struct InMemoryVehicleRepository {
    vehicle: Mutex<HashMap<VehicleId, Vehicle>>,
}

impl InMemoryVehicleRepository {
    /// Creates an empty repository.
    ///
    /// Создаёт пустой репозиторий.
    pub fn new() -> Self {
        Self {
            vehicle: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VehicleRepository for InMemoryVehicleRepository {
    /// Saves a vehicle, enforcing the optimistic-locking contract.
    ///
    /// A first insert always succeeds. A duplicate create (version 1 while
    /// an entry already exists) returns `AlreadyExists`. A stale update
    /// (the stored version's successor does not match the incoming version)
    /// returns `VersionConflict`.
    ///
    /// Сохраняет автомобиль, обеспечивая контракт оптимистичной блокировки.
    ///
    /// Первая вставка всегда проходит. Повторное создание (версия 1, когда
    /// запись уже существует) возвращает `AlreadyExists`. Устаревшее обновление
    /// (следующая ожидаемая версия не совпадает со входящей) возвращает
    /// `VersionConflict`.
    async fn save(&self, vehicle: &Vehicle) -> Result<(), application::error::RepositoryError> {
        // A poisoned lock means another thread panicked mid-write; reported as
        // a storage failure rather than unwrapped, so one panicking request
        // cannot bring down the process.
        //
        // Отравленная блокировка означает, что другой поток запаниковал во
        // время записи; сообщается как сбой хранилища, а не разворачивается
        // через unwrap, чтобы один запаниковавший запрос не обрушил процесс.
        let mut vehicles = self
            .vehicle
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let stored = vehicles.get(&vehicle.id()).map(|v| v.version());
        crate::check_version(stored, vehicle.version())?;
        vehicles.insert(vehicle.id(), vehicle.clone());

        Ok(())
    }

    /// Returns the stored vehicle, or `None` if absent.
    ///
    /// The aggregate is cloned so the caller cannot mutate the stored copy
    /// through a shared reference — the in-memory stand-in for a database
    /// returning a fresh row per query.
    ///
    /// Возвращает сохранённый автомобиль либо `None`, если его нет.
    ///
    /// Агрегат клонируется, чтобы вызывающая сторона не могла изменить
    /// сохранённую копию через общую ссылку — это замена в памяти для базы
    /// данных, возвращающей на каждый запрос свежую строку.
    async fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError> {
        let vehicles = self
            .vehicle
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(vehicles.get(&vehicle_id).cloned())
    }
}

impl Default for InMemoryVehicleRepository {
    fn default() -> Self {
        Self::new()
    }
}
