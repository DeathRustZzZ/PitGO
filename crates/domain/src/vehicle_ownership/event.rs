//! Typed domain events of the `VehicleOwnership` aggregate.
//!
//! Event payload structs carry a `V1` suffix because events are a published
//! contract: a breaking change introduces a `V2` alongside `V1` rather than
//! editing `V1` in place, so already-stored events stay readable.
//!
//! Типизированные доменные события агрегата `VehicleOwnership`.
//!
//! Структуры полезной нагрузки событий имеют суффикс `V1`, потому что события —
//! это опубликованный контракт: ломающее изменение вводит `V2` рядом с `V1`, а
//! не правит `V1` на месте, чтобы уже сохранённые события оставались читаемыми.

use crate::ids::{CustomerId, VehicleId, VehicleOwnershipId};
use crate::vehicle_ownership::state::OwnershipType;

/// An ownership record was created; status `PendingVerification`.
///
/// Carries the full set of identifying fields, unlike the later events. A
/// consumer building its own read model needs the whole relationship from this
/// one event — subsequent events only reference the ownership by id and would
/// be meaningless on their own.
///
/// Запись о владении создана; статус `PendingVerification`.
///
/// В отличие от последующих событий, несёт полный набор идентифицирующих
/// полей. Потребителю, строящему собственную модель чтения, всё отношение
/// требуется получить из этого единственного события: дальнейшие события
/// ссылаются на владение только по идентификатору и сами по себе бессмысленны.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipStartedV1 {
    /// Identifier of the created ownership record.
    ///
    /// Идентификатор созданной записи о владении.
    pub ownership_id: VehicleOwnershipId,
    /// Vehicle the ownership refers to.
    ///
    /// Автомобиль, к которому относится владение.
    pub vehicle_id: VehicleId,
    /// Customer recorded as the owner.
    ///
    /// Клиент, зафиксированный как владелец.
    pub owner_customer_id: CustomerId,
    /// Kind of ownership relationship.
    ///
    /// Тип отношения владения.
    pub ownership_type: OwnershipType,
}

/// An ownership record was confirmed; status `Active`.
///
/// Запись о владении подтверждена; статус `Active`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipVerifiedV1 {
    /// Identifier of the confirmed ownership record.
    ///
    /// Идентификатор подтверждённой записи о владении.
    pub ownership_id: VehicleOwnershipId,
}

/// An ownership record was terminated; status `Ended` (terminal).
///
/// Запись о владении завершена; статус `Ended` (терминальный).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleOwnershipEndedV1 {
    /// Identifier of the terminated ownership record.
    ///
    /// Идентификатор завершённой записи о владении.
    pub ownership_id: VehicleOwnershipId,
}

/// All domain events of the `VehicleOwnership` aggregate.
///
/// Each variant holds a concrete, typed payload. The `EventEnvelope` is built
/// at the persistence boundary, not inside the aggregate.
///
/// Перечень всех доменных событий агрегата `VehicleOwnership`.
///
/// Каждый вариант содержит конкретный типизированный payload. `EventEnvelope`
/// создаётся на границе персистентности, а не внутри агрегата.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleOwnershipEvent {
    /// See [`VehicleOwnershipStartedV1`].
    ///
    /// См. [`VehicleOwnershipStartedV1`].
    Started(VehicleOwnershipStartedV1),
    /// See [`VehicleOwnershipVerifiedV1`].
    ///
    /// См. [`VehicleOwnershipVerifiedV1`].
    Verified(VehicleOwnershipVerifiedV1),
    /// See [`VehicleOwnershipEndedV1`].
    ///
    /// См. [`VehicleOwnershipEndedV1`].
    Ended(VehicleOwnershipEndedV1),
}
