//! The `VehicleOwnership` aggregate — operational vehicle ownership in PitGO.
//!
//! Not a legal record of title; it models eligibility inside the platform. The
//! "at most one open ownership" rule is enforced jointly: a snapshot-based
//! invariant inside the aggregate, plus a partial unique index in the database.
//!
//! Агрегат `VehicleOwnership` — операционное владение автомобилем в PitGO.
//!
//! Не является юридической записью о собственности; моделирует пригодность
//! внутри платформы. Правило «не более одного открытого владения»
//! обеспечивается совместно: инвариант на снимке внутри агрегата и частичный
//! уникальный индекс в базе данных.

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

/// Aggregate root of an ownership record for one specific vehicle.
///
/// `CustomerId` and `VehicleId` are references only — neither the `Customer`
/// nor the `Vehicle` object is loaded. Holding ids rather than objects is what
/// keeps this aggregate's transaction boundary independent: an ownership can be
/// started, verified and ended without ever locking the customer or the vehicle.
///
/// # State machine
///
/// ```text
/// start ─→ PendingVerification ─(verify)─→ Active ─(end)─→ Ended
/// ```
///
/// `Ended` is terminal. `PendingVerification` and `Active` are both considered
/// *open*, and an open record occupies the vehicle — see
/// [`OwnershipStatus::is_open`].
///
/// # Invariants
///
/// - Fields are private; state changes only through named command methods.
/// - At most one open ownership record may exist per vehicle.
/// - The period's `ended_at` is never earlier than its `started_at`.
/// - The version increases by exactly one per raised event.
///
/// # Note on `Clone`
///
/// `Clone` is a temporary concession to the tests and the in-memory repository.
/// It should be removed once the PostgreSQL repository lands, since that
/// repository rehydrates aggregates from rows rather than cloning live ones.
///
/// Корень агрегата записи о владении конкретным автомобилем.
///
/// `CustomerId` и `VehicleId` — только ссылки: ни объект `Customer`, ни объект
/// `Vehicle` не загружаются. Хранение идентификаторов вместо объектов
/// сохраняет независимость транзакционной границы агрегата: владение можно
/// создать, подтвердить и завершить, ни разу не заблокировав клиента или
/// автомобиль.
///
/// # Машина состояний
///
/// ```text
/// start ─→ PendingVerification ─(verify)─→ Active ─(end)─→ Ended
/// ```
///
/// `Ended` — терминальное состояние. `PendingVerification` и `Active` считаются
/// *открытыми*, и открытая запись занимает автомобиль — см.
/// [`OwnershipStatus::is_open`].
///
/// # Инварианты
///
/// - Поля приватные; состояние изменяется только именованными командными методами.
/// - На один автомобиль может существовать не более одной открытой записи о владении.
/// - Поле `ended_at` периода никогда не раньше поля `started_at`.
/// - Версия увеличивается ровно на единицу на каждое порождённое событие.
///
/// # Замечание о `Clone`
///
/// `Clone` — временная уступка тестам и репозиторию в памяти. Его следует
/// убрать после реализации PostgreSQL-репозитория, который восстанавливает
/// агрегаты из строк таблицы, а не клонирует живые экземпляры.
#[derive(Debug, Clone)]
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
    /// Creates a new ownership record in the `PendingVerification` status.
    ///
    /// Uses `snapshot` to check that the vehicle has no open ownership. The
    /// aggregate cannot query the repository itself — the domain has no I/O —
    /// so the application service reads that fact first and hands it in as
    /// evidence.
    ///
    /// This check alone is not sufficient under concurrency: two callers can
    /// both observe an empty snapshot before either writes. The snapshot
    /// rejects the honest mistake cheaply and with a precise domain error,
    /// while the database's partial unique index remains the authority that
    /// makes the invariant actually hold. Documenting both halves matters —
    /// removing either one silently weakens the rule.
    ///
    /// Note the resulting record starts as `PendingVerification`, which already
    /// counts as *open* and therefore occupies the vehicle. Treating a pending
    /// record as free was the original defect this design closes.
    ///
    /// Создаёт новую запись о владении в статусе `PendingVerification`.
    ///
    /// Использует `snapshot`, чтобы убедиться в отсутствии открытого владения
    /// для данного автомобиля. Агрегат не может сам обратиться к репозиторию —
    /// в домене нет ввода-вывода, — поэтому сервис приложения сначала читает
    /// этот факт и передаёт его как доказательство.
    ///
    /// Одной этой проверки недостаточно при конкурентном доступе: два
    /// вызывающих могут увидеть пустой снимок раньше, чем любой из них
    /// выполнит запись. Снимок дёшево отклоняет добросовестную ошибку с точной
    /// доменной ошибкой, а окончательным гарантом инварианта остаётся
    /// частичный уникальный индекс базы данных. Важно документировать обе
    /// половины: удаление любой из них незаметно ослабляет правило.
    ///
    /// Обратите внимание: созданная запись начинается со статуса
    /// `PendingVerification`, который уже считается *открытым* и, значит,
    /// занимает автомобиль. Трактовка ожидающей записи как свободной и была
    /// исходным дефектом, который закрывает эта модель.
    pub fn start(
        id: VehicleOwnershipId,
        vehicle_id: VehicleId,
        owner_customer_id: CustomerId,
        ownership_type: OwnershipType,
        snapshot: OwnershipEligibilitySnapshot,
        now: DateTime<Utc>,
    ) -> Result<Self, OwnershipError> {
        if !snapshot.no_open_ownership_exists() {
            return Err(OwnershipError::ActiveOwnershipAlreadyExists);
        }

        let mut ownership = Self {
            id,
            vehicle_id,
            owner_customer_id,
            // Cloned because the same value is also moved into the `Started`
            // event below; the event owns its payload so that it stays a
            // self-contained record of what happened.
            //
            // Клонируется, поскольку то же значение перемещается в событие
            // `Started` ниже; событие владеет своей полезной нагрузкой, чтобы
            // оставаться самодостаточной записью о произошедшем.
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

    /// Confirms the ownership: `PendingVerification → Active`.
    ///
    /// Idempotent: returns `NoChange` if already `Active`, so a redelivered
    /// verification command is safe. Any other status is an error — in
    /// particular a terminated (`Ended`) record can never be revived, because
    /// doing so would resurrect a claim on a vehicle that may since have been
    /// legitimately transferred to someone else.
    ///
    /// Подтверждает владение: `PendingVerification → Active`.
    ///
    /// Идемпотентен: возвращает `NoChange`, если статус уже `Active`, поэтому
    /// повторно доставленная команда подтверждения безопасна. Любой другой
    /// статус приводит к ошибке — в частности, завершённую (`Ended`) запись
    /// нельзя оживить, так как это восстановило бы притязание на автомобиль,
    /// который к тому моменту мог быть законно передан другому владельцу.
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

    /// Ends the ownership: `Active → Ended` (terminal).
    ///
    /// Closing the period is fallible and is attempted *before* the status is
    /// changed: if `now` precedes `started_at` the whole command fails with
    /// [`OwnershipError::PeriodEndBeforeStart`] and the aggregate is left
    /// untouched. Ordering the mutations this way is what keeps a rejected
    /// command from leaving the aggregate half-modified.
    ///
    /// Idempotent: returns `NoChange` if already `Ended`. Returns an error for
    /// `PendingVerification` — an unverified claim is withdrawn rather than
    /// ended, and conflating the two would record ownership history that never
    /// actually took effect.
    ///
    /// Завершает владение: `Active → Ended` (терминальное состояние).
    ///
    /// Закрытие периода может завершиться неудачей и выполняется *до* смены
    /// статуса: если `now` раньше `started_at`, вся команда завершается
    /// ошибкой [`OwnershipError::PeriodEndBeforeStart`], а агрегат остаётся
    /// нетронутым. Именно такой порядок изменений не позволяет отклонённой
    /// команде оставить агрегат в наполовину изменённом виде.
    ///
    /// Идемпотентен: возвращает `NoChange`, если статус уже `Ended`. Для
    /// `PendingVerification` возвращает ошибку — неподтверждённое притязание
    /// отзывается, а не завершается, и смешение этих случаев записало бы
    /// историю владения, которое фактически не вступало в силу.
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

    /// Returns the buffered domain events and clears the internal buffer.
    ///
    /// Called by the persistence layer once the aggregate has been saved.
    /// Draining is intentional: events must be published exactly once.
    ///
    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    ///
    /// Вызывается слоем персистентности после сохранения агрегата. Опустошение
    /// буфера намеренно: события должны публиковаться ровно один раз.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<VehicleOwnershipEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Getters ───────────────────────────────────────────────────────────
    // ── Геттеры ───────────────────────────────────────────────────────────

    pub fn id(&self) -> VehicleOwnershipId {
        self.id
    }

    /// Vehicle this record refers to. Used by repositories to test the
    /// "one open ownership per vehicle" rule.
    ///
    /// Автомобиль, к которому относится запись. Используется репозиториями для
    /// проверки правила «одно открытое владение на автомобиль».
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
    fn raise(&mut self, event: VehicleOwnershipEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "aggregate_tests.rs"]
mod aggregate_tests;
