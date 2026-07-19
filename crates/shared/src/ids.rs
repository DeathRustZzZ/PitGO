//! Type-safe identifiers and the macro that generates them.
//!
//! Every identifier in the system is a distinct newtype over `Uuid` rather than
//! a bare `Uuid`. The cost is a macro; the benefit is that passing a
//! `CustomerId` where a `VehicleId` is expected becomes a compile error instead
//! of a data-corruption bug that surfaces in production.
//!
//! Типобезопасные идентификаторы и макрос для их генерации.
//!
//! Каждый идентификатор в системе — отдельный newtype над `Uuid`, а не «голый»
//! `Uuid`. Цена — макрос; выгода — передача `CustomerId` туда, где ожидается
//! `VehicleId`, становится ошибкой компиляции, а не багом порчи данных,
//! проявляющимся в продакшене.

/// Generates a type-safe identifier wrapping a `Uuid`.
///
/// The generated type is `#[repr(transparent)]` and `Copy`, so it costs nothing
/// at runtime compared to a raw `Uuid`. Doc comments passed to the macro are
/// forwarded to the generated struct, which is why each invocation below
/// carries its own documentation.
///
/// Генерирует типобезопасный идентификатор-обёртку над `Uuid`.
///
/// Генерируемый тип имеет `#[repr(transparent)]` и `Copy`, поэтому в рантайме
/// не стоит ничего по сравнению с «голым» `Uuid`. Doc-комментарии, переданные
/// в макрос, пробрасываются в генерируемую структуру — поэтому каждый вызов
/// ниже несёт собственную документацию.
#[macro_export]
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(::uuid::Uuid);

        // `new()` generates a random value, so a `Default` impl would silently
        // produce a fresh identity on every call — misleading enough that the
        // lint is suppressed rather than satisfied.
        //
        // `new()` порождает случайное значение, поэтому реализация `Default`
        // молча создавала бы новую сущность при каждом вызове — это достаточно
        // обманчиво, чтобы подавить линт, а не удовлетворить его.
        #[allow(clippy::new_without_default)]
        impl $name {
            /// Creates a new unique identifier (UUID v4).
            ///
            /// Создаёт новый уникальный идентификатор (UUID v4).
            #[must_use]
            pub fn new() -> Self {
                Self(::uuid::Uuid::new_v4())
            }

            /// Creates an identifier from an existing UUID.
            ///
            /// Used at trust boundaries — HTTP requests, database rows — where
            /// the UUID already exists and must not be regenerated.
            ///
            /// Создаёт идентификатор из существующего UUID.
            ///
            /// Используется на границах доверия — HTTP-запросы, строки базы
            /// данных — где UUID уже существует и не должен генерироваться
            /// заново.
            #[must_use]
            pub const fn from_uuid(value: ::uuid::Uuid) -> Self {
                Self(value)
            }

            /// Returns the wrapped UUID value.
            ///
            /// Возвращает внутреннее значение UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> ::uuid::Uuid {
                self.0
            }
        }

        impl From<::uuid::Uuid> for $name {
            fn from(value: ::uuid::Uuid) -> Self {
                Self::from_uuid(value)
            }
        }

        impl From<$name> for ::uuid::Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

define_id!(
    /// Identifier of the actor (user, service or the system) that performed an action.
    ///
    /// Идентификатор актора (пользователь, сервис или система), выполнившего действие.
    ActorId
);

define_id!(
    /// Correlation identifier — groups related events belonging to one business process.
    ///
    /// Идентификатор корреляции — связывает группу связанных событий в одном бизнес-процессе.
    CorrelationId
);

define_id!(
    /// Causation identifier — references the command or event that triggered the current action.
    ///
    /// Идентификатор причины — ссылка на команду или событие, вызвавшее текущее действие.
    CausationId
);

define_id!(
    /// Unique identifier of a domain event.
    ///
    /// Уникальный идентификатор доменного события.
    EventId
);
