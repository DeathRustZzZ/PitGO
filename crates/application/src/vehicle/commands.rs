//! Commands of the vehicle context.
//!
//! Команды контекста «Автомобиль».

use domain::VehicleId;

/// Request to create a new vehicle.
///
/// A plain data carrier with no behavior; business rules stay in the domain.
///
/// `vehicle_id` is supplied by the caller rather than generated here, which
/// makes creation idempotent: a retried request reuses the same id, and the
/// repository's version check rejects the duplicate instead of creating a
/// second vehicle.
///
/// Запрос на создание нового автомобиля.
///
/// Простой носитель данных без поведения; бизнес-правила остаются в домене.
///
/// `vehicle_id` передаётся вызывающей стороной, а не генерируется здесь, что
/// делает создание идемпотентным: повторный запрос использует тот же
/// идентификатор, и проверка версии в репозитории отклоняет дубликат, вместо
/// того чтобы создать второй автомобиль.
#[derive(Debug, Clone, Copy)]
pub struct CreateVehicleCommand {
    /// Identifier to assign to the new vehicle.
    ///
    /// Идентификатор, присваиваемый новому автомобилю.
    pub vehicle_id: VehicleId,
}
