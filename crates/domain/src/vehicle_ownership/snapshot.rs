//! State snapshot used to check cross-aggregate invariants when starting an
//! ownership.
//!
//! Снимок состояния для проверки кросс-агрегатных инвариантов при создании
//! владения.

use crate::ids::VehicleId;

/// Snapshot of the current ownership situation for one specific vehicle.
///
/// Built by the application service immediately before calling
/// [`super::VehicleOwnership::start`]. It lets the aggregate check the "at most
/// one open ownership" invariant without reading other aggregates itself — the
/// domain layer performs no I/O, so a fact it cannot fetch must instead be
/// handed to it.
///
/// This is a capability object: the aggregate trusts the snapshot's contents
/// unconditionally, and the application service is responsible for its
/// freshness. That trust is bounded on purpose. A snapshot is read before the
/// write and can go stale in between, so it cannot be the sole guarantee under
/// concurrency; the database's partial unique index is what ultimately enforces
/// the rule. The snapshot's job is to fail the common case early, cheaply, and
/// with a precise domain error rather than a constraint violation.
///
/// Снимок текущей ситуации с владением для одного конкретного автомобиля.
///
/// Создаётся сервисом приложения непосредственно перед вызовом
/// [`super::VehicleOwnership::start`]. Позволяет агрегату проверить инвариант
/// «не более одного открытого владения», не читая другие агрегаты
/// самостоятельно: доменный слой не выполняет ввод-вывод, поэтому факт,
/// который он не может получить сам, должен быть ему передан.
///
/// Это capability-объект: агрегат безусловно доверяет содержимому снимка, а за
/// его актуальность отвечает сервис приложения. Такое доверие намеренно
/// ограничено. Снимок читается до записи и за это время может устареть,
/// поэтому он не может быть единственной гарантией при конкурентном доступе;
/// окончательно правило обеспечивает частичный уникальный индекс базы данных.
/// Задача снимка — отклонить типичный случай рано, дёшево и с точной доменной
/// ошибкой, а не с нарушением ограничения СУБД.
#[derive(Debug, Clone)]
pub struct OwnershipEligibilitySnapshot {
    vehicle_id: VehicleId,
    has_open_ownership: bool,
}

impl OwnershipEligibilitySnapshot {
    /// Captures the eligibility facts for a vehicle.
    ///
    /// `has_open_ownership` must reflect *open* records — both
    /// `PendingVerification` and `Active` — not only confirmed ones. Passing
    /// confirmed-only data here would let a second claim slip past an
    /// unverified first one.
    ///
    /// Фиксирует сведения о пригодности для автомобиля.
    ///
    /// `has_open_ownership` должен отражать *открытые* записи — как
    /// `PendingVerification`, так и `Active`, — а не только подтверждённые.
    /// Передача сюда данных только по подтверждённым записям позволила бы
    /// второму притязанию проскользнуть мимо первого, неподтверждённого.
    pub fn new(vehicle_id: VehicleId, has_open_ownership: bool) -> Self {
        Self {
            vehicle_id,
            has_open_ownership,
        }
    }

    /// Vehicle this snapshot describes.
    ///
    /// Автомобиль, который описывает данный снимок.
    pub fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    /// Returns `true` if the vehicle currently has no open ownership.
    ///
    /// Возвращает `true`, если у автомобиля сейчас нет открытого владения.
    pub fn no_open_ownership_exists(&self) -> bool {
        !self.has_open_ownership
    }
}
