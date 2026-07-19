//! Commands of the customer context.
//!
//! Команды контекста «Клиент».

use domain::CustomerId;

/// Request to create a new customer.
///
/// A plain data carrier with no behavior; business rules stay in the domain.
///
/// `customer_id` is supplied by the caller rather than generated here, which
/// makes creation idempotent: a retried request reuses the same id, and the
/// repository's version check rejects the duplicate instead of creating a
/// second customer.
///
/// Запрос на создание нового клиента.
///
/// Простой носитель данных без поведения; бизнес-правила остаются в домене.
///
/// `customer_id` передаётся вызывающей стороной, а не генерируется здесь, что
/// делает создание идемпотентным: повторный запрос использует тот же
/// идентификатор, и проверка версии в репозитории отклоняет дубликат, вместо
/// того чтобы создать второго клиента.
#[derive(Debug, Clone, Copy)]
pub struct CreateCustomerCommand {
    /// Identifier to assign to the new customer.
    ///
    /// Идентификатор, присваиваемый новому клиенту.
    pub customer_id: CustomerId,
}
