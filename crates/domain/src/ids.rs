// Типобезопасные идентификаторы для различных сущностей в домене.
use uuid::Uuid;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

}
/// Типобезопасный идентификатор для клиента.
/// Используется для идентификации клиентов в системе, обеспечивая уникальность и безопасность типов.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(Uuid);

// impl ClientId {
//     /// Cоздает новый уникальный идентификатор клиента.
//     pub fn new() -> Self {
//         Self(Uuid::new_v4())
//     }

//     /// Cоздает идентификатор клиента из существующего UUID.
//     pub fn from_uuid(value: Uuid) -> Self {
//         Self(value)
//     }

//     /// Получает внутреннее значение UUID.
//     pub fn as_uuid(&self) -> Uuid {
//         self.0
//     }
// }

/// Типобезопасный идентификатор для автомобиля.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CarId(Uuid);
