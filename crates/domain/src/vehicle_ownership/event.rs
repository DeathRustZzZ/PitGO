//! Типизированные доменные события агрегата `VehicleOwnership`.

use crate::ids::{CustomerId, VehicleId, VehicleOwnershipId};
use crate::vehicle_ownership::state::OwnershipType;

/// Запись о владении создана; статус `PendingVerification`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipStartedV1 {
    pub ownership_id: VehicleOwnershipId,
    pub vehicle_id: VehicleId,
    pub owner_customer_id: CustomerId,
    pub ownership_type: OwnershipType,
}

/// Запись о владении подтверждена; статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipVerifiedV1 {
    pub ownership_id: VehicleOwnershipId,
}

/// Запись о владении завершена; статус `Ended` (терминальный).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipEndedV1 {
    pub ownership_id: VehicleOwnershipId,
}

/// Перечень всех доменных событий агрегата `VehicleOwnership`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleOwnershipEvent {
    Started(VehicleOwnershipStartedV1),
    Verified(VehicleOwnershipVerifiedV1),
    Ended(VehicleOwnershipEndedV1),
}
