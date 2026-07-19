//! Typed domain events of the `Vehicle` aggregate.
//!
//! Event payload structs carry a `V1` suffix because events are a published
//! contract: a breaking change introduces `VehicleCreatedV2` alongside `V1`
//! rather than editing `V1` in place, so already-stored events stay readable.
//!
//! Типизированные доменные события агрегата `Vehicle`.
//!
//! Структуры полезной нагрузки событий имеют суффикс `V1`, потому что события —
//! это опубликованный контракт: ломающее изменение вводит `VehicleCreatedV2`
//! рядом с `V1`, а не правит `V1` на месте, чтобы уже сохранённые события
//! оставались читаемыми.

use crate::ids::VehicleId;

/// A vehicle was created in the `Draft` status.
///
/// Автомобиль создан в статусе `Draft`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleCreatedV1 {
    /// Identifier of the created vehicle.
    ///
    /// Идентификатор созданного автомобиля.
    pub vehicle_id: VehicleId,
}

/// A vehicle was moved into the `Active` status.
///
/// Автомобиль переведён в статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleActivatedV1 {
    /// Identifier of the activated vehicle.
    ///
    /// Идентификатор активированного автомобиля.
    pub vehicle_id: VehicleId,
}

/// All domain events of the `Vehicle` aggregate.
///
/// Each variant holds a concrete, typed payload. The `EventEnvelope` is built
/// at the persistence boundary, not inside the aggregate, so the domain never
/// has to know how or when its events will be stored.
///
/// Перечень всех доменных событий агрегата `Vehicle`.
///
/// Каждый вариант содержит конкретный типизированный payload. `EventEnvelope`
/// создаётся на границе персистентности, а не внутри агрегата, поэтому домену
/// не нужно знать, как и когда его события будут сохранены.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleEvent {
    /// See [`VehicleCreatedV1`].
    ///
    /// См. [`VehicleCreatedV1`].
    Created(VehicleCreatedV1),
    /// See [`VehicleActivatedV1`].
    ///
    /// См. [`VehicleActivatedV1`].
    Activated(VehicleActivatedV1),
}
