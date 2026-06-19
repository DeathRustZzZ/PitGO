//! Статус жизненного цикла клиента.

/// Текущее состояние агрегата `Customer`.
///
/// Допустимые переходы кодируются именованными методами агрегата и
/// exhaustive match. Отдельный объект правил переходов не используется.
///
/// Реализованы только состояния текущего MVP-среза.
/// `Suspended`, `Blocked`, `Deleted` добавляются отдельными задачами.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerStatus {
    Draft,
    Active,
}

/// Дискриминатор для сообщений об ошибках без заимствования `CustomerStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerStatusKind {
    Draft,
    Active,
}

impl CustomerStatus {
    pub fn kind(&self) -> CustomerStatusKind {
        match self {
            Self::Draft => CustomerStatusKind::Draft,
            Self::Active => CustomerStatusKind::Active,
        }
    }
}

impl std::fmt::Display for CustomerStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Active => write!(f, "Active"),
        }
    }
}
