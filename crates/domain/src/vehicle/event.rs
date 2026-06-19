//! Типизированные доменные события агрегата `Vehicle`.

use crate::ids::VehicleId;

/// Автомобиль создан в статусе `Draft`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleCreatedV1 {
    pub vehicle_id: VehicleId,
}

/// Автомобиль переведён в статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleActivatedV1 {
    pub vehicle_id: VehicleId,
}

/// Перечень всех доменных событий агрегата `Vehicle`.
///
/// Каждый вариант содержит конкретный типизированный payload.
/// `EventEnvelope` создаётся на persistence-границе, а не внутри агрегата.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleEvent {
    Created(VehicleCreatedV1),
    Activated(VehicleActivatedV1),
}
