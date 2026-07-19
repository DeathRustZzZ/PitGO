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
    /// A write is accepted only if the incoming aggregate is exactly one
    /// version ahead of the stored one. When nothing is stored, `expected` is
    /// `None` and the first insert succeeds unconditionally.
    ///
    /// A duplicate create therefore surfaces as a `VersionConflict`: it arrives
    /// at version 1 while the store already holds version 1 and expects 2, so
    /// no separate existence check is needed.
    ///
    /// Сохраняет автомобиль, обеспечивая контракт оптимистичной блокировки.
    ///
    /// Запись принимается, только если входящий агрегат ровно на одну версию
    /// впереди сохранённого. Если ничего не сохранено, `expected` равно `None`
    /// и первая вставка проходит безусловно.
    ///
    /// Поэтому повторное создание проявляется как `VersionConflict`: оно
    /// приходит с версией 1, тогда как в хранилище уже лежит версия 1 и
    /// ожидается 2 — отдельная проверка существования не нужна.
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

        let actual = vehicle.version();
        let expected = vehicles
            .get(&vehicle.id())
            .map(|stored| stored.version().next());
        if let Some(expected_version) = expected
            && expected_version != actual
        {
            return Err(RepositoryError::VersionConflict {
                expected: expected_version.value(),
                actual: actual.value(),
            });
        }
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
