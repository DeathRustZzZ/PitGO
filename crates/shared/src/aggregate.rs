//! Aggregate versioning primitives and command outcome types.
//!
//! These types encode the optimistic-locking contract shared by every aggregate
//! in the system: an aggregate carries a monotonically increasing version, and
//! a command reports whether it actually changed state.
//!
//! Примитивы управления версиями агрегатов и результатов изменений.
//!
//! Эти типы кодируют контракт оптимистичной блокировки, общий для всех
//! агрегатов системы: агрегат несёт монотонно возрастающую версию, а команда
//! сообщает, изменила ли она состояние в действительности.

/// Aggregate version used for optimistic locking.
///
/// Starts at zero when the aggregate is constructed and increases by exactly
/// one per accepted domain event. On write the repository compares the expected
/// version against the stored one: a mismatch means another writer modified the
/// aggregate concurrently, so the write is rejected instead of silently
/// overwriting their change.
///
/// Версия агрегата для оптимистичной блокировки.
///
/// Начинается с нуля при создании агрегата и увеличивается ровно на единицу на
/// каждое принятое доменное событие. При записи репозиторий сравнивает
/// ожидаемую версию с сохранённой: расхождение означает, что агрегат был
/// изменён конкурентным писателем, поэтому запись отклоняется вместо
/// молчаливой перезаписи чужого изменения.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AggregateVersion(u64);

impl AggregateVersion {
    /// Initial version of a freshly constructed aggregate.
    ///
    /// A persisted aggregate is never at `INITIAL`: the creation event raises
    /// the version to 1 before the aggregate leaves its constructor.
    ///
    /// Начальная версия только что созданного агрегата.
    ///
    /// Сохранённый агрегат никогда не находится в `INITIAL`: событие создания
    /// поднимает версию до 1 ещё до выхода из конструктора.
    pub const INITIAL: Self = Self(0);

    /// Returns the next version.
    ///
    /// Возвращает следующую версию.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Returns the raw numeric value of the version.
    ///
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

/// Outcome of executing a command against an aggregate.
///
/// `NoChange` is deliberately not an error: it is the idempotent outcome of a
/// command that found the aggregate already in the requested state. Modelling
/// it as a success value is what makes commands safe to retry — a redelivered
/// message must not fail, and must not inflate the aggregate version either.
///
/// Результат выполнения команды над агрегатом.
///
/// `NoChange` намеренно не является ошибкой: это идемпотентный исход команды,
/// обнаружившей агрегат уже в запрошенном состоянии. Моделирование его как
/// успешного значения делает команды безопасными для повтора — повторно
/// доставленное сообщение не должно завершаться ошибкой и не должно
/// увеличивать версию агрегата.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOutcome {
    /// Aggregate state changed; the version was incremented.
    ///
    /// Состояние агрегата изменилось; версия была увеличена.
    Changed,
    /// Command was a no-op; the version is unchanged.
    ///
    /// Команда не привела к изменению состояния; версия не изменилась.
    NoChange,
}
