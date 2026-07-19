//! The `Customer` aggregate — the customer lifecycle boundary in PitGO.
//!
//! Агрегат `Customer` — lifecycle-граница клиента в PitGO.

use chrono::{DateTime, Utc};

use shared::aggregate::{AggregateVersion, ChangeOutcome};
use shared::event::PendingEvent;

use crate::customer::error::CustomerError;
use crate::customer::event::{CustomerActivatedV1, CustomerCreatedV1, CustomerEvent};
use crate::customer::permit::ActivationPermit;
use crate::customer::state::CustomerStatus;
use crate::ids::CustomerId;

/// Customer lifecycle aggregate root.
///
/// Holds lifecycle state only. Contacts, profile, preferences and consent are
/// separate aggregates (`CustomerContactBook`, `CustomerProfile`, …), which
/// keeps this aggregate's transaction boundary — and therefore its contention
/// window — as small as the lifecycle itself.
///
/// # Invariants
///
/// - Fields are private; state changes only through named command methods.
/// - The version increases by exactly one per raised event, never otherwise.
/// - A customer always exists in exactly one [`CustomerStatus`].
///
/// # Note on `Clone`
///
/// `Clone` is a temporary concession to the tests and the in-memory repository,
/// which store aggregates by value. A real Postgres repository rehydrates each
/// aggregate from its row instead, so this derive should be removed once that
/// repository lands — cloning an aggregate risks two live copies drifting apart
/// in version.
///
/// Корень агрегата жизненного цикла клиента.
///
/// Хранит только lifecycle-состояние. Контакты, профиль, предпочтения и
/// согласия вынесены в отдельные агрегаты (`CustomerContactBook`,
/// `CustomerProfile`, …), что удерживает транзакционную границу агрегата — а
/// значит, и окно конкуренции — не шире самого жизненного цикла.
///
/// # Инварианты
///
/// - Поля приватные; состояние изменяется только именованными командными методами.
/// - Версия увеличивается ровно на единицу на каждое порождённое событие, и никак иначе.
/// - Клиент всегда находится ровно в одном статусе [`CustomerStatus`].
///
/// # Замечание о `Clone`
///
/// `Clone` — временная уступка тестам и репозиторию в памяти, которые хранят
/// агрегаты по значению. Настоящий Postgres-репозиторий вместо этого
/// восстанавливает агрегат из строки таблицы, поэтому данный derive следует
/// убрать после его реализации: клонирование агрегата создаёт риск расхождения
/// версий двух живых копий.
#[derive(Debug, Clone)]
pub struct Customer {
    id: CustomerId,
    status: CustomerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingEvent<CustomerEvent>>,
}

impl Customer {
    /// Creates a new customer in the `Draft` status.
    ///
    /// The only public constructor, which is what guarantees that no customer
    /// can come into existence without a `CustomerCreated` event describing it.
    /// The aggregate version ends at 1, not 0: creation is itself a recorded
    /// event.
    ///
    /// `now` is passed in rather than read from the system clock so that the
    /// domain stays free of ambient I/O and tests can pin time exactly.
    ///
    /// Создаёт нового клиента в статусе `Draft`.
    ///
    /// Единственный публичный конструктор — именно это гарантирует, что клиент
    /// не может возникнуть без описывающего его события `CustomerCreated`.
    /// Версия агрегата в итоге равна 1, а не 0: создание само по себе является
    /// записанным событием.
    ///
    /// `now` передаётся снаружи, а не читается из системных часов, чтобы домен
    /// оставался свободным от неявного ввода-вывода, а тесты могли точно
    /// зафиксировать время.
    pub fn create(id: CustomerId, now: DateTime<Utc>) -> Self {
        let mut customer = Self {
            id,
            status: CustomerStatus::Draft,
            created_at: now,
            updated_at: now,
            version: AggregateVersion::INITIAL,
            pending_events: Vec::new(),
        };
        customer.raise(
            CustomerEvent::Created(CustomerCreatedV1 { customer_id: id }),
            now,
        );
        customer
    }

    /// Moves the customer from `Draft` to `Active`.
    ///
    /// The aggregate checks only the permit's *local* conditions — matching id,
    /// matching version and expiry. Global eligibility (verified contact,
    /// recorded consent) was evaluated by `CustomerActivationPolicy` before the
    /// permit was issued, and is re-asserted by `UnitOfWork` at commit time.
    /// This split is what lets the aggregate stay ignorant of other aggregates
    /// while still refusing to activate on stale evidence: the version check
    /// makes a permit worthless the moment the customer changes underneath it.
    ///
    /// Returns `NoChange` without touching state if the customer is already
    /// `Active`, so a retried activation is safe. Any other status is an error.
    ///
    /// Переводит клиента из `Draft` в `Active`.
    ///
    /// Агрегат проверяет только *локальные* условия permit — совпадение
    /// идентификатора, совпадение версии и срок действия. Глобальная
    /// пригодность (подтверждённый контакт, зафиксированное согласие) была
    /// проверена `CustomerActivationPolicy` до выдачи permit и повторно
    /// утверждается `UnitOfWork` при commit. Такое разделение позволяет
    /// агрегату не знать о других агрегатах и при этом отказывать в активации
    /// по устаревшим данным: проверка версии обесценивает permit в тот момент,
    /// когда клиент изменяется «под ним».
    ///
    /// Возвращает `NoChange`, не изменяя состояние, если клиент уже `Active` —
    /// поэтому повторная активация безопасна. Любой другой статус приводит к
    /// ошибке.
    pub fn activate(
        &mut self,
        permit: ActivationPermit,
        now: DateTime<Utc>,
    ) -> Result<ChangeOutcome, CustomerError> {
        permit.validate_local(self.id, self.version, now)?;

        match &self.status {
            CustomerStatus::Draft => {
                self.status = CustomerStatus::Active;
                self.updated_at = now;
                self.raise(
                    CustomerEvent::Activated(CustomerActivatedV1 {
                        customer_id: self.id,
                    }),
                    now,
                );
                Ok(ChangeOutcome::Changed)
            }
            CustomerStatus::Active => Ok(ChangeOutcome::NoChange),
        }
    }

    /// Returns the buffered domain events and clears the internal buffer.
    ///
    /// Called by the persistence layer once the aggregate has been saved
    /// successfully. Draining is intentional: events must be published exactly
    /// once, so a second call after a successful save yields nothing.
    ///
    /// Возвращает накопленные доменные события и очищает внутренний буфер.
    ///
    /// Вызывается слоем персистентности после успешного сохранения агрегата.
    /// Опустошение буфера намеренно: события должны публиковаться ровно один
    /// раз, поэтому повторный вызов после успешного сохранения вернёт пустой
    /// список.
    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<CustomerEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Getters (for tests and repository rehydration) ────────────────────
    // ── Геттеры (для тестов и восстановления в репозитории) ───────────────

    pub fn id(&self) -> CustomerId {
        self.id
    }

    pub fn status(&self) -> &CustomerStatus {
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
    /// Routing every state change through this single method is what keeps the
    /// "one event, one version increment" invariant true by construction: a
    /// command cannot record history without also advancing the version used
    /// for concurrency control.
    ///
    /// Записывает событие в буфер и увеличивает версию агрегата.
    ///
    /// Проведение каждого изменения состояния через этот единственный метод
    /// делает инвариант «одно событие — одно увеличение версии» истинным по
    /// построению: команда не может записать историю, не сдвинув при этом
    /// версию, используемую для контроля конкурентного доступа.
    fn raise(&mut self, event: CustomerEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.version = self.version.next();
    }
}

#[cfg(test)]
#[path = "tests/aggregate_tests.rs"]
mod aggregate_tests;
