//! Агрегат `Vehicle` — identity и lifecycle конкретного автомобиля в PitGO.

use chrono::{DateTime, Utc};

use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::ids::VehicleId;
use crate::vehicle::error::VehicleError;
use crate::vehicle::event::{VehicleActivatedV1, VehicleCreatedV1, VehicleEvent};
use crate::vehicle::permit::VehicleActivationPermit;
use crate::vehicle::state::VehicleStatus;

/// Агрегат конкретного автомобиля.
///
/// Хранит identity, lifecycle-статус и версию. `VehicleSpecs`, VIN и
/// license plate реализуются отдельными задачами. `VehicleOwnership`
/// является отдельным Aggregate Root.
///
/// Поля приватные — состояние изменяется только именованными командами.
#[derive(Debug)]
pub struct Vehicle {
    id: VehicleId,
    status: VehicleStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingEvent<VehicleEvent>>,
}

impl Vehicle {
    /// Создаёт новый автомобиль в статусе `Draft`.
    ///
    /// `VehicleSpecs` и `VehicleIdentity` будут добавлены после реализации
    /// соответствующих value objects.
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

    /// Переводит автомобиль из `Draft` в `Active`.
    ///
    /// Проверяет локальные условия permit (vehicle_id, срок). Глобальная
    /// проверка надёжного идентификатора выполняется `VehicleActivationPolicy`
    /// до выдачи permit.
    ///
    /// Возвращает `NoChange` без изменения состояния, если автомобиль уже `Active`.
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

    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<VehicleEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Геттеры ───────────────────────────────────────────────────────────

    pub fn id(&self) -> VehicleId {
        self.id
    }

    pub fn status(&self) -> &VehicleStatus {
        &self.status
    }

    pub fn version(&self) -> AggregateVersion {
        self.version
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ── Приватные вспомогательные методы ──────────────────────────────────

    /// Записывает событие в буфер и увеличивает версию агрегата.
    fn raise(&mut self, event: VehicleEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod aggregate_tests;
