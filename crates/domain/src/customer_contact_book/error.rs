use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PhoneNumberError {
    /// Пользователь ничего не ввел или передал строку только из пробелов.
    #[error("phone number is empty")]
    Empty,

    /// После очистки строка не соответствует поддерживаемым белорусским форматам.
    #[error("invalid Belarus phone number format")]
    InvalidFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CustomerContactBookError {
    #[error("phone contact already exists")]
    PhoneAlreadyExists,
}
