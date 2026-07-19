//! In-memory adapter for the [`CustomerRepository`] port.
//!
//! Адаптер в памяти для порта [`CustomerRepository`].

use std::collections::HashMap;
use std::sync::Mutex;

use application::customer::ports::CustomerRepository;
use application::error::RepositoryError;
use domain::CustomerId;
use domain::customer::Customer;

/// In-memory implementation of [`CustomerRepository`].
///
/// Intended for tests and for local runs before the PostgreSQL adapter exists.
/// State is lost on restart.
///
/// # Why `Mutex` and not `RwLock` or a Tokio mutex
///
/// The port takes `&self`, so the adapter needs interior mutability to serve
/// concurrent tasks through a single shared `Arc`. `std::sync::Mutex` — not
/// `tokio::sync::Mutex` — is the right choice here specifically because no lock
/// guard is ever held across an `.await`: every method locks, does purely
/// synchronous work, and drops the guard before returning. A blocking mutex is
/// cheaper, and the usual hazard of stalling a Tokio worker thread cannot
/// arise. Anyone adding an `.await` inside a locked section must revisit this
/// decision — that is the change that would make it unsound.
///
/// `RwLock` would buy little: the critical sections are a hash lookup and an
/// insert, short enough that reader parallelism would not repay the extra cost.
///
/// Реализация [`CustomerRepository`] в памяти.
///
/// Предназначена для тестов и локальных запусков до появления адаптера
/// PostgreSQL. Состояние теряется при перезапуске.
///
/// # Почему `Mutex`, а не `RwLock` или мьютекс из Tokio
///
/// Порт принимает `&self`, поэтому адаптеру нужна внутренняя изменяемость,
/// чтобы обслуживать конкурентные задачи через один общий `Arc`.
/// `std::sync::Mutex`, а не `tokio::sync::Mutex`, выбран именно потому, что
/// охранник блокировки никогда не удерживается через `.await`: каждый метод
/// захватывает блокировку, выполняет исключительно синхронную работу и
/// освобождает её до возврата. Блокирующий мьютекс дешевле, а обычная угроза
/// остановки рабочего потока Tokio здесь возникнуть не может. Тот, кто добавит
/// `.await` внутрь заблокированного участка, обязан пересмотреть это решение —
/// именно такое изменение сделало бы его некорректным.
///
/// `RwLock` дал бы немного: критические секции — это поиск в хеш-таблице и
/// вставка, они слишком коротки, чтобы параллелизм читателей окупил
/// дополнительные издержки.
pub struct InMemoryCustomerRepository {
    customers: Mutex<HashMap<CustomerId, Customer>>,
}

impl InMemoryCustomerRepository {
    /// Creates an empty repository.
    ///
    /// Создаёт пустой репозиторий.
    pub fn new() -> Self {
        Self {
            customers: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl CustomerRepository for InMemoryCustomerRepository {
    /// Saves a customer, enforcing the optimistic-locking contract.
    ///
    /// The expected version is derived from what is already stored: a write is
    /// only accepted if the incoming aggregate is exactly one version ahead of
    /// the stored one. When nothing is stored yet, `expected` is `None` and any
    /// version is accepted, which is how a first insert succeeds.
    ///
    /// This is also what rejects a duplicate create. A second
    /// `Customer::create` for the same id arrives at version 1 while the store
    /// already holds version 1 and therefore expects 2 — so the duplicate
    /// surfaces as a `VersionConflict` and no separate existence check is
    /// needed.
    ///
    /// Сохраняет клиента, обеспечивая контракт оптимистичной блокировки.
    ///
    /// Ожидаемая версия выводится из уже сохранённого значения: запись
    /// принимается, только если входящий агрегат ровно на одну версию впереди
    /// сохранённого. Если ничего ещё не сохранено, `expected` равно `None` и
    /// принимается любая версия — так проходит первая вставка.
    ///
    /// Этим же отклоняется повторное создание. Второй вызов `Customer::create`
    /// для того же идентификатора приходит с версией 1, тогда как в хранилище
    /// уже лежит версия 1 и, значит, ожидается 2 — поэтому дубликат проявляется
    /// как `VersionConflict`, и отдельная проверка существования не нужна.
    async fn save(&self, customer: &Customer) -> Result<(), application::error::RepositoryError> {
        // A poisoned lock means another thread panicked mid-write, so the map
        // may be inconsistent. Reported as a storage failure rather than
        // unwrapped, so one panicking request cannot bring down the process.
        //
        // Отравленная блокировка означает, что другой поток запаниковал во
        // время записи, поэтому таблица может быть несогласованной. Сообщается
        // как сбой хранилища, а не разворачивается через unwrap, чтобы один
        // запаниковавший запрос не обрушил процесс.
        let mut customers = self
            .customers
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let actual = customer.version();
        let expected = customers
            .get(&customer.id())
            .map(|stored| stored.version().next());

        if let Some(expected_version) = expected
            && expected_version != actual
        {
            return Err(RepositoryError::VersionConflict {
                expected: expected_version.value(),
                actual: actual.value(),
            });
        }
        customers.insert(customer.id(), customer.clone());

        Ok(())
    }

    /// Returns the stored customer, or `None` if absent.
    ///
    /// The aggregate is cloned so the caller cannot mutate the stored copy
    /// through a shared reference — the in-memory stand-in for a database
    /// returning a fresh row per query.
    ///
    /// Возвращает сохранённого клиента либо `None`, если его нет.
    ///
    /// Агрегат клонируется, чтобы вызывающая сторона не могла изменить
    /// сохранённую копию через общую ссылку — это замена в памяти для базы
    /// данных, возвращающей на каждый запрос свежую строку.
    async fn find_by_id(
        &self,
        customer_id: CustomerId,
    ) -> Result<Option<Customer>, RepositoryError> {
        let customers = self
            .customers
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(customers.get(&customer_id).cloned())
    }
}

impl Default for InMemoryCustomerRepository {
    fn default() -> Self {
        Self::new()
    }
}
