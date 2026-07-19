//! Repository port for the customer context.
//!
//! Порт репозитория для контекста «Клиент».

use crate::error::RepositoryError;
use domain::CustomerId;
use domain::customer::aggregate::Customer;

/// Persistence port for [`Customer`] aggregates.
///
/// Declared in the application layer, beside its consumer, so that
/// `infrastructure` depends on `application` and never the reverse — the
/// application owns the interface it needs, and adapters conform to it.
///
/// The methods are async because a real adapter performs network I/O: an
/// awaiting handler yields its Tokio worker thread rather than blocking it.
/// `#[async_trait]` is needed because native `async fn` in traits does not yet
/// support dynamic dispatch, and this trait must remain usable as
/// `dyn CustomerRepository`.
///
/// `Send + Sync` is mandatory: implementors are shared as
/// `Arc<dyn CustomerRepository>` across concurrent tasks that may migrate
/// between worker threads. Methods take `&self`, so implementations must handle
/// their own interior mutability and stay correct under concurrent calls.
///
/// Порт персистентности для агрегатов [`Customer`].
///
/// Объявлен в слое приложения, рядом с потребителем, чтобы `infrastructure`
/// зависел от `application`, но не наоборот: приложение владеет нужным ему
/// интерфейсом, а адаптеры ему соответствуют.
///
/// Методы асинхронные, поскольку настоящий адаптер выполняет сетевой
/// ввод-вывод: ожидающий обработчик освобождает рабочий поток Tokio, а не
/// блокирует его. `#[async_trait]` необходим, так как встроенные `async fn` в
/// трейтах пока не поддерживают динамическую диспетчеризацию, а этот трейт
/// должен оставаться пригодным для использования как `dyn CustomerRepository`.
///
/// `Send + Sync` обязательны: реализации разделяются как
/// `Arc<dyn CustomerRepository>` между конкурентными задачами, которые могут
/// мигрировать между рабочими потоками. Методы принимают `&self`, поэтому
/// реализации обязаны сами обеспечить внутреннюю изменяемость и корректность
/// при конкурентных вызовах.
#[async_trait::async_trait]
pub trait CustomerRepository: Send + Sync {
    /// Persists the customer aggregate.
    ///
    /// Implementations must enforce optimistic locking: if the stored version
    /// is not the one preceding the aggregate's current version, the write is
    /// rejected with [`RepositoryError::VersionConflict`] rather than
    /// overwriting a concurrent change.
    ///
    /// Сохраняет агрегат клиента.
    ///
    /// Реализации обязаны обеспечивать оптимистичную блокировку: если
    /// сохранённая версия не предшествует текущей версии агрегата, запись
    /// отклоняется с [`RepositoryError::VersionConflict`], а не перезаписывает
    /// конкурентное изменение.
    async fn save(&self, customer: &Customer) -> Result<(), RepositoryError>;

    /// Looks up a customer aggregate by its identifier.
    ///
    /// A missing customer is `Ok(None)`, not an error: absence is an ordinary
    /// outcome of a lookup, and the error channel stays reserved for genuine
    /// storage failures.
    ///
    /// Находит агрегат клиента по его идентификатору.
    ///
    /// Отсутствие клиента — это `Ok(None)`, а не ошибка: отсутствие является
    /// обычным исходом поиска, а канал ошибок остаётся зарезервированным под
    /// настоящие сбои хранилища.
    async fn find_by_id(
        &self,
        customer_id: CustomerId,
    ) -> Result<Option<Customer>, RepositoryError>;
}
