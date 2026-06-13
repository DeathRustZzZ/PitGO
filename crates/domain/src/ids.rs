//! Типобезопасные идентификаторы для различных сущностей в домене.

use uuid::Uuid;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        #[allow(clippy::new_without_default)]
        impl $name {
            /// Cоздает новый уникальный идентификатор.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Cоздает идентификатор из существующего UUID.
            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            /// Получает внутреннее значение UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self::from_uuid(value)
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

define_id!(
    /// Типобезопасный идентификатор для клиента.
    /// Используется для идентификации клиентов в системе, обеспечивая уникальность и безопасность типов.
    ClientId
);

define_id!(
    /// Типобезопасный идентификатор для автомобиля.
    CarId
);

define_id!(
    /// Типобезопасный индификатор для записи на обслуживание
    BookingId
);

define_id!(
    /// Типобезопасный идентификатор для поставки запчастей
    PartDeliveryId
);

define_id!(
    /// Типобезопасный идентификатор для запчасти или расходника
    PartId
);

define_id!(
    /// Типобезопасный идентификатор для ремонта
    RepairId
);

define_id!(
    /// Типобезопасный идентификатор для запчасти, используемой в ремонте
    RepairPartId
);

define_id!(
    /// Типобезопасный идентификатор для платежа
    PaymentId
);

define_id!(
    /// Типобезопасный идентификатор для перемещения запчастей
    StockMovementId
);
