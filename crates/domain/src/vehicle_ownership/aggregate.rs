//! Агрегат `VehicleOwnership` — операционное владение автомобилем в PitGO.
//!
//! Не является юридической записью о собственности; моделирует eligibility
//! внутри платформы. Правило «не более одного активного владения» обеспечивается
//! совместно: инвариант на снимке внутри агрегата + частичный unique index в БД.

use chrono::{DateTime, Utc};

use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::ids::{CustomerId, VehicleId, VehicleOwnershipId};
use crate::vehicle_ownership::error::OwnershipError;
use crate::vehicle_ownership::event::{
    VehicleOwnershipEndedV1, VehicleOwnershipEvent, VehicleOwnershipStartedV1,
    VehicleOwnershipVerifiedV1,
};
use crate::vehicle_ownership::snapshot::OwnershipEligibilitySnapshot;
use crate::vehicle_ownership::state::{OwnershipPeriod, OwnershipStatus, OwnershipType};

/// Агрегат записи о владении конкретным автомобилем.
///
/// `CustomerId` является только ссылкой — объект `Customer` не загружается.
/// `VehicleId` является только ссылкой — объект `Vehicle` не загружается.
///
/// Поля приватные — состояние изменяется только именованными командами.
#[derive(Debug)]
pub struct VehicleOwnership {
    id: VehicleOwnershipId,
    vehicle_id: VehicleId,
    owner_customer_id: CustomerId,
    ownership_type: OwnershipType,
    status: OwnershipStatus,
    period: OwnershipPeriod,
    version: AggregateVersion,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pending_events: Vec<PendingEvent<VehicleOwnershipEvent>>,
}

impl VehicleOwnership {
    /// Создаёт новую запись о владении в статусе `PendingVerification`.
    ///
    /// Проверяет через `snapshot`, что для данного автомобиля нет активного
    /// владения. Кросс-агрегатный инвариант обеспечивается application service,
    /// который строит снимок и повторно проверяет версии при commit.
    pub fn start(
        id: VehicleOwnershipId,
        vehicle_id: VehicleId,
        owner_customer_id: CustomerId,
        ownership_type: OwnershipType,
        snapshot: OwnershipEligibilitySnapshot,
        now: DateTime<Utc>,
    ) -> Result<Self, OwnershipError> {
        if !snapshot.no_active_ownership_exists() {
            return Err(OwnershipError::ActiveOwnershipAlreadyExists);
        }

        let mut ownership = Self {
            id,
            vehicle_id,
            owner_customer_id,
            ownership_type: ownership_type.clone(),
            status: OwnershipStatus::PendingVerification,
            period: OwnershipPeriod::new(now),
            version: AggregateVersion::INITIAL,
            created_at: now,
            updated_at: now,
            pending_events: Vec::new(),
        };
        ownership.raise(
            VehicleOwnershipEvent::Started(VehicleOwnershipStartedV1 {
                ownership_id: id,
                vehicle_id,
                owner_customer_id,
                ownership_type,
            }),
            now,
        );
        Ok(ownership)
    }

    /// Подтверждает владение: `PendingVerification → Active`.
    ///
    /// Идемпотентен: возвращает `NoChange` если уже `Active`.
    /// Возвращает ошибку для любого другого статуса.
    pub fn verify(&mut self, now: DateTime<Utc>) -> Result<ChangeOutcome, OwnershipError> {
        match &self.status {
            OwnershipStatus::PendingVerification => {
                self.status = OwnershipStatus::Active;
                self.updated_at = now;
                self.raise(
                    VehicleOwnershipEvent::Verified(VehicleOwnershipVerifiedV1 {
                        ownership_id: self.id,
                    }),
                    now,
                );
                Ok(ChangeOutcome::Changed)
            }
            OwnershipStatus::Active => Ok(ChangeOutcome::NoChange),
            other => Err(OwnershipError::StatusDoesNotAllow(other.kind())),
        }
    }

    /// Завершает владение: `Active → Ended` (терминальное).
    ///
    /// Идемпотентен: возвращает `NoChange` если уже `Ended`.
    /// Возвращает ошибку для `PendingVerification`.
    pub fn end(&mut self, now: DateTime<Utc>) -> Result<ChangeOutcome, OwnershipError> {
        match &self.status {
            OwnershipStatus::Active => {
                self.period = self
                    .period
                    .clone()
                    .close(now)
                    .ok_or(OwnershipError::PeriodEndBeforeStart)?;
                self.status = OwnershipStatus::Ended;
                self.updated_at = now;
                self.raise(
                    VehicleOwnershipEvent::Ended(VehicleOwnershipEndedV1 {
                        ownership_id: self.id,
                    }),
                    now,
                );
                Ok(ChangeOutcome::Changed)
            }
            OwnershipStatus::Ended => Ok(ChangeOutcome::NoChange),
            other => Err(OwnershipError::StatusDoesNotAllow(other.kind())),
        }
    }

    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<VehicleOwnershipEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Геттеры ───────────────────────────────────────────────────────────

    pub fn id(&self) -> VehicleOwnershipId {
        self.id
    }

    pub fn vehicle_id(&self) -> VehicleId {
        self.vehicle_id
    }

    pub fn owner_customer_id(&self) -> CustomerId {
        self.owner_customer_id
    }

    pub fn ownership_type(&self) -> &OwnershipType {
        &self.ownership_type
    }

    pub fn status(&self) -> &OwnershipStatus {
        &self.status
    }

    pub fn period(&self) -> &OwnershipPeriod {
        &self.period
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

    fn raise(&mut self, event: VehicleOwnershipEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod aggregate_tests;
