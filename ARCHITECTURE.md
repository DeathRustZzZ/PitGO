# PitGO — Production-Grade Architecture

## 1. Bounded Contexts

```
┌─────────────────────────────────────────────────────────────────┐
│                        PitGO Platform                           │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │   Customer   │  │   Contact    │  │  Identity Registry   │   │
│  │      BC      │  │      BC      │  │        BC            │   │
│  └──────┬───────┘  └──────┬───────┘  └──────────────────────┘   │
│         │                 │                    ▲                │
│         └─────────────────┘                    │                │
│                    │                  (uniqueness reservation)  │
│         ┌──────────▼──────────┐               │                 │
│         │      Vehicle         │───────────────┘                │
│         │         BC           │                                │
│         └──────────┬──────────┘                                 │
│                    │                                            │
│         ┌──────────▼──────────┐                                 │
│         │  VehicleOwnership   │                                 │
│         │         BC           │                                │
│         └─────────────────────┘                                 │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Verification │  │     Audit    │  │    Notification      │   │
│  │   (shared)   │  │   (shared)   │  │    (future)          │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

Каждый BC — отдельный деплоить-независимый модуль, общающийся только через события или Anti-Corruption Layer.

---

## 2. Workspace / Crates Structure

```
pitgo/
├── Cargo.toml                          # workspace
├── Cargo.lock
│
├── crates/
│   │
│   ├── shared-kernel/                  # типы, разделяемые между BC
│   │   └── pitgo-shared-kernel/
│   │       # EntityId<T>, AggregateRoot trait, DomainEvent trait,
│   │       # Money, Timestamp, TenantId, UserId, Pagination,
│   │       # DomainError, ValidationError, Clock trait
│   │
│   ├── domain/                         # чистый домен, без I/O
│   │   ├── pitgo-customer-domain/
│   │   ├── pitgo-contact-domain/
│   │   ├── pitgo-vehicle-domain/
│   │   ├── pitgo-ownership-domain/
│   │   └── pitgo-identity-domain/
│   │
│   ├── application/                    # use cases, команды, запросы
│   │   ├── pitgo-customer-app/
│   │   ├── pitgo-contact-app/
│   │   ├── pitgo-vehicle-app/
│   │   └── pitgo-ownership-app/
│   │
│   ├── infrastructure/                 # I/O реализации
│   │   ├── pitgo-postgres/             # sqlx, миграции, репозитории
│   │   ├── pitgo-eventbus/             # outbox + Kafka/NATS
│   │   ├── pitgo-identity-registry/    # Redis + Postgres
│   │   ├── pitgo-external-providers/   # адаптеры к ГИБДД, VIN API и т.д.
│   │   └── pitgo-cache/               # Redis abstractions
│   │
│   └── api/
│       ├── pitgo-http-api/             # axum, REST + OpenAPI
│       └── pitgo-grpc-api/             # tonic (future)
│
└── services/                           # бинари
    ├── customer-service/
    ├── vehicle-service/
    └── ownership-service/
```

### Правила зависимостей crates

```
api → application → domain → shared-kernel
infrastructure → application (реализует трейты из domain)
services → api + infrastructure (composition root)
```

Domain никогда не зависит от infrastructure. Зависимости направлены внутрь.

---

## 3. Domain / Application / Infrastructure Layers

### Domain Layer

```rust
// pitgo-customer-domain/src/aggregate.rs

pub struct Customer {
    id: CustomerId,
    tenant_id: TenantId,
    status: CustomerStatus,
    profile: CustomerProfile,
    version: AggregateVersion,         // optimistic lock + ES-ready
    pending_events: Vec<CustomerEvent>, // collect-and-dispatch pattern
}

impl Customer {
    // factory — единственный способ создания
    pub fn register(
        id: CustomerId,
        tenant_id: TenantId,
        profile: CustomerProfile,
        clock: &dyn Clock,
    ) -> Result<Self, CustomerError> {
        // валидация инвариантов
        profile.validate()?;
        let mut customer = Self { ... };
        customer.record(CustomerEvent::Registered { ... });
        Ok(customer)
    }

    pub fn activate(&mut self, clock: &dyn Clock) -> Result<(), CustomerError> {
        match self.status {
            CustomerStatus::Draft => {
                self.status = CustomerStatus::Active;
                self.record(CustomerEvent::Activated { at: clock.now() });
                Ok(())
            }
            s => Err(CustomerError::InvalidTransition { from: s, to: CustomerStatus::Active }),
        }
    }

    // state machine exhaustive via match — компилятор гарантирует покрытие
    fn validate_transition(&self, to: CustomerStatus) -> Result<(), CustomerError> { ... }

    fn record(&mut self, event: CustomerEvent) {
        self.pending_events.push(event);
        self.version.increment();
    }

    pub fn take_events(&mut self) -> Vec<CustomerEvent> {
        std::mem::take(&mut self.pending_events)
    }
}
```

```rust
// Value Object — без ID, сравнение по значению, иммутабельный
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomerProfile {
    pub name: PersonName,
    pub birth_date: Option<BirthDate>,
}

// Newtype pattern для строгой типизации
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CustomerId(Uuid);

impl CustomerId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}
// Компилятор не позволит перепутать CustomerId и VehicleId
```

```rust
// Domain Error — явные, типизированные
#[derive(Debug, thiserror::Error)]
pub enum CustomerError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: CustomerStatus, to: CustomerStatus },
    #[error("Customer is already deleted")]
    AlreadyDeleted,
    #[error(transparent)]
    Validation(#[from] ValidationError),
}
```

```rust
// Repository trait — только в domain, реализация в infrastructure
#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn find_by_id(&self, id: &CustomerId) -> Result<Option<Customer>, RepositoryError>;
    async fn save(&self, customer: &mut Customer) -> Result<(), RepositoryError>;
    async fn exists(&self, id: &CustomerId) -> Result<bool, RepositoryError>;
}
```

### Application Layer

```rust
// Command — простой DTO без логики
pub struct ActivateCustomerCommand {
    pub customer_id: CustomerId,
    pub tenant_id: TenantId,
    pub requested_by: UserId,
}

// Command Handler — orchestration only
pub struct ActivateCustomerHandler {
    customer_repo: Arc<dyn CustomerRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ActivateCustomerHandler {
    pub async fn handle(&self, cmd: ActivateCustomerCommand) -> Result<(), AppError> {
        let mut customer = self.customer_repo
            .find_by_id(&cmd.customer_id)
            .await?
            .ok_or(AppError::NotFound(cmd.customer_id.to_string()))?;

        // авторизация на уровне application
        self.authorize(&customer, &cmd.requested_by)?;

        // бизнес-логика — только в агрегате
        customer.activate(&SystemClock)?;

        // save + publish в одной транзакции через Unit of Work
        self.customer_repo.save(&mut customer).await?;

        // события публикуются после commit через outbox
        let events = customer.take_events();
        self.event_publisher.publish_all(events).await?;

        Ok(())
    }
}
```

---

## 4. CQRS Organization

```
Write Side                          Read Side
──────────────────────────────      ─────────────────────────────────
Command → CommandHandler            Query → QueryHandler
    → Aggregate (business logic)        → ReadModel (denormalized)
    → Repository (save)                 → direct sqlx query
    → Outbox (events)                   → cache (Redis)

Write DB: PostgreSQL                Read DB: PostgreSQL read replica
(normalized, aggregate-per-table)   (views, materialized, projections)
```

```rust
// Query side — никакого агрегата, прямой SQL
pub struct GetCustomerSummaryHandler {
    db: PgPool,
}

impl GetCustomerSummaryHandler {
    pub async fn handle(&self, id: CustomerId) -> Result<CustomerSummaryView, AppError> {
        sqlx::query_as!(
            CustomerSummaryView,
            r#"
            SELECT c.id, c.status, c.name, ccb.primary_phone
            FROM customers c
            LEFT JOIN customer_contact_books ccb ON ccb.customer_id = c.id
            WHERE c.id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotFound(id.to_string()))
    }
}
```

---

## 5. Event Bus & Integration Events

### Outbox Pattern (надёжная доставка)

```
┌───────────────────────────────────┐
│          PostgreSQL               │
│                                   │
│  ┌──────────┐    ┌─────────────┐  │
│  │ aggregate│    │   outbox    │  │
│  │  table   │    │   table     │  │
│  └──────────┘    └──────┬──────┘  │
│       └──── same TX ────┘         │
└───────────────────────────────────┘
          ▼ polling / CDC
    ┌─────────────┐
    │  Kafka /    │
    │   NATS JB   │
    └─────────────┘
          ▼
  другие BC / сервисы
```

```sql
CREATE TABLE outbox_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id   UUID NOT NULL,
    event_type     TEXT NOT NULL,
    payload        JSONB NOT NULL,
    metadata       JSONB NOT NULL DEFAULT '{}',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ,
    sequence_no    BIGSERIAL
);

-- Только непубликованные события для polling
CREATE INDEX idx_outbox_unpublished ON outbox_events (created_at)
WHERE published_at IS NULL;
```

### Domain Event vs Integration Event

```rust
// Domain Event — внутри BC, богатый тип
pub enum CustomerEvent {
    Registered { id: CustomerId, profile: CustomerProfile, at: Timestamp },
    Activated   { at: Timestamp },
    Suspended   { reason: SuspendReason, at: Timestamp },
}

// Integration Event — пересекает BC границу, стабильный контракт
// versioned, backward-compatible
#[derive(Serialize, Deserialize)]
pub struct CustomerActivatedV1 {
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub schema_version: u8, // = 1
}
```

Mapping Domain → Integration событий — в Application layer.

---

## 6. Repository Abstractions

```rust
// Generic trait в shared-kernel
#[async_trait]
pub trait Repository<A: AggregateRoot>: Send + Sync {
    async fn find_by_id(&self, id: &A::Id) -> Result<Option<A>, RepositoryError>;
    async fn save(&self, aggregate: &mut A) -> Result<(), RepositoryError>;
}

// Специфичные расширения в domain
#[async_trait]
pub trait VehicleRepository: Repository<Vehicle> {
    async fn find_by_vin(&self, vin: &Vin) -> Result<Option<Vehicle>, RepositoryError>;
    async fn find_by_plate(&self, plate: &LicensePlate) -> Result<Option<Vehicle>, RepositoryError>;
}

// Реализация в infrastructure
pub struct PgVehicleRepository {
    pool: PgPool,
}

#[async_trait]
impl VehicleRepository for PgVehicleRepository {
    async fn find_by_vin(&self, vin: &Vin) -> Result<Option<Vehicle>, RepositoryError> {
        let row = sqlx::query!(
            "SELECT * FROM vehicles WHERE vin = $1 AND deleted_at IS NULL",
            vin.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        row.map(VehicleMapper::from_row).transpose()
    }
}
```

---

## 7. Transaction Boundaries

**Правило:** одна транзакция = один агрегат.

```rust
// Unit of Work паттерн
pub struct PgUnitOfWork {
    tx: PgTransaction<'static>,
}

impl PgUnitOfWork {
    pub async fn begin(pool: &PgPool) -> Result<Self, DbError> {
        Ok(Self { tx: pool.begin().await? })
    }

    pub async fn commit(self) -> Result<(), DbError> {
        self.tx.commit().await?;
        Ok(())
    }
}

// Handler использует UoW
pub async fn handle(&self, cmd: ActivateCustomerCommand) -> Result<(), AppError> {
    let uow = PgUnitOfWork::begin(&self.pool).await?;

    let mut customer = self.customer_repo.find_by_id_tx(&uow, &cmd.customer_id).await?;
    customer.activate(&SystemClock)?;

    // агрегат + outbox записываются в одну транзакцию
    self.customer_repo.save_tx(&uow, &mut customer).await?;
    self.outbox_repo.save_tx(&uow, customer.take_events()).await?;

    uow.commit().await?;
    // EventRelay прочитает outbox асинхронно и опубликует в Kafka
    Ok(())
}
```

Для cross-aggregate операций — Saga/Process Manager через события, не распределённые транзакции.

---

## 8. Cross-Aggregate Consistency

```
Создание VehicleOwnership (Saga):

1. CreateOwnershipCommand
        ↓
2. OwnershipSaga начинает process
        ↓
3. CustomerExistsQuery → Customer BC         (sync, anti-corruption)
   VehicleExistsQuery  → Vehicle BC          (sync)
        ↓
4. Создать VehicleOwnership (PendingVerification)
        ↓
5. Publish: OwnershipPendingVerificationEvent
        ↓
6. Verification BC слушает событие → запускает верификацию
        ↓
7a. VerificationApproved → OwnershipSaga → активировать ownership
7b. VerificationRejected → OwnershipSaga → отклонить ownership
```

```rust
// Process Manager / Saga state (хранится в БД)
pub struct OwnershipCreationSaga {
    id: SagaId,
    state: SagaState,
    customer_id: CustomerId,
    vehicle_id: VehicleId,
    started_at: Timestamp,
    timeout_at: Timestamp,
}

pub enum SagaState {
    ValidatingParties,
    AwaitingVerification { ownership_id: OwnershipId },
    Completed,
    Failed { reason: String },
    TimedOut,
}
```

---

## 9. Identity Registry

Цель: глобальная уникальность контактов (телефон/email) поперёк всех tenant-ов (или в рамках одного — зависит от бизнес-правила).

```
┌──────────────────────────────────────────────────────────┐
│                   Identity Registry BC                    │
│                                                          │
│  ReserveIdentityCommand(phone/email, owner_id)           │
│       → проверить uniqueness                             │
│       → записать reservation                             │
│       → вернуть IdentityToken                            │
│                                                          │
│  ConfirmIdentityCommand(token)                           │
│       → перевести reservation → confirmed                │
│                                                          │
│  ReleaseIdentityCommand(token)                           │
│       → освободить при отмене                            │
└──────────────────────────────────────────────────────────┘
```

```sql
CREATE TABLE identity_reservations (
    id          UUID PRIMARY KEY,
    kind        TEXT NOT NULL,           -- 'phone' | 'email' | 'telegram'
    value_hash  TEXT NOT NULL,           -- bcrypt/sha256 для приватности
    owner_id    UUID NOT NULL,
    status      TEXT NOT NULL,           -- 'reserved' | 'confirmed' | 'released'
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (kind, value_hash)            -- DB-level uniqueness guarantee
);
```

Хранить хэш контакта, не plaintext — чтобы не раскрывать данные одного клиента другому.

---

## 10. Vehicle Ownership Workflows

```
                    ┌──────────────────────┐
                    │  PendingVerification  │
                    └──────────┬───────────┘
                               │
             ┌─────────────────┴─────────────────┐
             ▼                                   ▼
        ┌──────────┐                       ┌──────────┐
        │  Active  │                       │ Rejected │
        └────┬─────┘                       └──────────┘
             │
    ┌─────────┴─────────┐
    ▼                   ▼
┌────────┐         ┌──────────┐
│ Ended  │         │ Disputed │
└────────┘         └────┬─────┘
                        │
                  ┌─────┴──────┐
                  ▼            ▼
              Active        Rejected
```

```rust
pub struct VehicleOwnership {
    id: OwnershipId,
    customer_id: CustomerId,
    vehicle_id: VehicleId,
    ownership_type: OwnershipType,
    period: OwnershipPeriod,           // started_at + ended_at: Option
    status: OwnershipStatus,
    verification: OwnershipVerification,
    version: AggregateVersion,
    pending_events: Vec<OwnershipEvent>,
}

pub struct OwnershipPeriod {
    pub started_at: Date,
    pub ended_at: Option<Date>,
}

impl OwnershipPeriod {
    pub fn is_active(&self, on: Date) -> bool {
        on >= self.started_at && self.ended_at.map_or(true, |e| on <= e)
    }
}

pub struct OwnershipVerification {
    pub source: VerificationSource,
    pub trust_level: TrustLevel,       // Unverified | SelfDeclared | ThirdParty | Official
    pub verified_at: Option<Timestamp>,
    pub document_ref: Option<DocumentRef>,
}
```

---

## 11. Authorization Model

Двухуровневая авторизация: **Tenant isolation** + **RBAC**.

```rust
// Claims из JWT
pub struct AuthContext {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
}

pub enum Role {
    PlatformAdmin,
    TenantAdmin,
    ServiceManager,
    Customer,
}

pub enum Permission {
    CustomerRead,
    CustomerWrite,
    VehicleRead,
    VehicleWrite,
    OwnershipManage,
    VerificationApprove,
}
```

```rust
// Authorization Guard в application layer
pub fn authorize_customer_write(
    ctx: &AuthContext,
    customer: &Customer,
) -> Result<(), AuthError> {
    // tenant isolation — жёсткое правило
    if ctx.tenant_id != customer.tenant_id() {
        return Err(AuthError::TenantMismatch);
    }
    // permission check
    if !ctx.permissions.contains(&Permission::CustomerWrite) {
        return Err(AuthError::InsufficientPermissions);
    }
    Ok(())
}
```

Будущее: перейти на ABAC (attribute-based) через policy engine (OPA или собственный), когда появятся сложные правила вида "менеджер видит только своих клиентов".

---

## 12. Audit Model

Domain Events — и есть audit trail. Не нужна отдельная таблица логов.

```sql
CREATE TABLE audit_log (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id      UUID NOT NULL,
    event_id       UUID NOT NULL UNIQUE,    -- идемпотентность
    aggregate_type TEXT NOT NULL,
    aggregate_id   UUID NOT NULL,
    event_type     TEXT NOT NULL,
    actor_id       UUID,                    -- кто инициировал
    actor_type     TEXT,                    -- 'user' | 'system' | 'external'
    payload        JSONB NOT NULL,
    metadata       JSONB NOT NULL,          -- ip, user_agent, request_id
    occurred_at    TIMESTAMPTZ NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_aggregate ON audit_log (aggregate_type, aggregate_id, occurred_at);
CREATE INDEX idx_audit_actor     ON audit_log (actor_id, occurred_at);
```

Audit Log Projector подписывается на события из Kafka и записывает в audit_log. Это append-only — никаких UPDATE/DELETE.

---

## 13. Aggregate Versioning

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AggregateVersion(u64);

impl AggregateVersion {
    pub fn initial() -> Self { Self(0) }
    pub fn increment(&mut self) { self.0 += 1; }
    pub fn value(&self) -> u64 { self.0 }
}
```

```sql
-- Optimistic locking в репозитории
UPDATE customers
SET
    status     = $1,
    profile    = $2,
    version    = version + 1,
    updated_at = NOW()
WHERE id = $3
  AND version = $4    -- ← expected version
RETURNING version;

-- Если 0 rows affected → конфликт → ConcurrencyError → retry
```

Версия инкрементируется при каждом `record()` — что делает её эквивалентом sequence number для Event Sourcing.

---

## 14. Migration Strategy

### Инструмент: `sqlx migrate`

```
pitgo-postgres/
└── migrations/
    ├── 20240101000000_create_customers.sql
    ├── 20240101000001_create_vehicles.sql
    ├── 20240101000002_create_outbox.sql
    └── ...
```

### Принципы

1. **Backward-compatible changes only** во время rolling deploy:
   - добавлять колонки с DEFAULT — ✓
   - удалять колонки — только после удаления кода, который их читает
   - переименовывать — через expand/contract (добавить новую, скопировать данные, удалить старую)

2. **Expand/Contract Pattern** для breaking changes:
   ```
   Phase 1 (Expand):  добавить новое поле, код пишет в оба
   Phase 2 (Migrate): бэкфилл данных
   Phase 3 (Contract): код читает только из нового, удалить старое
   ```

3. **Schema versioning** — `aggregate_schema_version` в таблице агрегата для отслеживания формата payload.

4. **Тестирование миграций** в CI: применить на пустую БД, откатить (если DOWN migration есть), применить снова.

### Domain Evolution

```rust
// Версионирование Integration Events
#[derive(Deserialize)]
#[serde(tag = "schema_version")]
pub enum CustomerActivatedEvent {
    #[serde(rename = "1")]
    V1(CustomerActivatedV1),
    #[serde(rename = "2")]
    V2(CustomerActivatedV2), // добавлено поле activation_source
}

// Consumer умеет читать старые версии
impl From<CustomerActivatedV1> for CustomerActivated {
    fn from(v: CustomerActivatedV1) -> Self {
        Self {
            customer_id: v.customer_id,
            occurred_at: v.occurred_at,
            activation_source: ActivationSource::Unknown, // дефолт для старых событий
        }
    }
}
```

---

## 15. Event Sourcing Readiness

Архитектура проектируется ES-ready без полного внедрения ES сейчас.

### Что уже готово

| Элемент | Статус |
|---|---|
| `pending_events: Vec<DomainEvent>` в агрегате | ✓ collect-and-dispatch |
| `AggregateVersion` — sequence number | ✓ |
| Событие содержит все данные для реконструкции | ✓ rich events |
| Audit Log = append-only event store | ✓ |
| Outbox = ordered event stream | ✓ |

### Путь к полному Event Sourcing

```rust
// Когда понадобится ES, агрегат получит:
impl Customer {
    // Реконструкция из событий
    pub fn from_events(events: Vec<CustomerEvent>) -> Result<Self, DomainError> {
        let mut customer = Self::empty();
        for event in events {
            customer.apply(&event);   // apply уже есть — это то, что сейчас делает мутация
        }
        Ok(customer)
    }

    // apply — чистая функция, меняет состояние без side effects
    fn apply(&mut self, event: &CustomerEvent) {
        match event {
            CustomerEvent::Registered { profile, .. } => {
                self.profile = profile.clone();
                self.status  = CustomerStatus::Draft;
            }
            CustomerEvent::Activated { .. } => {
                self.status = CustomerStatus::Active;
            }
            // ...
        }
        self.version.increment();
    }
}
```

### Event Store Schema (подготовлена, не активна)

```sql
CREATE TABLE event_store (
    global_sequence BIGSERIAL PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    aggregate_type  TEXT NOT NULL,
    aggregate_id    UUID NOT NULL,
    aggregate_version BIGINT NOT NULL,
    event_type      TEXT NOT NULL,
    event_version   SMALLINT NOT NULL DEFAULT 1,
    payload         JSONB NOT NULL,
    metadata        JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (aggregate_id, aggregate_version)   -- optimistic lock
);

CREATE INDEX idx_es_stream ON event_store (aggregate_id, aggregate_version);
CREATE INDEX idx_es_global ON event_store (global_sequence);
```

Когда переключитесь на ES: репозитории читают из `event_store` и реконструируют агрегат через `from_events()`. Read side — без изменений, она уже читает из проекций.

---

---

## Vehicle Domain

### Обзор

Vehicle BC отвечает за физический автомобиль как самостоятельную доменную сущность и за факт его операционного владения в рамках PitGO. BC строится по тем же архитектурным принципам, что и Customer Domain: несколько компактных агрегатов с независимыми lifecycle, typed domain events с полным payload, capability-объекты для TOCTOU-рисков, cross-aggregate правила в application layer.

**Важное терминологическое решение:** `CarId` переименовывается в `VehicleId` до начала реализации любых агрегатов. Ubiquitous Language должен быть консистентен во всём codebase.

---

### Агрегат Vehicle

**Зачем существует:** описывает конкретный физический автомобиль в системе — его идентичность, технические характеристики, текущий статус верификации и жизненный цикл. Существует независимо от того, известен ли его владелец.

**Состояния:** Draft → Active → Archived / Disputed. Deleted — терминальное состояние из любого.

**Внутренняя структура — три Value Object:**

`VehicleIdentity` — набор идентификаторов: `vin: Option<Vin>`, `license_plate: Option<LicensePlate>`, `registration_document: Option<RegistrationDocument>`, `external_refs: Vec<ExternalVehicleReference>`. Все поля опциональны: Vehicle может существовать в Draft без VIN. VIN — единственный глобально уникальный и неизменяемый идентификатор; госномер принадлежит регистрации, а не автомобилю.

`VehicleSpecs` — технические характеристики: brand (обязательно), model (обязательно), generation, manufacture_year, body_type, engine, transmission, drivetrain, fuel_type. Описывает текущее известное состояние характеристик, не каталог.

`VerificationInfo` — текущий результат верификации: `source: VerificationSource`, `confidence: ConfidenceLevel`, `verified_at: DateTime<Utc>`, `external_ref: Option<ExternalVerificationRef>`. Хранит только текущее состояние; история восстанавливается из событий.

**Инварианты:**

- VehicleId иммутабелен после создания
- VIN после присвоения не меняется напрямую — только через `correct_vin` с явной причиной
- Active Vehicle обязан иметь reliable identity set (см. VehicleActivationPolicy)
- Brand и model — обязательные поля specs
- Deleted Vehicle не принимает никаких команд
- External references уникальны по provider внутри одного Vehicle
- No-op команды возвращают `ChangeOutcome::NoChange` без события и без инкремента версии

**Команды:**

| Команда | Precondition | Результат |
|---|---|---|
| `create` | — | Draft |
| `assign_vin(vin, proof, ctx)` | VIN не назначен, proof валиден | VehicleVinAssignedV1 |
| `correct_vin(prev, new, reason, proof, ctx)` | VIN назначен, prev совпадает с текущим | VehicleVinCorrectedV1 |
| `change_license_plate(plate, ctx)` | не Deleted | VehicleLicensePlateChangedV1 |
| `change_registration_document(doc, ctx)` | не Deleted | VehicleRegistrationDocumentChangedV1 |
| `update_specs(specs, ctx)` | не Deleted | VehicleSpecsUpdatedV1 |
| `verify(source, confidence, ext_ref, ctx)` | не Deleted | VehicleVerifiedV1 |
| `link_external_reference(provider, ext_id, ctx)` | не Deleted, provider уникален | VehicleExternalReferenceLinkedV1 |
| `unlink_external_reference(provider, ctx)` | reference существует | VehicleExternalReferenceUnlinkedV1 |
| `activate(permit, ctx)` | Draft, permit валиден | VehicleStatusChangedV1 |
| `archive(reason, ctx)` | Active, нет non-terminal Ownership (enforced by VehicleLifecyclePolicy) | VehicleStatusChangedV1 |
| `mark_disputed(reason, ctx)` | Active | VehicleStatusChangedV1 |
| `resolve_dispute(ctx)` | Disputed | VehicleStatusChangedV1 |
| `delete(reason, ctx)` | не Deleted, нет non-terminal Ownership (enforced by VehicleLifecyclePolicy) | VehicleDeletedV1 |

**Что Vehicle не хранит:** историю владения, историю верификаций, показания одометра, историю обслуживания, документы (фото СТС/ПТС), данные каталога, страховку, телематику.

**Связи только через ID:** не знает о Customer, VehicleOwnership, сервисных записях.

---

### Агрегат VehicleOwnership

**Зачем существует:** фиксирует факт операционного владения — какой Customer признаётся системой PitGO как владелец конкретного Vehicle для выполнения бизнес-операций в рамках платформы.

**Важное уточнение семантики.** `VehicleOwnership` в PitGO — это **не юридическое владение** в государственном смысле. Это доменный факт: "данный Customer имеет право управлять этим Vehicle в контексте PitGO-операций (записи на сервис, история обслуживания, доступ к данным автомобиля)". Юридический собственник по СТС/ПТС, страхователь, fleet operator и назначенный водитель — это отдельные концепции, которые могут не совпадать с VehicleOwnership и моделируются отдельно при необходимости.

**Типы владения:** Private, Company, Leasing, Fleet, Unknown.

**Состояния:** PendingVerification → Active → Ended / Rejected. Disputed — боковое состояние из Active. Ended и Rejected — терминальные.

**Правило одновременности:** у одного Vehicle не может существовать более одного non-terminal Ownership. Terminal = Ended | Rejected. Non-terminal = PendingVerification | Active | Disputed. Disputed блокирует создание нового Ownership до resolution. Это правило обеспечивается `OwnershipConsistencyPolicy` + уникальным partial index в persistence.

**Инварианты:**

- `ended_at >= started_at` для OwnershipPeriod
- Ended и Rejected — терминальные (переходов из них нет)
- Active Ownership обязан иметь `verified_at`
- Ссылки на Customer и Vehicle — только по ID, объекты не загружаются

**Команды:**

| Команда | Precondition | Результат |
|---|---|---|
| `start(data, eligibility_snapshot, ctx)` | snapshot валиден, нет non-terminal ownership | VehicleOwnershipStartedV1 |
| `verify(source, evidence_ref, ctx)` | PendingVerification | VehicleOwnershipVerifiedV1 |
| `reject(reason, ctx)` | PendingVerification | VehicleOwnershipRejectedV1 |
| `mark_disputed(reason, evidence_ref, ctx)` | Active | VehicleOwnershipDisputedV1 |
| `resolve_dispute(outcome, reason, evidence_ref, ctx)` | Disputed, actor имеет permission `vehicle.ownership.resolve_dispute` | VehicleOwnershipDisputeResolvedV1 |
| `end(ended_at, reason, ctx)` | Active | VehicleOwnershipEndedV1 |

**Что VehicleOwnership не хранит:** specs автомобиля, профиль Customer, историю сервиса, документы.

---

### VehicleIdentityRegistry

По аналогии с `IdentityRegistry` из Customer Domain. VIN глобально уникален — два Vehicle не могут разделять один VIN. Без reservation механизма параллельные запросы `assign_vin` для одного VIN порождают DB constraint violation вместо доменной ошибки.

**Операции:** `reserve(identity_key, vehicle_id, ttl)` → `VehicleIdentityReservationProof`, `claim(proof)`, `release(identity_key, vehicle_id)`, `cancel_reservation(reservation_id)`.

**`VehicleIdentityReservationProof`** — capability-объект, требуемый командами `assign_vin` и `correct_vin`. Агрегат принимает proof как precondition и валидирует его локально (vehicle_id, TTL, identity_key). Агрегат никогда не делает exists-запросов напрямую.

**`VehicleIdentityKey`** — нормализованный идентификатор с типом: сейчас поддерживает Vin, потенциально — TrustedExternalRegistryId (per provider). Расширяемость заложена в дизайн с самого начала.

**Scope уникальности по типу идентификатора:**

| Идентификатор | Уникальность | Механизм |
|---|---|---|
| VIN | Глобально в рамках tenant | VehicleIdentityRegistry + reservation proof |
| trusted external registry id | В рамках provider + tenant | VehicleIdentityRegistry + reservation proof |
| license_plate | Не является глобально уникальным | Нет reservation механизма |

**License plate** не является самостоятельным proof уникальности: госномер может переиспользоваться, меняться, быть временным и зависит от страны/региона. Для MVP license plate хранится как информационный атрибут в VehicleIdentity без глобального uniqueness constraint. Участвует в reliable identity только в комбинации с registration_document. `LicensePlateRegistry` отложен в Deferred Concepts.

---

### VehicleActivationPolicy и VehicleActivationPermit

По аналогии с `CustomerActivationPolicy` / `ActivationPermit`.

**`VehicleActivationPolicy`** — Domain Service. Читает snapshots состояния Vehicle, проверяет eligibility, выдаёт `VehicleActivationPermit`. `Vehicle.activate()` принимает только permit — не проверяет другие агрегаты самостоятельно.

**`VehicleActivationPermit`** несёт: `vehicle_id`, `vehicle_version`, `identity_state_snapshot`, `specs_completeness_snapshot`, `issued_at`, `expires_at`. Служит аудиторским свидетельством того, какие условия были проверены в момент активации.

**Reliable identity set** — правило активации, выраженное через Policy. Для перехода Draft→Active Vehicle должен иметь хотя бы один из следующих наборов:

1. **VIN** — достаточен сам по себе. Верификация VIN не является обязательным условием активации.
2. **license\_plate + registration\_document** — только в комбинации. `license_plate` без `registration_document` не считается reliable identity.
3. **trusted external registry reference** — external reference считается reliable только если provider находится в списке trusted providers, настраиваемом на уровне конфигурации платформы.

Это правило живёт исключительно в `VehicleActivationPolicy`. Application Service не содержит разветвлённых if-условий по типам идентификаторов. Если правила расширятся, меняется только Policy — агрегат не затрагивается.

---

### OwnershipEligibilitySnapshot и OwnershipConsistencyPolicy

**`OwnershipConsistencyPolicy`** — cross-aggregate Domain Policy. Проверяет: нет ли у Vehicle non-terminal Ownership. Не является методом агрегата — это знание, которое требует доступа к репозиторию.

**`OwnershipEligibilitySnapshot`** — capability-объект, передаваемый в `VehicleOwnership::start()`. Несёт: `vehicle_id`, `vehicle_status`, `vehicle_version`, `has_conflicting_ownership: bool`, `checked_at`. `VehicleOwnership::start()` локально валидирует snapshot — не загружает Vehicle.

Application Service проверяет eligibility, Policy выдаёт snapshot, `start()` принимает его как precondition. UnitOfWork при commit проверяет версии обоих агрегатов через optimistic locking.

---

### Domain Events — Vehicle

Все события содержат typed payload, достаточный для audit, replay и projections. События без смыслового payload не допускаются.

**Vehicle события:**

| Event | Ключевые поля payload |
|---|---|
| `VehicleCreatedV1` | vehicle_id, identity, specs, occurred_at, actor |
| `VehicleVinAssignedV1` | vehicle_id, vin, occurred_at, actor |
| `VehicleVinCorrectedV1` | vehicle_id, previous_vin, corrected_vin, reason, occurred_at, actor |
| `VehicleLicensePlateChangedV1` | vehicle_id, previous_plate, new_plate, occurred_at, actor |
| `VehicleRegistrationDocumentChangedV1` | vehicle_id, previous_doc, new_doc, occurred_at, actor |
| `VehicleSpecsUpdatedV1` | vehicle_id, previous_specs, new_specs, occurred_at, actor |
| `VehicleVerifiedV1` | vehicle_id, source, confidence, external_ref, occurred_at, actor |
| `VehicleExternalReferenceLinkedV1` | vehicle_id, provider, external_id, occurred_at, actor |
| `VehicleExternalReferenceUnlinkedV1` | vehicle_id, provider, external_id, occurred_at, actor |
| `VehicleStatusChangedV1` | vehicle_id, previous_status, new_status, occurred_at, actor |
| `VehicleDeletedV1` | vehicle_id, reason, occurred_at, actor |

**VehicleOwnership события:**

| Event | Ключевые поля payload |
|---|---|
| `VehicleOwnershipStartedV1` | ownership_id, vehicle_id, customer_id, ownership_type, started_at, actor |
| `VehicleOwnershipVerifiedV1` | ownership_id, vehicle_id, customer_id, source, verified_at, actor |
| `VehicleOwnershipRejectedV1` | ownership_id, vehicle_id, customer_id, reason, occurred_at, actor |
| `VehicleOwnershipDisputedV1` | ownership_id, vehicle_id, customer_id, reason, evidence_ref, occurred_at, actor |
| `VehicleOwnershipDisputeResolvedV1` | ownership_id, vehicle_id, outcome, resolution_evidence, occurred_at, actor |
| `VehicleOwnershipEndedV1` | ownership_id, vehicle_id, customer_id, ended_at, reason, actor |

Все события оборачиваются в `EventEnvelope` на persistence boundary через `EventEnvelopeFactory`. Агрегаты накапливают `PendingVehicleEvent` / `PendingOwnershipEvent` с полями из `ActionContext`. Агрегат не знает об `EventEnvelope`.

**`ChangeOutcome`** применяется так же, как в Customer Domain: `Changed` / `NoChange`. `NoChange` не инкрементирует `AggregateVersion`, не создаёт событий.

---

### Verification не обновляет Specs

`verify()` и `update_specs()` — разные доменные команды. Это разные факты: "данные подтверждены" и "данные обновлены". Если внешний реестр (VIN API, ГИБДД) возвращает и данные, и подтверждение, Application Service выполняет обе команды последовательно в рамках одного транзакционного boundary:

1. `vehicle.update_specs(new_specs, ctx)` → `VehicleSpecsUpdatedV1`
2. `vehicle.verify(source, confidence, ext_ref, ctx)` → `VehicleVerifiedV1`

Внешний реестр транслируется через ACL (`ExternalVehicleRegistryAdapter` в infrastructure). Агрегат не знает о существовании ГИБДД или конкретного VIN API.

---

### Application Services и Transaction Boundaries

**VehicleApplicationService** — операции над одним Vehicle. Оркестрирует: нормализацию VIN через `VinNormalizer`, резервирование через `VehicleIdentityRegistry`, сбор snapshots для `VehicleActivationPolicy`, вызов команды агрегата, сохранение через UnitOfWork с optimistic locking.

**VehicleOwnershipApplicationService** — оркестрирует ownership transitions. Смена владельца для MVP:

1. Загрузить current non-terminal VehicleOwnership
2. Загрузить Vehicle snapshot
3. Вызвать `OwnershipConsistencyPolicy` → `OwnershipEligibilitySnapshot`
4. `current_ownership.end(ended_at, reason, ctx)`
5. `VehicleOwnership::start(new_data, eligibility_snapshot, ctx)`
6. UnitOfWork: сохранить оба агрегата + outbox в одной транзакции

Правило: одна транзакция = один или два агрегата (текущий + новый ownership) при смене владельца. Более широкий scope — через events и eventual consistency.

**VehicleLifecyclePolicy** — cross-aggregate Domain Policy для операций archive и delete. Перед `vehicle.archive()` и `vehicle.delete()` Application Service обязан:

1. Загрузить Vehicle
2. Проверить наличие non-terminal Ownership через `VehicleLifecyclePolicy`
3. Если non-terminal Ownership существует — выполнить `ownership.end(reason, ctx)` до archive/delete
4. Сохранить оба агрегата и outbox в одной транзакции

`Vehicle.archive()` и `Vehicle.delete()` сами не завершают Ownership — агрегаты не изменяют друг друга напрямую. Если Application Service попытается вызвать `vehicle.delete()` при наличии non-terminal Ownership без предварительного завершения, `VehicleLifecyclePolicy` возвращает ошибку. Это precondition на уровне application layer, не инвариант самого агрегата.

**Privileged operations.** Authorization выполняется application layer до вызова domain command. Агрегат получает `ActionContext` для audit — не делает live permission check. Распределение permissions:

| Операция | Требуемый permission | Роли MVP |
|---|---|---|
| `correct_vin` | `vehicle.identity.correct_vin` | PlatformAdmin, VehicleDataModerator |
| `resolve_dispute` (ownership) | `vehicle.ownership.resolve_dispute` | PlatformModerator, PlatformAdmin |

Обычный Customer/Owner не может выполнять `correct_vin`. Customer может только инициировать запрос на исправление, если такой workflow появится в будущем как отдельная концепция.

---

### Deferred Concepts

Следующие концепции явно исключены из MVP и будут рассмотрены при появлении конкретных бизнес-требований:

**`OdometerHistory`** — история показаний одометра. Требует отдельного subdomain с правилами источника данных (водитель, сервис, телематика), монотонности и временных меток. Не хранится в Vehicle.

**`VehicleRegistration`** как полноценный агрегат — история регистрационных действий в государственном реестре, смена собственника по документам, РФ-специфика (гос. реестр ГИБДД). Сейчас RegistrationDocument хранится как Value Object в VehicleIdentity как snapshot известных данных, не как authoritative record.

**`VehicleDocumentLedger`** — хранение и управление документами (фото СТС, ПТС, страхового полиса, талона ТО). Требует file storage, OCR, retention policy, отдельного lifecycle.

**`VehicleOwnershipTransfer`** как отдельный агрегат — multi-step confirmation workflow, когда продавец инициирует передачу, а покупатель подтверждает. Для MVP достаточно атомарной операции end+start через Application Service.

**`VehicleCatalog`** — справочник марок, моделей, поколений. Внешний supporting domain, не часть Vehicle агрегата. VehicleSpecs хранит snapshot известных характеристик конкретного автомобиля, не ссылку на каталог.

**`InsurancePolicy`** — отдельный домен.

**`TelematicsData`** — IoT/телематика, полностью отдельный домен.

**`FleetMembership`** — принадлежность Vehicle к корпоративному автопарку без юридического владения (лизинг). Актуально при появлении корпоративных клиентов.

**`LicensePlateRegistry`** — глобальная уникальность госномера с учётом страны, региона и периода действия. Требует моделирования lifecycle регистрационных действий и интеграции с государственными реестрами. Не вводится на MVP.

**`OwnershipDisputeCase`** — полноценный агрегат для ведения спора по владению с юридической процедурой, несколькими участниками, историей заявлений и решений. Для MVP dispute resolution выполняется Platform Moderator через `resolve_dispute` без отдельного lifecycle.

**`LegalReviewCase`** — юридическое рассмотрение спора с участием юристов, судебных инстанций или нотариусов. Полностью выходит за рамки MVP.

**`EvidenceLedger`** — immutable ledger доказательств для disputes и legal cases. Документы, временные метки, подписи. Отложить до появления OwnershipDisputeCase или LegalReviewCase.

---

### Closed Decisions

Все открытые вопросы Vehicle Domain закрыты. Принятые решения отражены в соответствующих разделах выше. Для справки — краткое резюме:

| Вопрос | Решение |
|---|---|
| Reliable identity set | VIN / (plate + doc) / trusted external ref. Правило в VehicleActivationPolicy. |
| License plate uniqueness | Не является глобально уникальным. LicensePlateRegistry — Deferred. |
| Ownership при archive/delete Vehicle | Explicit workflow через VehicleLifecyclePolicy. Агрегаты не меняют друг друга. |
| Privileged operations | Authorization в application layer. `correct_vin` требует `vehicle.identity.correct_vin`. |
| Dispute resolution | Platform Moderator / Admin через `vehicle.ownership.resolve_dispute`. Юридический процесс — Deferred. |

---

## Итоговая карта зависимостей

```
shared-kernel
    ↑
domain crates (customer, vehicle, contact, ownership)
    ↑
application crates (handlers, sagas, ports)
    ↑                   ↑
infrastructure      api layer
    ↑                   ↑
         services (binary, DI composition)
```

Направление зависимостей — строго внутрь. Infrastructure implements domain traits — Dependency Inversion на уровне crate boundaries, гарантировано Cargo.
