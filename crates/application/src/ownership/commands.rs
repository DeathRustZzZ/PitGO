//! Commands of the vehicle-ownership context.
//!
//! Команды контекста владения автомобилем.

use domain::vehicle_ownership::state::OwnershipType;
use domain::{CustomerId, VehicleId, VehicleOwnershipId};

/// Request to start a vehicle ownership.
///
/// A plain data carrier with no behavior: validation and business rules belong
/// to the domain, and keeping the command inert means there is exactly one
/// place where a rule can live.
///
/// `ownership_id` is supplied by the caller rather than generated here, which
/// makes the operation idempotent: a retried request reuses the same id, and
/// the repository's version check rejects the duplicate instead of silently
/// creating a second ownership record.
///
/// Запрос на начало владения автомобилем.
///
/// Простой носитель данных без поведения: валидация и бизнес-правила
/// принадлежат домену, а инертность команды означает, что у правила есть ровно
/// одно место, где оно может находиться.
///
/// `ownership_id` передаётся вызывающей стороной, а не генерируется здесь, что
/// делает операцию идемпотентной: повторный запрос использует тот же
/// идентификатор, и проверка версии в репозитории отклоняет дубликат, вместо
/// того чтобы молча создать вторую запись о владении.
#[derive(Debug, Clone)]
pub struct StartVehicleOwnershipCommand {
    /// Identifier to assign to the new ownership record.
    ///
    /// Идентификатор, присваиваемый новой записи о владении.
    pub ownership_id: VehicleOwnershipId,
    /// Vehicle the ownership is being started for.
    ///
    /// Автомобиль, для которого создаётся владение.
    pub vehicle_id: VehicleId,
    /// Customer to be recorded as the owner.
    ///
    /// Клиент, который будет зафиксирован как владелец.
    pub owner_customer_id: CustomerId,
    /// Kind of ownership relationship.
    ///
    /// Тип отношения владения.
    pub ownership_type: OwnershipType,
}
