//! Типобезопасные идентификаторы и макрос для их генерации.

/// Генерирует типобезопасный идентификатор-обёртку над `Uuid`.
#[macro_export]
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(::uuid::Uuid);

        #[allow(clippy::new_without_default)]
        impl $name {
            /// Создаёт новый уникальный идентификатор.
            #[must_use]
            pub fn new() -> Self {
                Self(::uuid::Uuid::new_v4())
            }

            /// Создаёт идентификатор из существующего UUID.
            #[must_use]
            pub const fn from_uuid(value: ::uuid::Uuid) -> Self {
                Self(value)
            }

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
    /// Идентификатор актора (пользователь, сервис или система), выполнившего действие.
    ActorId
);

define_id!(
    /// Идентификатор корреляции — связывает группу связанных событий в одном бизнес-процессе.
    CorrelationId
);

define_id!(
    /// Идентификатор причины — ссылка на команду или событие, вызвавшее текущее действие.
    CausationId
);

define_id!(
    /// Уникальный идентификатор доменного события.
    EventId
);
