//! Use-case handlers for the vehicle context.
//!
//! Обработчики сценариев использования для контекста «Автомобиль».

use crate::error::ApplicationError;
use crate::vehicle::commands::CreateVehicleCommand;
use crate::vehicle::ports::VehicleRepository;
use chrono::Utc;
use domain::VehicleId;
use domain::vehicle::aggregate::Vehicle;
use std::sync::Arc;

/// Handler for the "create a vehicle" use case.
///
/// The repository is held as `Arc<dyn VehicleRepository>`: `dyn` keeps the
/// handler independent of which adapter is wired in, and `Arc` lets one adapter
/// instance be shared across many concurrent request tasks instead of being
/// duplicated per task.
///
/// Обработчик сценария «создать автомобиль».
///
/// Репозиторий хранится как `Arc<dyn VehicleRepository>`: `dyn` делает
/// обработчик независимым от подключённого адаптера, а `Arc` позволяет
/// разделять один экземпляр адаптера между множеством конкурентных
/// задач-запросов, а не дублировать его на каждую задачу.
pub struct CreateVehicleHandler {
    repository: Arc<dyn VehicleRepository>,
}

impl CreateVehicleHandler {
    /// Builds the handler around a repository adapter.
    ///
    /// Создаёт обработчик поверх адаптера репозитория.
    pub fn new(repository: Arc<dyn VehicleRepository>) -> Self {
        Self { repository }
    }

    /// Executes [`CreateVehicleCommand`].
    ///
    /// Constructs the aggregate — which starts in `Draft` and raises its own
    /// creation event — and persists it. Creating an already-existing vehicle
    /// is refused by the repository's optimistic-locking check rather than by
    /// an explicit existence query, which avoids a redundant read and closes
    /// the race a check-then-write would open.
    ///
    /// Выполняет [`CreateVehicleCommand`].
    ///
    /// Создаёт агрегат — он начинается в статусе `Draft` и сам порождает
    /// событие создания — и сохраняет его. Создание уже существующего
    /// автомобиля отклоняется проверкой оптимистичной блокировки в
    /// репозитории, а не отдельным запросом на существование: это избавляет от
    /// лишнего чтения и закрывает состояние гонки, которое возникло бы при
    /// схеме «проверить, затем записать».
    pub async fn handle(&self, cmd: CreateVehicleCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let vehicle = Vehicle::create(cmd.vehicle_id, now);
        self.repository.save(&vehicle).await?;
        Ok(())
    }
}

/// Handler for the "fetch a vehicle by id" use case.
///
/// Обработчик сценария «получить автомобиль по идентификатору».
pub struct GetVehicleHandler {
    repository: Arc<dyn VehicleRepository>,
}

impl GetVehicleHandler {
    /// Builds the handler around a repository adapter.
    ///
    /// Создаёт обработчик поверх адаптера репозитория.
    pub fn new(repository: Arc<dyn VehicleRepository>) -> Self {
        Self { repository }
    }

    /// Returns the vehicle, or `None` if no such vehicle exists.
    ///
    /// A read-only use case with no domain rules to apply, so the handler
    /// delegates straight to the port. Turning `None` into a `404` is the HTTP
    /// layer's decision: absence is not an application error.
    ///
    /// Возвращает автомобиль либо `None`, если такого автомобиля нет.
    ///
    /// Сценарий только для чтения, доменных правил здесь нет, поэтому
    /// обработчик напрямую делегирует порту. Преобразование `None` в `404` —
    /// решение HTTP-слоя: отсутствие не является ошибкой приложения.
    pub async fn handle(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, ApplicationError> {
        Ok(self.repository.find_by_id(vehicle_id).await?)
    }
}
