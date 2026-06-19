//! Примитивы управления версиями агрегатов и результатов изменений.

/// Версия агрегата для оптимистичной блокировки.
///
/// Начинается с нуля при создании агрегата и увеличивается при каждом
/// успешном изменении состояния.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AggregateVersion(u64);

impl AggregateVersion {
    /// Начальная версия нового агрегата.
    pub const INITIAL: Self = Self(0);

    /// Возвращает следующую версию.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Возвращает числовое значение версии.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for AggregateVersion {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<AggregateVersion> for u64 {
    fn from(value: AggregateVersion) -> Self {
        value.0
    }
}

impl std::fmt::Display for AggregateVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Результат выполнения команды над агрегатом.
///
/// `NoChange` не является ошибкой — это идемпотентный исход команды,
/// при котором версия агрегата не увеличивается.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOutcome {
    /// Состояние агрегата изменилось; версия должна быть увеличена.
    Changed,
    /// Команда не привела к изменению состояния.
    NoChange,
}
