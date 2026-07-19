//! The `Vehicle` aggregate — identity and lifecycle of a single vehicle in PitGO.
//!
//! Агрегат `Vehicle` — identity и lifecycle конкретного автомобиля в PitGO.

use chrono::{DateTime, Utc};

use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::ids::VehicleId;
use crate::vehicle::error::VehicleError;
use crate::vehicle::event::{VehicleActivatedV1, VehicleCreatedV1, VehicleEvent};
use crate::vehicle::permit::VehicleActivationPermit;
use crate::vehicle::state::VehicleStatus;

/// Aggregate root of a single vehicle.
///
/// Holds identity, lifecycle status and version. `VehicleSpecs`, VIN and
/// license plate are implemented by separate tasks. `VehicleOwnership` is a
/// distinct aggregate root — a vehicle does not own its ownership records, it
/// is merely referenced by them.
///
/// # Invariants
///
/// - Fields are private; state changes only through named command methods.
/// - The version increases by exactly one per raised event.
/// - A vehicle always exists in exactly one [`VehicleStatus`].
///
/// # Note on `Clone`
///
/// `Clone` is a temporary concession to the tests and the in-memory repository,
/// which store aggregates by value. A real Postgres repository rehydrates each
/// aggregate from its row instead, so this derive should be removed once that
/// repository lands — cloning an aggregate risks two live copies drifting apart
/// in version.
///
/// Корень агрегата конкретного автомобиля.
///
/// Хранит identity, статус жизненного цикла и версию. `VehicleSpecs`, VIN и
/// государственный номер реализуются отдельными задачами. `VehicleOwnership` —
/// самостоятельный корень агрегата: автомобиль не владеет записями о владении,
/// они лишь ссылаются на него.
///
/// # Инварианты
///
/// - Поля приватные; состояние изменяется только именованными командными методами.
/// - Версия увеличивается ровно на единицу на каждое порождённое событие.
/// - Автомобиль всегда находится ровно в одном статусе [`VehicleStatus`].
///
/// # Замечание о `Clone`
///
/// `Clone` — временная уступка тестам и репозиторию в памяти, которые хранят
/// агрегаты по значению. Настоящий Postgres-репозиторий вместо этого
/// восстанавливает агрегат из строки таблицы, поэтому данный derive следует
/// убрать после его реализации: клонирование агрегата создаёт риск расхождения
/// версий двух живых копий.
#[derive(Debug, Clone)]
pub struct Vehicle {
    id: VehicleId,
    status: VehicleStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingEvent<VehicleEvent>>,
}

impl Vehicle {
    /// Creates a new vehicle in the `Draft` status.
    ///
    /// `VehicleSpecs` and `VehicleIdentity` will be added once the
    /// corresponding value objects are implemented. Until then a `Draft`
    /// vehicle is identity-only, which is precisely why activation is gated
    /// behind a permit rather than being allowed at creation.
    ///
    /// Создаёт новый автомобиль в статусе `Draft`.
    ///
    /// `VehicleSpecs` и `VehicleIdentity` будут добавлены после реализации
    /// соответствующих объектов-значений. До тех пор автомобиль в статусе
    /// `Draft` содержит только identity — именно поэтому активация закрыта
    /// permit, а не разрешена сразу при создании.
    pub fn create(id: VehicleId, now: DateTime<Utc>) -> Self {
        let mut vehicle = Self {
            id,
            status: VehicleStatus::Draft,
            created_at: now,
            updated_at: now,
            version: AggregateVersion::INITIAL,
            pending_events: Vec::new(),
        };
        vehicle.raise(
            VehicleEvent::Created(VehicleCreatedV1 { vehicle_id: id }),
            now,
        );
        vehicle
    }

    /// Moves the vehicle from `Draft` to `Active`.
    ///
    /// Validates only the permit's local conditions — matching `vehicle_id` and
    /// expiry. The substantive check, that the vehicle carries a trustworthy
    /// identifier (a VIN, or a trusted external reference plus a license
    /// plate), is performed by `VehicleActivationPolicy` before the permit is
    /// issued.
    ///
    /// Unlike [`crate::customer::Customer::activate`], this permit is not bound
    /// to an aggregate version: a vehicle's identifying facts do not change in
    /// ways that would invalidate the eligibility decision, so pinning the
    /// version would only cause spurious rejections.
    ///
    /// Returns `NoChange` without touching state if the vehicle is already
    /// `Active`.
    ///
    /// Переводит автомобиль из `Draft` в `Active`.
    ///
    /// Проверяет только локальные условия permit — совпадение `vehicle_id` и
    /// срок действия. Содержательная проверка наличия надёжного идентификатора
    /// (VIN либо доверенная внешняя ссылка вместе с государственным номером)
    /// выполняется `VehicleActivationPolicy` до выдачи permit.
    ///
    /// В отличие от [`crate::customer::Customer::activate`], этот permit не
    /// привязан к версии агрегата: идентифицирующие сведения об автомобиле не
    /// меняются так, чтобы обесценить решение о пригодности, поэтому привязка
    /// к версии приводила бы лишь к ложным отказам.
    ///
    /// Возвращает `NoChange`, не изменяя состояние, если автомобиль уже `Active`.
    pub fn activate(
        &mut self,
        permit: VehicleActivationPermit,
        now: DateTime<Utc>,
    ) -> Result<ChangeOutcome, VehicleError> {
        permit.validate_local(self.id, now)?;

        match &self.status {
            VehicleStatus::Draft => {
                self.status = VehicleStatus::Active;
                self.updated_at = now;
                self.raise(
                    VehicleEvent::Activated(VehicleActivatedV1 {
                        vehicle_id: self.id,
                    }),
                    now,
                );
                Ok(ChangeOutcome::Changed)
            }
            VehicleStatus::Active => Ok(ChangeOutcome::NoChange),
        }
    }

    /// Returns the buffered domain events and clears the internal buffer.
    ///
    /// Called by the persistence layer once the aggregate has been saved.
    /// Draining is intentional: events must be published exactly once.
    ///
    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    ///
    /// Вызывается слоем персистентности после сохранения агрегата. Опустошение
    /// буфера намеренно: события должны публиковаться ровно один раз.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<VehicleEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Getters ───────────────────────────────────────────────────────────
    // ── Геттеры ───────────────────────────────────────────────────────────

    pub fn id(&self) -> VehicleId {
        self.id
    }

    pub fn status(&self) -> &VehicleStatus {
        &self.status
    }

    /// Current version, used by repositories for the optimistic-locking check.
    ///
    /// Текущая версия, используемая репозиториями для проверки оптимистичной блокировки.
    pub fn version(&self) -> AggregateVersion {
        self.version
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ── Private helpers ───────────────────────────────────────────────────
    // ── Приватные вспомогательные методы ──────────────────────────────────

    /// Buffers an event and advances the aggregate version.
    ///
    /// Routing every state change through this single method keeps the
    /// "one event, one version increment" invariant true by construction.
    ///
    /// Записывает событие в буфер и увеличивает версию агрегата.
    ///
    /// Проведение каждого изменения состояния через этот единственный метод
    /// делает инвариант «одно событие — одно увеличение версии» истинным по
    /// построению.
    fn raise(&mut self, event: VehicleEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod aggregate_tests;
