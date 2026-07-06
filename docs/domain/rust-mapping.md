# Customer Rust Mapping

## 1. Goal

Этот документ переводит модель из `docs/domain/customer.md` в Rust-oriented
implementation blueprint. Он фиксирует границы модулей, форму типов, публичные
API и места проверки инвариантов, но не является полной реализацией.

Customer domain состоит из независимых агрегатов:

- `Customer` владеет lifecycle клиента;
- `CustomerContactBook` владеет контактными каналами и primary contact;
- `CustomerProfile` владеет публичными данными профиля;
- `CustomerPreferences` владеет пользовательскими настройками;
- `CustomerConsentLedger` владеет юридическими фактами согласий;
- `IdentityRegistryEntry` владеет состоянием одного глобально уникального
  identity key.

Агрегаты связываются типизированными идентификаторами, а не вложенными object
graphs. Поля агрегатов закрыты. Состояние меняется только именованными
бизнес-методами. Cross-aggregate orchestration, authorization, транзакции и
outbox принадлежат application/infrastructure boundaries.

## 2. Module Structure

Рекомендуемая структура сохраняет отдельный модуль на каждый агрегат и не
превращает `common` в хранилище бизнес-логики:

```text
crates/domain/src/customer/
├── mod.rs
├── ids.rs
├── repositories.rs
├── common/
│   ├── mod.rs
│   ├── action_context.rs
│   ├── actor_ref.rs
│   ├── aggregate_version.rs
│   ├── change_outcome.rs
│   └── errors.rs
├── lifecycle/
│   ├── mod.rs
│   ├── aggregate.rs
│   ├── status.rs
│   ├── reasons.rs
│   ├── activation_permit.rs
│   ├── activation_policy.rs
│   ├── events.rs
│   └── errors.rs
├── contact_book/
│   ├── mod.rs
│   ├── aggregate.rs
│   ├── contacts.rs
│   ├── primary_contact.rs
│   ├── verification.rs
│   ├── reservation_proof.rs
│   ├── events.rs
│   └── errors.rs
├── profile/
│   ├── mod.rs
│   ├── aggregate.rs
│   ├── display_name.rs
│   ├── full_name.rs
│   ├── avatar.rs
│   ├── location.rs
│   ├── events.rs
│   └── errors.rs
├── preferences/
│   ├── mod.rs
│   ├── aggregate.rs
│   ├── notification_preferences.rs
│   ├── communication_preferences.rs
│   ├── quiet_hours.rs
│   ├── events.rs
│   └── errors.rs
├── consent/
│   ├── mod.rs
│   ├── ledger.rs
│   ├── consent_kind.rs
│   ├── effective_consent.rs
│   ├── consent_grant.rs
│   ├── consent_revocation.rs
│   ├── events.rs
│   └── errors.rs
├── identity_registry/
│   ├── mod.rs
│   ├── entry.rs
│   ├── identity_key.rs
│   ├── reservation.rs
│   ├── claimed_identity.rs
│   ├── normalizer.rs
│   ├── outbox.rs
│   ├── events.rs
│   └── errors.rs
└── events/
    ├── mod.rs
    ├── envelope.rs
    ├── pending_event.rs
    ├── event_type.rs
    └── role_snapshot.rs
```

`mod.rs` должен экспортировать только устойчивый публичный API. Внутренние
типы состояния и reconstitution helpers остаются в дочерних модулях с
`pub(crate)` visibility. Если `events` станет большим, допустимо держать
pending event рядом с агрегатом, а в общем модуле оставить только envelope
metadata. Не следует заранее вводить отдельный crate на каждый агрегат.

## 3. Aggregate Roots

### Customer

```rust
pub struct Customer {
    id: CustomerId,
    status: CustomerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingCustomerEvent>,
}
```

Публичный API выражает lifecycle, а не mutation primitives:

```rust
impl Customer {
    pub fn activate(
        &mut self,
        permit: &ActivationPermit,
        ctx: &ActionContext,
    ) -> Result<ChangeOutcome, CustomerError>;

    pub fn suspend(
        &mut self,
        reason: SuspensionReason,
        ctx: &ActionContext,
    ) -> Result<ChangeOutcome, CustomerError>;

    pub fn restore(&mut self, ctx: &ActionContext)
        -> Result<ChangeOutcome, CustomerError>;
    pub fn block(&mut self, reason: BlockReason, ctx: &ActionContext)
        -> Result<ChangeOutcome, CustomerError>;
    pub fn reduce_block_to_suspension(
        &mut self,
        reason: SuspensionReason,
        ctx: &ActionContext,
    ) -> Result<ChangeOutcome, CustomerError>;
    pub fn unblock(&mut self, ctx: &ActionContext)
        -> Result<ChangeOutcome, CustomerError>;
    pub fn delete(&mut self, reason: DeletionReason, ctx: &ActionContext)
        -> Result<ChangeOutcome, CustomerError>;
}
```

`activate` проверяет принадлежность, срок действия и expected versions permit.
Фактическую согласованность source versions при commit повторно обеспечивает
application `UnitOfWork`.

### CustomerContactBook

```rust
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone: Option<PhoneContact>,
    email: Option<EmailContact>,
    telegram: Option<TelegramContact>,
    primary: Option<PrimaryContact>,
    version: AggregateVersion,
    updated_at: DateTime<Utc>,
    pending_events: Vec<PendingContactEvent>,
}
```

Операции, меняющие globally unique identity, принимают
`&IdentityReservationProof`: `add_phone`, `change_phone`, `add_email`,
`change_email`, `link_telegram`. Proof проверяется на identity key, customer,
reservation и expiry. Агрегат не загружает `IdentityRegistryEntry` и не вызывает
registry service. `change_primary` работает только с существующим и допустимым
каналом.

### CustomerProfile

`CustomerProfile` содержит `CustomerId`, display/full name, avatar, location,
version, timestamps и pending events. Его API ограничен методами
`change_display_name`, `change_full_name`, `change_avatar` и `change_location`.
Каждый метод принимает уже валидированный value object и `ActionContext`.

### CustomerPreferences

`CustomerPreferences` владеет language, timezone, notification и communication
preferences, quiet hours, preferred channel и allowed channels. Методы:
`change_language`, `change_timezone`, `update_notification_preferences`,
`update_communication_preferences`, `set_quiet_hours`, `clear_quiet_hours`,
`change_preferred_channel`, `change_allowed_channels`.

Инвариант `preferred_channel` belongs to `allowed_channels` проверяется внутри
агрегата при любом изменении обеих сторон.

### CustomerConsentLedger

Ledger хранит юридические факты, а не mutable booleans. `grant_consent` создает
`ConsentGrant`; `revoke_consent` добавляет `ConsentRevocation`. История не
перезаписывается. Effective state вычисляется из последнего применимого grant и
revocation для `(ConsentKind, DocumentVersion)`.

### IdentityRegistryEntry

Один aggregate root соответствует одному нормализованному `IdentityKey`.
Методы `reserve`, `claim`, `release`, `cancel_reservation` и
`expire_reservation` реализуют state machine. Global uniqueness дополнительно
защищается уникальным ограничением persistence storage.

У агрегатов не должно быть `set_status`, `update_field`, generic patch API или
public mutable fields. Read-only getters допустимы, когда они нужны policy или
application layer и не раскрывают возможность нарушить инвариант.

## 4. Value Objects

Все value objects имеют private fields и создаются через constructors,
сохраняющие инварианты.

| Type | Rust form | Constructor and validation |
| --- | --- | --- |
| `CustomerId` | UUID newtype | `new`, `from_uuid`; UUID не валидируется повторно |
| `PhoneNumber` | string newtype | `parse`; normalization и допустимый формат, `PhoneNumberError` |
| `EmailAddress` | string newtype | `parse`; trim/canonicalization и syntax, `EmailAddressError` |
| `TelegramUserId` | integer/string newtype | `new`; допустимый provider range, `TelegramUserIdError` |
| `DisplayName` | string newtype | `new`; trim, length, forbidden chars, `DisplayNameError` |
| `FullName` | struct/newtype | `new`; непустые допустимые части и length, `FullNameError` |
| `AvatarAssetId` | UUID newtype | `new`, `from_uuid`; type safety |
| `LanguageCode` | string newtype | `parse`; supported BCP 47 subset, `LanguageCodeError` |
| `IanaTimezone` | string newtype | `parse`; known IANA identifier, `IanaTimezoneError` |
| `CityName` | string newtype | `new`; trim и length, `LocationError` |
| `CountryCode` | two-letter newtype | `parse`; canonical uppercase ISO code, `LocationError` |
| `RegionCode` | string newtype | `new`; country-aware format where required, `LocationError` |
| `GeoId` | provider-qualified newtype | `new`; non-empty provider/value, `LocationError` |
| `IdentityKey` | struct | `new(kind, normalized)`; matching kind/value, `IdentityKeyError` |
| `NormalizedIdentityValue` | string newtype | только через normalizer, `IdentityNormalizationError` |
| `IdentityReservationProof` | struct | private constructor; только `IdentityReservation::proof` |
| `ActivationPermit` | struct | только activation policy; complete snapshots и expiry |
| `ActionContext` | struct | application factory; complete audit metadata |
| `PrincipalRoleSnapshot` | struct | factory from authorization decision; stable role identity |
| `QuietHours` | struct | `new(start, end, timezone)`; non-zero valid interval, `QuietHoursError` |
| `CommunicationPreferences` | struct | `new`; preferred/allowed consistency, `CommunicationPreferencesError` |
| `NotificationPreferences` | struct | `new`; supported channels and categories, `NotificationPreferencesError` |
| `EffectiveConsent` | enum | создается ledger calculation; externally not freely constructible |

Пример локальной валидации:

```rust
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: impl Into<String>) -> Result<Self, DisplayNameError> {
        let value = value.into().trim().to_owned();
        validate_display_name(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Normalization, зависящая от внешней библиотеки или provider policy, может быть
domain service (`IdentityNormalizer` port). Конструктор value object все равно
не должен принимать заведомо неканоническое значение.

## 5. Strongly Typed IDs

Идентификаторы являются отдельными `#[repr(transparent)]` newtypes вокруг
`Uuid`: `CustomerId`, `PrincipalId`, `RoleId`, `EventId`,
`IdentityReservationId`, `CorrelationId` и идентификаторы остальных агрегатов.

```rust
macro_rules! define_id {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self { Self(Uuid::new_v4()) }

            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self { Self(value) }

            #[must_use]
            pub const fn as_uuid(&self) -> Uuid { self.0 }
        }
    };
}
```

Macro implementation также добавляет `Display`, `From<Uuid>` и обратный
`From<Id> for Uuid`. `Default` не реализуется: случайный ID не является
семантическим default. Если `new_without_default` срабатывает, lint разрешается
локально у macro с явным объяснением. Следует согласовать термин
`CustomerId` с существующим `ClientId` до начала миграции, а не держать два
взаимозаменяемых ID.

## 6. State Machines

```rust
pub enum CustomerStatus {
    Draft,
    Active,
    Suspended(Suspension),
    Blocked(Block),
    Deleted(Deletion),
}

pub enum EffectiveConsent {
    NotGranted,
    Granted(ConsentGrant),
    Revoked {
        grant: ConsentGrant,
        revocation: ConsentRevocation,
    },
}

pub enum IdentityRegistryStatus {
    Available,
    Reserved { reservation: IdentityReservation },
    Claimed { owner_customer_id: CustomerId },
}
```

Переходы `CustomerStatus` доступны только через методы `Customer` и exhaustive
`match`. Не используется `String`, generic transition table или public setter.
Payload states (`Suspension`, `Block`, `Deletion`) сохраняют reason, actor и
time, необходимые для решений и аудита.

`EffectiveConsent` не хранится как независимо изменяемое поле: ledger получает
его детерминированным вычислением. В SQL состояние registry `Available` может
быть представлено отсутствием строки; repository обязан восстановить это как
доменное состояние, не раскрывая storage detail агрегату.

## 7. Domain Events

Каждый агрегат создает typed immutable pending events:

```rust
pub enum CustomerEvent {
    CustomerCreatedV1 { customer_id: CustomerId },
    CustomerActivatedV1 { customer_id: CustomerId },
    CustomerSuspendedV1 {
        customer_id: CustomerId,
        reason: SuspensionReason,
    },
}

pub struct PendingCustomerEvent {
    payload: CustomerEvent,
    occurred_at: DateTime<Utc>,
    actor: ActorRef,
    role_snapshots: Vec<PrincipalRoleSnapshot>,
    correlation_id: CorrelationId,
    causation_id: Option<EventId>,
}

pub struct EventEnvelope<T> {
    event_id: EventId,
    aggregate_id: AggregateId,
    aggregate_version: AggregateVersion,
    event_type: EventType,
    schema_version: EventSchemaVersion,
    occurred_at: DateTime<Utc>,
    actor: ActorRef,
    principal_role_snapshots: Vec<PrincipalRoleSnapshot>,
    correlation_id: CorrelationId,
    causation_id: Option<EventId>,
    payload: T,
}
```

Аналогично определяются `ContactEvent`, `ProfileEvent`, `PreferencesEvent`,
`ConsentEvent` и `IdentityRegistryEvent`. Event variants имеют явный schema
suffix (`V1`) и конкретные payload fields; generic old/new value bags не
используются.

Aggregate копирует audit fields из `ActionContext` в pending event, но не знает
`EventEnvelope`. На persistence boundary `EventEnvelopeFactory`:

- назначает новый `EventId`;
- сопоставляет variant с `EventType` и `EventSchemaVersion`;
- назначает последовательный `AggregateVersion` при успешном append;
- переносит неизмененными occurred_at, actor, roles и correlation metadata.

Role snapshots фиксируют роли, участвовавшие в решении в момент действия;
последующее изменение access model не должно переписывать историю. Events
immutable, потому что это зафиксированные факты, основа replay, audit и
projections. После успешного commit pending events извлекаются controlled
методом; при failed commit они не должны считаться опубликованными.

## 8. Action Context and Audit

```rust
pub struct ActionContext {
    actor: ActorRef,
    role_snapshots: Vec<PrincipalRoleSnapshot>,
    now: DateTime<Utc>,
    correlation_id: CorrelationId,
    causation_id: Option<EventId>,
}
```

`ActionContextFactory` находится в application layer. Он принимает успешный
authorization decision либо trusted system/service identity, проверяет полноту
audit metadata и создает immutable context. Aggregate принимает `&ActionContext`
и не обращается к `AccessContext`, clock, request или authorization service.

Поток данных:

```text
AccessContext -> AuthorizationDecision -> ActionContext
              -> PendingDomainEvent -> EventEnvelope
```

## 9. Errors

Не вводится единый огромный `DomainError`. Ошибки scoped по bounded behavior:

- `CustomerError`;
- `ContactError`;
- `ProfileError`;
- `PreferencesError`;
- `ConsentError`;
- `IdentityRegistryError`;
- `ActivationApplicationError`;
- `ContactApplicationError`.

Domain errors описывают business failure: invalid transition, expired permit,
proof mismatch, invalid primary contact или absent grant. Repository/storage
errors переводятся application layer в application errors и не раскрывают SQL.

```rust
pub type DomainResult<E> = Result<ChangeOutcome, E>;

pub enum CustomerError {
    InvalidTransition {
        from: CustomerStatusKind,
        action: CustomerAction,
    },
    ActivationPermitExpired,
    ActivationPermitCustomerMismatch,
    ActivationPermitCustomerVersionMismatch,
}
```

Constructor errors value objects остаются отдельными небольшими enum. `String`
не используется как error model; человекочитаемый text формируется через
`Display` или на transport boundary.

## 10. Repository Ports

Repository traits объявляются в domain как ports, а PostgreSQL реализации и
persistence mapping находятся в infrastructure:

```rust
pub trait CustomerRepository {
    fn load(
        &self,
        id: CustomerId,
    ) -> Result<Versioned<Customer>, RepositoryError>;

    fn save(
        &self,
        customer: &mut Customer,
        expected_version: AggregateVersion,
    ) -> Result<(), RepositoryError>;
}
```

Аналогичные ports нужны для `CustomerContactBook`, `CustomerProfile`,
`CustomerPreferences`, `CustomerConsentLedger` и `IdentityRegistryEntry`.
`Versioned<T>` несет агрегат и загруженную version; optimistic lock сравнивает
ее при save.

Trait signatures могут стать `async` на application-facing boundary, но domain
не должен зависеть от `sqlx`, PostgreSQL types или database transactions.
Транзакционно согласованное сохранение нескольких агрегатов координируется
`UnitOfWork` application port, а не методом одного repository.

## 11. Application Layer Mapping

В aggregate не находятся orchestration, загрузка нескольких агрегатов,
authorization, `UnitOfWork`, transactions, outbox dispatch, projection building,
DTO mapping и external API calls.

### CustomerActivationApplicationService

Activation flow:

1. Авторизовать действие.
2. Создать `ActionContext`.
3. Загрузить `Customer`.
4. Загрузить `CustomerContactBook`.
5. Загрузить `CustomerConsentLedger`.
6. Передать snapshots и versions в `CustomerActivationPolicy`, получить permit.
7. Вызвать `Customer::activate`.
8. Сохранить Customer с optimistic lock.
9. В `UnitOfWork` повторно проверить source versions из permit.
10. Создать envelopes, записать events и commit атомарно.

Policy проверяет cross-aggregate eligibility и выдает короткоживущий
`ActivationPermit`. Permit не заменяет commit-time concurrency checks.

### CustomerContactApplicationService

Identity replacement flow:

1. Нормализовать raw identity в `IdentityKey`.
2. Зарезервировать key в `IdentityRegistryEntry`.
3. Получить `IdentityReservationProof`.
4. Загрузить `CustomerContactBook`.
5. Вызвать `change_phone`, `change_email` или `link_telegram` с proof.
6. Сохранить contact book с optimistic lock.
7. Атомарно записать outbox commands для claim нового и release старого key.
8. Commit.
9. Dispatcher idempotently выполняет claim/release.

Failed commit оставляет reservation до cancel/TTL expiry, но не делает identity
claimed. Application errors различают authorization, conflict, domain rejection
и infrastructure failure.

## 12. Identity Registry Mapping

```rust
pub enum IdentityKind {
    Phone,
    Email,
    Telegram,
}

pub struct IdentityKey {
    kind: IdentityKind,
    value: NormalizedIdentityValue,
}

pub struct IdentityReservation {
    id: IdentityReservationId,
    identity_key: IdentityKey,
    customer_id: CustomerId,
    expires_at: DateTime<Utc>,
}

pub struct IdentityReservationProof {
    reservation_id: IdentityReservationId,
    identity_key: IdentityKey,
    customer_id: CustomerId,
    expires_at: DateTime<Utc>,
}
```

`IdentityReservationProof::new` private. Только
`IdentityReservation::proof(&self)` может создать proof из уже валидной
reservation. Contact book имеет read-only проверку proof и не может подделать
или продлить его.

`IdentityRegistryEntry` хранит key, status, version и pending registry events.
`ClaimedIdentity` является результатом успешного claim, а не вторым владельцем
state. `IdentityNormalizer` канонизирует phone/email/Telegram по явной policy.

```rust
pub enum IdentityOutboxCommand {
    Claim {
        reservation_id: IdentityReservationId,
        customer_id: CustomerId,
    },
    Release {
        identity_key: IdentityKey,
        customer_id: CustomerId,
    },
}

pub trait IdentityOutbox {
    fn enqueue(&mut self, command: IdentityOutboxCommand);
}
```

Конкретная outbox table, serialization, retries и dispatcher принадлежат
infrastructure. Commands имеют стабильный idempotency key. `claim`, `release`,
`cancel_reservation` и expiry handling должны быть idempotent либо возвращать
структурированный conflict при несовместимом owner/state.

## 13. No-op and Versioning Semantics

```rust
pub enum ChangeOutcome {
    Changed,
    NoChange,
}
```

Единое правило для всех агрегатов: если команда не меняет observable domain
state, метод возвращает `NoChange`, не обновляет `updated_at`, не увеличивает
`AggregateVersion` и не создает event. Version назначается только сохраненному
изменению. Infrastructure не должно увеличивать version только потому, что был
вызван `save`.

Увеличение version и envelope assignment должны быть атомарны. Для нескольких
pending events версии назначаются последовательно от expected version. Conflict
не очищает pending events и не маскируется как `NoChange`.

## 14. Visibility and Encapsulation Rules

- Aggregate fields всегда private.
- Public constructors доступны только если полностью сохраняют invariants.
- Named business methods являются единственным mutation API.
- Read-only accessors возвращают borrowed values или копируемые IDs.
- Reconstitution constructors имеют `pub(crate)` visibility и принимают
  validated persistence state.
- Unsafe/unchecked constructors, если неизбежны, изолируются в persistence
  mapping module и не экспортируются из domain public API.
- Pending event mutation и version assignment недоступны обычным callers.
- Repository rehydration идет по отдельному controlled path.
- `pub` используется для меж-crate контракта, `pub(crate)` для сборки domain,
  private visibility для деталей агрегата.

Не следует создавать trait для каждого value object или aggregate только ради
унификации. Traits оправданы для ports и реально полиморфного поведения.

## 15. Serialization / Reconstitution

Domain types не являются transport DTO или database row models. HTTP DTO и
application commands находятся в application/transport modules, SQL rows и
mapping в infrastructure.

Прямой `Deserialize` для aggregate root опасен: он позволяет обойти constructor
и создать invalid state. Предпочтительный путь:

1. Infrastructure десериализует row/event DTO.
2. Mapper преобразует primitives в validated value objects.
3. `pub(crate)` reconstitution API проверяет structural invariants.
4. Aggregate восстанавливается с пустыми pending events и сохраненной version.

`Serialize` допустим для stable event payloads или явно предназначенных для
persistence value objects. Serde attributes не должны становиться частью
business API. Schema migration и backward compatibility event payloads
обрабатываются на event serialization boundary.

## 16. Testing Strategy

Domain unit tests должны покрывать observable behavior:

- каждый разрешенный и запрещенный lifecycle transition;
- invariant violations value objects и aggregates;
- `NoChange` без timestamp, version и event mutations;
- точный typed event и audit metadata для каждого изменения;
- identity proof: matching, mismatch, expiry и невозможность внешнего создания;
- activation permit: customer, expiry и source version checks;
- consent grant/revoke state machine и document versions;
- quiet hours validation, timezone и boundary intervals;
- preferred/allowed communication channel consistency;
- identity registry reserve/claim/release/cancel/expire transitions.

Application tests используют fake/in-memory ports и проверяют activation и
contact flows, authorization ordering, optimistic lock conflicts, rollback,
event envelope creation и atomic outbox enqueue. Infrastructure integration
tests проверяют unique constraints, SQL mapping, transaction boundaries,
idempotent dispatcher и concurrent reservations. Тесты не должны зависеть от
private field layout.

## 17. Open Questions

1. Нужна миграция существующего `ClientId` в `CustomerId` или `Client` остается
   отдельным ubiquitous term? До реализации должен существовать один canonical
   type.
2. Какие contact identities глобально уникальны: phone, email и Telegram все
   вместе либо policy различается по tenant/organization?
3. Какой exact TTL reservation и допустим ли grace period при claim после
   commit?
4. Какие consent kinds и document versions обязательны для activation?
5. Должен ли activation commit атомарно сохранять только `Customer` с version
   assertions источников или также изменять другие агрегаты?
6. Как представляется удаление: irreversible terminal state, legal anonymization
   workflow или оба процесса отдельно?
7. Нужны ли `CustomerProfile` и `CustomerPreferences` сразу отдельными
   aggregates в первой реализации, либо их extraction можно выполнить поэтапно
   без изменения публичного domain contract?
8. Какой event store используется: отдельный append-only stream, transactional
   outbox поверх state tables или оба механизма?
9. Где живет список поддерживаемых languages/timezones/channels: compile-time
   domain policy или versioned configuration port?
10. Какие role fields должны входить в `PrincipalRoleSnapshot`, чтобы audit был
    достаточным и не сохранял лишние персональные данные?
