//! Use-case handlers for the customer context.
//!
//! Обработчики сценариев использования для контекста «Клиент».

use crate::customer::commands::CreateCustomerCommand;
use crate::customer::ports::CustomerRepository;
use crate::error::ApplicationError;
use chrono::Utc;
use domain::CustomerId;
use domain::customer::aggregate::Customer;
use std::sync::Arc;

/// Handler for the "create a customer" use case.
///
/// The repository is held as `Arc<dyn CustomerRepository>`: `dyn` keeps the
/// handler independent of which adapter is wired in, and `Arc` lets one adapter
/// instance be shared across many concurrent request tasks instead of being
/// duplicated per task.
///
/// Обработчик сценария «создать клиента».
///
/// Репозиторий хранится как `Arc<dyn CustomerRepository>`: `dyn` делает
/// обработчик независимым от подключённого адаптера, а `Arc` позволяет
/// разделять один экземпляр адаптера между множеством конкурентных
/// задач-запросов, а не дублировать его на каждую задачу.
pub struct CreateCustomerHandler {
    repository: Arc<dyn CustomerRepository>,
}

impl CreateCustomerHandler {
    /// Builds the handler around a repository adapter.
    ///
    /// Создаёт обработчик поверх адаптера репозитория.
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    /// Executes [`CreateCustomerCommand`].
    ///
    /// Constructs the aggregate — which starts in `Draft` and raises its own
    /// creation event — and persists it. Creating an already-existing customer
    /// is refused by the repository's optimistic-locking check rather than by
    /// an explicit existence query, which avoids a redundant read and closes
    /// the race a check-then-write would open.
    ///
    /// `now` is read here and passed inward so the domain stays free of ambient
    /// clock access and remains deterministic under test.
    ///
    /// Выполняет [`CreateCustomerCommand`].
    ///
    /// Создаёт агрегат — он начинается в статусе `Draft` и сам порождает
    /// событие создания — и сохраняет его. Создание уже существующего клиента
    /// отклоняется проверкой оптимистичной блокировки в репозитории, а не
    /// отдельным запросом на существование: это избавляет от лишнего чтения и
    /// закрывает состояние гонки, которое возникло бы при схеме
    /// «проверить, затем записать».
    ///
    /// `now` считывается здесь и передаётся внутрь, чтобы домен оставался
    /// свободным от неявного обращения к часам и был детерминированным в тестах.
    pub async fn handle(&self, cmd: CreateCustomerCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let customer = Customer::create(cmd.customer_id, now);

        self.repository.save(&customer).await?;

        Ok(())
    }
}

/// Handler for the "fetch a customer by id" use case.
///
/// Обработчик сценария «получить клиента по идентификатору».
pub struct GetCustomerHandler {
    repository: Arc<dyn CustomerRepository>,
}

impl GetCustomerHandler {
    /// Builds the handler around a repository adapter.
    ///
    /// Создаёт обработчик поверх адаптера репозитория.
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    /// Returns the customer, or `None` if no such customer exists.
    ///
    /// A read-only use case with no domain rules to apply, so the handler
    /// delegates straight to the port. Turning `None` into a `404` is the HTTP
    /// layer's decision, not this layer's: absence is not an application error.
    ///
    /// Возвращает клиента либо `None`, если такого клиента нет.
    ///
    /// Сценарий только для чтения, доменных правил здесь нет, поэтому
    /// обработчик напрямую делегирует порту. Преобразование `None` в `404` —
    /// решение HTTP-слоя, а не этого: отсутствие не является ошибкой приложения.
    pub async fn handle(
        &self,
        customer_id: CustomerId,
    ) -> Result<Option<Customer>, ApplicationError> {
        Ok(self.repository.find_by_id(customer_id).await?)
    }
}
