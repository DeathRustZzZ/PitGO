//! Type-safe identifiers of domain entities.
//!
//! The `define_id!` macro is re-exported from `shared`. This module defines the
//! identifiers specific to this bounded context. Each aggregate root gets its
//! own identifier type so that references between aggregates — for example
//! `VehicleOwnership` holding a `CustomerId` and a `VehicleId` — cannot be
//! mixed up at a call site.
//!
//! Типобезопасные идентификаторы доменных сущностей.
//!
//! Макрос `define_id!` экспортируется из `shared`. Здесь определяются
//! идентификаторы, специфичные для данного ограниченного контекста. Каждый
//! корень агрегата получает собственный тип идентификатора, чтобы ссылки между
//! агрегатами — например, `VehicleOwnership`, хранящий `CustomerId` и
//! `VehicleId`, — нельзя было перепутать в месте вызова.

use shared::define_id;

define_id!(
    /// Type-safe identifier of a customer.
    ///
    /// Типобезопасный идентификатор клиента.
    CustomerId
);

define_id!(
    /// Type-safe identifier of a vehicle.
    ///
    /// Типобезопасный идентификатор автомобиля.
    VehicleId
);

define_id!(
    /// Type-safe identifier of a vehicle ownership record.
    ///
    /// Типобезопасный идентификатор записи о владении автомобилем.
    VehicleOwnershipId
);
