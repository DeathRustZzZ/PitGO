//! Repository port for the vehicle-ownership context.
//!
//! Порт репозитория для контекста владения автомобилем.

use crate::error::RepositoryError;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::{VehicleId, VehicleOwnershipId};

/// Persistence port for [`VehicleOwnership`] aggregates.
///
/// # Why this is a trait
///
/// The application layer must not depend on a database driver. Declaring the
/// port here — beside its consumer rather than beside its implementation —
/// inverts the dependency: `infrastructure` depends on `application`, never the
/// reverse. In practice this is what lets the handler tests run against an
/// in-memory double with no PostgreSQL anywhere in the build.
///
/// # Why the methods are async
///
/// A real adapter performs network I/O against a database. Making the port
/// async means an awaiting handler yields its Tokio worker thread instead of
/// blocking it, so a few threads can serve many in-flight requests. The
/// in-memory adapter completes immediately and never actually suspends, but it
/// must still satisfy this signature — the port is shaped by the demanding
/// implementation, not the convenient one.
///
/// `#[async_trait]` is required because native `async fn` in traits does not
/// yet support dynamic dispatch; the macro rewrites each method to return a
/// boxed future so that `dyn VehicleOwnershipRepository` remains usable.
///
/// # Thread-safety contract
///
/// `Send + Sync` is mandatory: implementors are shared as
/// `Arc<dyn VehicleOwnershipRepository>` across concurrently executing Tokio
/// tasks that may migrate between worker threads. Methods take `&self`, not
/// `&mut self`, so an implementation must handle its own interior mutability
/// and remain correct under concurrent calls.
///
/// Порт персистентности для агрегатов [`VehicleOwnership`].
///
/// # Почему это трейт
///
/// Слой приложения не должен зависеть от драйвера базы данных. Объявление порта
/// здесь — рядом с потребителем, а не рядом с реализацией — инвертирует
/// зависимость: `infrastructure` зависит от `application`, но не наоборот. На
/// практике именно это позволяет тестам обработчиков работать с реализацией в
/// памяти, без какого-либо PostgreSQL в сборке.
///
/// # Почему методы асинхронные
///
/// Настоящий адаптер выполняет сетевой ввод-вывод к базе данных.
/// Асинхронность порта означает, что ожидающий обработчик освобождает рабочий
/// поток Tokio, а не блокирует его, и несколько потоков могут обслуживать
/// множество одновременных запросов. Адаптер в памяти завершается сразу и
/// фактически никогда не приостанавливается, но обязан соответствовать этой
/// сигнатуре: форму порта задаёт требовательная реализация, а не удобная.
///
/// `#[async_trait]` необходим, поскольку встроенные `async fn` в трейтах пока
/// не поддерживают динамическую диспетчеризацию; макрос переписывает каждый
/// метод так, чтобы он возвращал упакованный future, и `dyn
/// VehicleOwnershipRepository` оставался пригодным к использованию.
///
/// # Контракт потокобезопасности
///
/// `Send + Sync` обязательны: реализации разделяются как
/// `Arc<dyn VehicleOwnershipRepository>` между конкурентно исполняемыми
/// задачами Tokio, которые могут мигрировать между рабочими потоками. Методы
/// принимают `&self`, а не `&mut self`, поэтому реализация обязана сама
/// обеспечить внутреннюю изменяемость и корректность при конкурентных вызовах.
#[async_trait::async_trait]
pub trait VehicleOwnershipRepository: Send + Sync {
    /// Checks whether the given vehicle currently has an open ownership record
    /// (`PendingVerification` or `Active`). An open record occupies the vehicle
    /// and blocks starting a new one.
    ///
    /// The definition of "open" must match
    /// [`domain::vehicle_ownership::OwnershipStatus::is_open`] exactly. An
    /// implementation that counted only `Active` records would let a second
    /// claim be started while an unverified one is still pending — the precise
    /// invariant hole this method exists to close.
    ///
    /// Проверяет, есть ли у указанного автомобиля открытая запись о владении
    /// (`PendingVerification` или `Active`). Открытая запись занимает
    /// автомобиль и блокирует создание новой.
    ///
    /// Определение «открытости» обязано в точности совпадать с
    /// [`domain::vehicle_ownership::OwnershipStatus::is_open`]. Реализация,
    /// учитывающая только записи `Active`, позволила бы создать второе
    /// притязание, пока неподтверждённое ещё ожидает проверки, — именно эту
    /// дыру в инварианте закрывает данный метод.
    async fn has_open_ownership(&self, vehicle_id: VehicleId) -> Result<bool, RepositoryError>;

    /// Persists the ownership aggregate.
    ///
    /// Implementations must enforce optimistic locking: if the stored version
    /// is not the one preceding the aggregate's current version, the write is
    /// rejected with [`RepositoryError::VersionConflict`] rather than
    /// overwriting a concurrent change.
    ///
    /// Сохраняет агрегат владения.
    ///
    /// Реализации обязаны обеспечивать оптимистичную блокировку: если
    /// сохранённая версия не предшествует текущей версии агрегата, запись
    /// отклоняется с [`RepositoryError::VersionConflict`], а не перезаписывает
    /// конкурентное изменение.
    async fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError>;

    /// Looks up an ownership aggregate by its identifier.
    ///
    /// A missing record is `Ok(None)`, not an error: absence is an ordinary
    /// outcome of a lookup, and reserving the error channel for genuine
    /// failures keeps the two distinguishable at the call site.
    ///
    /// Находит агрегат владения по его идентификатору.
    ///
    /// Отсутствие записи — это `Ok(None)`, а не ошибка: отсутствие является
    /// обычным исходом поиска, а резервирование канала ошибок под настоящие
    /// сбои позволяет различать эти случаи в месте вызова.
    async fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError>;
}
