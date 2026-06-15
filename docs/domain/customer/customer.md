# Customer domain architecture review

Текущая модель улучшает типобезопасность по сравнению с CRUD, но остается
слишком крупной consistency boundary. Главная проблема не в количестве типов, а
в том, что данные с разными жизненными циклами, частотой изменений, требованиями
к аудиту и владельцами правил объединены под одной блокировкой `Customer`.

Ниже предлагается намеренно ломающая переработка. `Customer` остается Aggregate
Root бизнес-жизненного цикла клиента, но контакты, профиль, предпочтения и
юридические согласия становятся отдельными агрегатами или bounded contexts.

## Findings

### 1. Customer является god aggregate

#### Problem

Один `Customer` владеет lifecycle, контактами, профилем, preferences, consent,
timestamps и всеми событиями. Эти части не требуют общей атомарности для
большинства операций.

#### Why it is bad

Через 2-5 лет изменение timezone будет конфликтовать с блокировкой аккаунта,
обновление аватара с подтверждением телефона, а массовое обновление legal
consent с пользовательскими настройками. Один stream/version увеличит число
optimistic locking conflicts, размер snapshot и стоимость восстановления.

#### Recommended solution

Разделить consistency boundaries:

- `Customer` — lifecycle и бизнес-доступность клиента;
- `CustomerContactBook` — контактные каналы и primary contact;
- `CustomerProfile` — публичное представление человека;
- `CustomerPreferences` — пользовательские настройки;
- `CustomerConsentLedger` — юридические факты и версии документов;
- Auth и Access — отдельные bounded contexts.

#### Rust implementation

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

#### DDD justification

Aggregate boundary определяется инвариантами, требующими немедленной
согласованности, а не удобством загрузки одного объекта.

#### Priority

Critical

### 2. CustomerIdentity ошибочно смоделирован как OwnedEntity

#### Problem

`CustomerIdentity` дублирует `CustomerId`, не имеет собственного identity и
объединяет login identity с contact channels. Это не Entity и не единая доменная
концепция.

#### Why it is bad

Ложная Entity провоцирует отдельный repository/ID, усложняет mapping и скрывает
реальную границу Auth. Дублированный `CustomerId` создает возможность
восстановить противоречивый объект.

#### Recommended solution

Удалить `CustomerIdentity`. Текущие способы связи хранить в отдельном агрегате
`CustomerContactBook`. Login credentials и login identities полностью передать
Auth bounded context.

#### Rust implementation

```rust
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone: Option<PhoneContact>,
    email: Option<EmailContact>,
    telegram: Option<TelegramContact>,
    primary: Option<PrimaryContact>,
    version: AggregateVersion,
    pending_events: Vec<PendingContactEvent>,
}
```

#### DDD justification

Contact book имеет самостоятельные инварианты и consistency boundary, но
ссылается на Customer только типизированным ID.

#### Priority

Critical

### 3. LoginIdentityRef не должен существовать в Customer context

#### Problem

`LoginIdentityRef` связывает Customer domain с технической моделью входа.

#### Why it is bad

Добавление OAuth provider, passkey, SSO или account merge заставит менять
Customer aggregate и его event schema. Возникает циклическая зависимость между
Customer и Auth contexts.

#### Recommended solution

Удалить `LoginIdentityRef` полностью. `AuthAccount` владеет credentials и
login identities, а `CustomerId` является внешней ссылкой на business subject.

#### Rust implementation

```rust
pub struct AuthAccount {
    id: AuthAccountId,
    subject: CustomerId,
    login_identities: NonEmptySet<LoginIdentity>,
    security_state: SecurityState,
}
```

#### DDD justification

Authentication и Customer Management используют разные ubiquitous languages и
должны интегрироваться контрактами, а не общей внутренней моделью.

#### Priority

Critical

### 4. Generic ChangePayload разрушает type safety

#### Problem

`OldValue`, `NewValue` и `ChangeReason` не задают тип конкретного изменения.

#### Why it is bad

Serializer допускает несовместимые пары значений, consumer вынужден делать
runtime dispatch, а schema registry не может надежно описать payload. Любое
расширение превращает generic payload в tagged JSON bag.

#### Recommended solution

Каждый event должен иметь собственный payload struct с конкретными типами.

#### Rust implementation

```rust
pub struct PhoneChanged {
    pub previous: PhoneNumber,
    pub current: PhoneNumber,
    pub reason: ContactChangeReason,
}

pub enum ContactBookEvent {
    PhoneChanged(PhoneChanged),
    PrimaryContactChanged(PrimaryContactChanged),
}
```

#### DDD justification

Domain event является конкретным фактом ubiquitous language, а не универсальным
контейнером изменений.

#### Priority

Critical

### 5. Customer API слишком широк

#### Problem

Один `impl Customer` содержит lifecycle, contacts, profile, preferences и
consent operations. Разделение на несколько traits или impl-блоков не исправит
неверную aggregate boundary.

#### Why it is bad

Большая публичная поверхность увеличивает число допустимых interleavings,
усложняет тестовую матрицу и заставляет каждый consumer зависеть от монолитного
типа.

#### Recommended solution

Разделить агрегаты. Использовать несколько `impl Customer` только для навигации
по коду lifecycle aggregate, а не для имитации модульности god object.

#### Rust implementation

```rust
impl Customer {
    pub fn activate(&mut self, permit: ActivationPermit, ctx: ActionContext<'_>)
        -> Result<ChangeOutcome, CustomerError>;

    pub fn suspend(&mut self, reason: SuspensionReason, ctx: ActionContext<'_>)
        -> Result<ChangeOutcome, CustomerError>;
}

impl CustomerContactBook {
    pub fn change_phone(&mut self, command: ChangePhone, ctx: ActionContext<'_>)
        -> Result<ChangeOutcome, ContactBookError>;
}
```

#### DDD justification

Поведение располагается на агрегате, который владеет соответствующим
инвариантом.

#### Priority

Critical

### 6. CustomerStatusTransitions не является отдельным domain object

#### Problem

Отдельный класс правил переходов дублирует поведение методов агрегата и создает
второй источник истины.

#### Why it is bad

Диаграмма и код неизбежно расходятся: enum разрешает состояние, transitions
class описывает граф, а методы Customer реализуют третий вариант.

#### Recommended solution

Использовать обычный Rust `enum CustomerStatus`; допустимые переходы кодировать
исключительно именованными методами и exhaustive `match`. Typestate здесь
нежелателен: состояние восстанавливается из persistence динамически, а единый
repository должен возвращать любой статус.

#### Rust implementation

```rust
pub enum CustomerStatus {
    Draft,
    Active,
    Suspended(Suspension),
    Blocked(Block),
    Deleted(Deletion),
}

match &self.status {
    CustomerStatus::Draft => self.raise(CustomerActivated { ... }),
    current => return Err(CustomerError::CannotActivate(current.kind())),
}
```

#### DDD justification

Инвариант перехода принадлежит Aggregate Root и проверяется в момент команды.

#### Priority

High

### 7. ActorRef смешивает identity actor и его role

#### Problem

`SupportOperator`, `Admin` и `GarageManager` представлены вариантами actor enum,
хотя это роли, а не стабильные виды субъекта.

#### Why it is bad

Один principal может иметь несколько ролей; роли меняются без смены identity.
Customer context начинает зависеть от Garage и Access terminology.

#### Recommended solution

Actor должен идентифицировать инициатора, а authorization decision должен
передаваться application layer как capability/permit. Роли остаются в Access
context.

#### Rust implementation

```rust
pub enum ActorRef {
    Customer(CustomerId),
    Principal(PrincipalId),
    System(SystemActorId),
    Service(ServiceId),
}

pub struct AuthorizedAction<A> {
    action: A,
    authorized_by: AuthorizationDecisionId,
}
```

#### DDD justification

Customer проверяет бизнес-переход; Access context решает, имеет ли principal
право инициировать команду.

#### Priority

High

### 8. ConsentEvidence и CustomerConsents имеют неверную boundary

#### Problem

Customer хранит legal evidence, IP, user agent и revoked timestamps. Четыре
специализированных wrapper-типа вокруг общего evidence почти не добавляют
семантики, но связывают lifecycle Customer с legal audit.

#### Why it is bad

История согласий растет, версии документов меняются независимо, требования
retention отличаются от Customer, а IP/user agent могут считаться отдельными
персональными данными. Обновление consent вызывает лишний конфликт с Customer.

#### Recommended solution

Создать отдельный `CustomerConsentLedger` в Consent bounded context. Моделировать
`ConsentKind`, `ConsentDocumentRef`, `ConsentGrant` и `ConsentRevocation`.
Customer activation принимает короткоживущий `ActivationPermit`, выданный
policy после проверки contacts и required consent. Перед commit application
layer обязан повторно сравнить версии source aggregates из permit.

#### Rust implementation

```rust
pub enum ConsentKind {
    TermsOfService,
    PrivacyPolicy,
    DataProcessing,
    Marketing,
}

pub struct ConsentGrant {
    document: ConsentDocumentRef,
    accepted_at: DateTime<Utc>,
    capture: ConsentCapture,
}

pub enum EffectiveConsent {
    NotGranted,
    Granted(ConsentGrant),
    Revoked { grant: ConsentGrant, revocation: ConsentRevocation },
}
```

#### DDD justification

Legal consent имеет собственный язык, историю, retention и invariants, поэтому
является отдельной моделью, а не частью Customer lifecycle aggregate.

#### Priority

Critical

### 9. Profile и preferences создают лишние конфликты

#### Problem

Профиль и notification preferences лежат внутри Customer, хотя изменяются часто
и не участвуют в lifecycle transitions.

#### Why it is bad

Мобильный клиент может одновременно обновить avatar, timezone и notification
settings. Один aggregate version превратит независимые изменения в конфликты.

#### Recommended solution

Выделить `CustomerProfile` и `CustomerPreferences` как агрегаты с собственными
версиями. `CustomerProfile` не нужно дробить дальше: full name, display name,
avatar и location образуют небольшой cohesive profile snapshot.

#### Rust implementation

```rust
pub struct CustomerProfile {
    customer_id: CustomerId,
    full_name: Option<FullName>,
    display_name: DisplayName,
    avatar: Option<AvatarAssetId>,
    location: Option<CustomerLocation>,
    version: AggregateVersion,
}
```

#### DDD justification

Отдельный агрегат допустим без собственного surrogate ID: `CustomerId` может
быть identity one-to-one aggregate, если lifecycle и repository независимы.

#### Priority

High

### 10. Event metadata неполна и смешана с domain payload

#### Problem

События не имеют `event_id`, `schema_version`, `aggregate_version`,
`causation_id`; metadata вложена непосредственно в каждый enum variant.

#### Why it is bad

Невозможны надежная дедупликация, упорядочивание, replay, tracing и безопасная
schema evolution. Consumer не может отличить повторную доставку от нового
события.

#### Recommended solution

Разделить typed domain event data и сериализуемый envelope. Actor может быть
business-relevant частью event context; transport metadata не должна
дублироваться в payload.

#### Rust implementation

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub aggregate_version: AggregateVersion,
    pub event_type: EventType,
    pub schema_version: EventSchemaVersion,
    pub occurred_at: DateTime<Utc>,
    pub actor: ActorRef,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub payload: E,
}
```

#### DDD justification

Domain event data описывает факт. Envelope обеспечивает event store и
integration semantics, не загрязняя конкретный payload generic metadata.

#### Priority

Critical

### 11. Optimistic locking отсутствует

#### Problem

В агрегатах нет revision/version и явного expected version contract.

#### Why it is bad

Две параллельные команды могут потерять изменение primary contact, повторно
активировать customer или записать события в неправильном порядке. Проверка
уникальности контакта отдельно от commit создает TOCTOU race.

#### Recommended solution

Каждый агрегат получает `AggregateVersion`. Repository выполняет compare-and-set
или append с expected version. Уникальность normalized contact обеспечивается
атомарным unique index/reservation в Identity Registry context.

#### Rust implementation

```rust
pub trait CustomerRepository {
    async fn load(&self, id: CustomerId) -> Result<Versioned<Customer>, LoadError>;
    async fn save(
        &self,
        customer: &Customer,
        expected: AggregateVersion,
    ) -> Result<(), SaveError>;
}

pub enum SaveError {
    ConcurrentModification,
    Storage(StorageFailure),
}
```

#### DDD justification

Aggregate гарантирует consistency только при атомарной фиксации одной версии.
Repository обязан сохранять эту семантику.

#### Priority

Critical

### 12. Domain events слишком мелкие для некоторых consumers и слишком общие для других

#### Problem

Текущая модель одновременно содержит field-level profile events и не определяет
versioning/compatibility policy. Неясно, являются ли они internal domain events
или публичными integration events.

#### Why it is bad

Внешние consumers привязываются к внутренней структуре агрегата. Переименование
поля становится breaking change. При event sourcing удалить или изменить старый
event нельзя.

#### Recommended solution

Разделить:

- domain events — внутренние typed facts агрегата;
- integration events — стабильные контракты, публикуемые outbox mapper;
- event-sourced history — immutable events с upcasters.

Новые поля добавлять backward-compatible; breaking payload создавать как новый
event type/schema version. Старые events никогда не переписывать.

#### Rust implementation

```rust
pub enum CustomerEvent {
    Activated(CustomerActivatedV1),
    Suspended(CustomerSuspendedV1),
    Blocked(CustomerBlockedV1),
    Deleted(CustomerDeletedV1),
}

pub trait Upcast<From, To> {
    fn upcast(value: From) -> To;
}
```

#### DDD justification

Domain model свободно развивается внутри bounded context, а integration contract
изменяется явно и независимо.

#### Priority

High

## Checklist verdicts

### 1. CustomerIdentity как OwnedEntity — ✅ Agree

Замечание верно: это не Entity. У него нет самостоятельной identity, а
`CustomerId` дублирует root ID. Однако простая замена на Value Object также
недостаточна. Тип следует удалить и заменить отдельным `CustomerContactBook`
aggregate; login identity вынести в Auth.

### 2. LoginIdentityRef — ✅ Agree

В Customer Aggregate он не нужен. Полная замена: `AuthAccount` владеет
`LoginIdentity`; Customer/Contact contexts не знают provider, credentials,
passkeys или sessions. Связь выполняется через `CustomerId`.

### 3. ChangePayload — ✅ Agree

Generic payload ухудшает compile-time guarantees. Нужны event-specific structs:
`PhoneChanged`, `PrimaryContactChanged`, `DisplayNameChanged` и другие.

### 4. Customer Aggregate API — ✅ Agree

API является симптомом god aggregate. Дополнительные traits или impl-блоки сами
по себе не решают проблему. Нужны отдельные агрегаты с узкими API.

### 5. CustomerStatusTransitions — ✅ Agree

Отдельный объект правил не нужен. Transition invariant должен жить в именованных
методах `Customer` и exhaustive match по `CustomerStatus`.

### 6. CriticalActionAuthorizationPolicy — ✅ Agree

В представленном виде policy течет из Access context в Customer domain.
Application layer должен получить authorization decision/capability из Access,
после чего вызвать Customer. Customer проверяет actor presence, reason и
собственный state transition, но не роли.

### 7. Versioning Domain Events — ✅ Agree

Нужны `schema_version`, `aggregate_version`, immutable event type и upcasting
strategy. Version payload не заменяет aggregate sequence: это разные оси.

### 8. ConsentEvidence — ⚠ Partially Agree

Повторное использование evidence не всегда плохо: общие capture metadata можно
композировать. Плохо использовать один тип как полную семантику всех consent.
Решение: общие `ConsentCapture` и `ConsentDocumentRef`, но отдельные grant и
revocation records в Consent context.

### 9. Optimistic Locking — ✅ Agree

Необходим для каждого mutable aggregate. При event sourcing
`AggregateVersion == last applied stream sequence`; при state persistence это
CAS revision. Нельзя использовать `updated_at` как lock token.

### 10. Edge cases — ✅ Agree

- Primary contact не может указывать на отсутствующий или неподтвержденный канал.
- Удаление/замена primary contact требует атомарно выбрать другой primary либо
  вернуть domain error.
- Customer может существовать без contacts; `CustomerContactBook` может быть
  пустым, но activation permit тогда не выдается.
- Telegram unlink запрещен, если Telegram primary, пока primary не изменен.
- Quiet hours требуют timezone, поддержки интервала через полночь и DST policy.
- Parallel updates разрешаются независимо между агрегатами, но конфликтуют
  внутри одного aggregate version.
- Уникальность phone/email/Telegram требует atomic reservation/unique constraint,
  а не предварительного `exists()`.
- Cross-aggregate activation требует versioned permit или транзакционного
  application workflow, иначе возможен stale eligibility race.
- Если `Active` должен означать непрерывное выполнение prerequisites, contacts и
  consent нельзя отделять без синхронного deactivation workflow. Более чистая
  семантика: `Active` означает завершенный lifecycle transition, а eligibility
  для booking и других операций вычисляется по актуальным данным отдельно.

## Revised model

```mermaid
classDiagram
    direction LR

    %% Customer lifecycle aggregate
    class Customer {
        <<AggregateRoot>>
        -CustomerId id
        -CustomerStatus status
        -DateTime~Utc~ created_at
        -DateTime~Utc~ updated_at
        -AggregateVersion version
        -Vec~PendingCustomerEvent~ pending_events
        +create(id, now) Customer
        +activate(permit, ctx) Result~ChangeOutcome, CustomerError~
        +suspend(reason, ctx) Result~ChangeOutcome, CustomerError~
        +restore(reason, ctx) Result~ChangeOutcome, CustomerError~
        +block(reason, ctx) Result~ChangeOutcome, CustomerError~
        +reduce_block_to_suspension(reason, ctx) Result~ChangeOutcome, CustomerError~
        +unblock(reason, ctx) Result~ChangeOutcome, CustomerError~
        +delete(reason, ctx) Result~ChangeOutcome, CustomerError~
    }

    class CustomerStatus {
        <<EnumStateMachine>>
        Draft
        Active
        Suspended(Suspension)
        Blocked(Block)
        Deleted(Deletion)
    }

    %% Invariants belong to domain model, not DTO validation.
    %% Aggregate must protect its own local invariants.
    %% No-op commands must not create domain events.
    %% NoChange does not update updated_at or increment AggregateVersion.
    %% NoChange does not emit DomainEvent.
    class ChangeOutcome {
        <<Enum>>
        Changed
        NoChange
    }
    class InvariantViolation {
        <<Enum>>
        LocalInvariantViolation
        CrossAggregateInvariantViolation
    }
    class DomainResult {
        <<RustTypeAlias>>
        Result~ChangeOutcome_DomainError~
    }

    class CustomerInvariants {
        <<Invariants>>
        deleted_is_terminal
        Draft_activates_via_activate
        Suspended_returns_via_restore
        suspend_only_from_Active
        block_only_from_Active_or_Suspended
        reduce_Blocked_to_Suspended_explicitly
        unblock_Blocked_to_Active_only_via_unblock
        lifecycle_change_requires_ActionContext
        critical_change_requires_reason_and_actor
        Changed_updates_updated_at_and_version
        NoChange_updates_nothing_and_emits_nothing
    }

    Customer ..> CustomerInvariants : activate suspend restore block reduce_block unblock delete
    Customer ..> DomainResult : command result
    DomainResult *-- ChangeOutcome
    DomainResult ..> InvariantViolation : maps invariant failures

    class ActionContext {
        <<ValueObject>>
        +ActorRef actor
        +Vec~PrincipalRoleSnapshot~ role_snapshots
        +DateTime~Utc~ now
        +CorrelationId correlation_id
        +Option~EventId~ causation_id
    }

    class ActionContextInvariants {
        <<Invariants>>
        actor_matches_authorization_principal
        role_snapshots_copied_from_authorization_decision
        role_snapshots_are_immutable_historical_evidence
        role_snapshots_are_never_used_for_authorization
        authorization_happens_before_domain_command
        empty_roles_allowed_only_for_System_or_Service
        now_is_domain_event_occurred_at
    }

    class ActorRef {
        <<Enum>>
        Customer(CustomerId)
        Principal(PrincipalId)
        System(SystemActorId)
        Service(ServiceId)
    }

    class ActivationPermit {
        <<ValueObject>>
        +ActivationPermitId id
        +CustomerId customer_id
        +AggregateVersion customer_version
        +AggregateVersion contact_version
        +AggregateVersion consent_version
        +DateTime~Utc~ issued_at
        +DateTime~Utc~ expires_at
        +ActivationEligibilitySnapshot eligibility_snapshot
    }

    class ActivationEligibilitySnapshot {
        <<ValueObject>>
        +bool has_verified_primary_contact
        +bool required_terms_granted
        +bool privacy_policy_granted
        +bool data_processing_granted
        +bool customer_is_in_draft_or_suspended
    }

    Customer *-- CustomerStatus
    Customer ..> ActionContext : command context
    ActionContext ..> ActionContextInvariants : construction rules
    Customer ..> ActivationPermit : activation requires
    ActivationPermit *-- ActivationEligibilitySnapshot : audit evidence

    %% Independent contact aggregate
    class CustomerContactBook {
        <<AggregateRoot>>
        -CustomerId customer_id
        -Option~PhoneContact~ phone
        -Option~EmailContact~ email
        -Option~TelegramContact~ telegram
        -Option~PrimaryContact~ primary
        -AggregateVersion version
        -Vec~PendingContactEvent~ pending_events
        +add_phone(phone, reservation_proof, ctx) Result~ChangeOutcome, ContactError~
        +change_phone(command, reservation_proof, ctx) Result~ChangeOutcome, ContactError~
        +verify_phone(proof, ctx) Result~ChangeOutcome, ContactError~
        +remove_phone(replacement_primary, ctx) Result~ChangeOutcome, ContactError~
        +add_email(email, reservation_proof, ctx) Result~ChangeOutcome, ContactError~
        +change_email(command, reservation_proof, ctx) Result~ChangeOutcome, ContactError~
        +verify_email(proof, ctx) Result~ChangeOutcome, ContactError~
        +remove_email(replacement_primary, ctx) Result~ChangeOutcome, ContactError~
        +link_telegram(link, reservation_proof, ctx) Result~ChangeOutcome, ContactError~
        +unlink_telegram(replacement_primary, ctx) Result~ChangeOutcome, ContactError~
        +change_primary(primary, ctx) Result~ChangeOutcome, ContactError~
    }

    class PhoneContact {
        <<ValueObject>>
        +PhoneNumber number
        +VerificationStatus verification
    }
    class EmailContact {
        <<ValueObject>>
        +EmailAddress address
        +VerificationStatus verification
    }
    class TelegramContact {
        <<ValueObject>>
        +TelegramUserId user_id
        +Option~TelegramUsername~ username
        +DateTime~Utc~ linked_at
        +VerificationStatus verification
        +Option~DateTime_Utc~ last_seen_at
    }
    class PrimaryContact {
        <<ValidatedValueObject>>
        Phone
        Email
        Telegram
    }
    class VerificationStatus {
        <<Enum>>
        Unverified
        Verified(VerifiedAt)
    }

    class IdentityReservationProof {
        <<ImmutableCapabilityValueObject>>
        -IdentityReservationId reservation_id
        -IdentityKey identity_key
        -CustomerId customer_id
        -DateTime~Utc~ expires_at
        -new(reservation) IdentityReservationProof
        +reservation_id() IdentityReservationId
        +identity_key() IdentityKey
        +customer_id() CustomerId
        +expires_at() DateTime~Utc~
    }

    CustomerContactBook *-- PhoneContact
    CustomerContactBook *-- EmailContact
    CustomerContactBook *-- TelegramContact
    CustomerContactBook *-- PrimaryContact
    CustomerContactBook ..> IdentityReservationProof : required for unique identity mutation

    class ContactBookInvariants {
        <<Invariants>>
        primary_references_existing_contact
        primary_removal_requires_replacement
        primary_is_verified_when_required
        phone_or_email_change_resets_verification
        telegram_relink_resets_verification_without_proof
        contact_uniqueness_delegated_to_IdentityRegistry
        global_identity_change_requires_reservation_proof
        reservation_proof_must_match_new_contact
        reservation_proof_must_belong_to_same_customer
        expired_reservation_proof_is_rejected
        reserve_new_identity_before_releasing_old
        failed_reservation_leaves_state_unchanged
        old_identity_is_not_released_before_contactbook_commit
        contactbook_never_claims_or_releases_identity
        failed_contactbook_commit_leaves_registry_reservation_to_expire_or_cancel
        NoChange_emits_nothing_and_preserves_version
        contact_book_never_mutates_customer_lifecycle
    }
    class ContactError {
        <<DomainErrorEnum>>
        ReservationProofRequired
        ReservationProofCustomerMismatch
        ReservationProofIdentityMismatch
        ReservationProofExpired
        PrimaryContactWouldBeMissing
        PrimaryContactMustBeVerified
        ContactNotFound
    }

    CustomerContactBook ..> ContactBookInvariants : add change verify remove contacts and change_primary
    CustomerContactBook ..> DomainResult : command result
    CustomerContactBook ..> ContactError : returns

    %% Independent profile aggregate
    class CustomerProfile {
        <<AggregateRoot>>
        -CustomerId customer_id
        -Option~FullName~ full_name
        -DisplayName display_name
        -Option~AvatarAssetId~ avatar
        -Option~CustomerLocation~ location
        -AggregateVersion version
        -Vec~PendingProfileEvent~ pending_events
        +change_display_name(value, ctx) DomainResult
        +change_full_name(value, ctx) DomainResult
        +change_avatar(asset_id, ctx) DomainResult
        +change_location(location, ctx) DomainResult
    }
    class CustomerLocation {
        <<ValueObject>>
        +CityName city
        +Option~CountryCode~ country
        +Option~RegionCode~ region
        +Option~GeoId~ geo_id
    }
    CustomerProfile *-- CustomerLocation

    class ProfileInvariants {
        <<Invariants>>
        display_name_is_required_and_valid
        full_name_is_optional
        avatar_contains_only_AvatarAssetId
        location_is_optional
        city_name_is_normalized_and_valid
        geo_id_disambiguates_location
        changes_emit_granular_profile_events
        NoChange_emits_nothing_and_preserves_version
    }

    CustomerProfile ..> ProfileInvariants : change_display_name change_full_name change_avatar change_location
    CustomerProfile ..> DomainResult : command result

    %% Independent preferences aggregate
    %% CommunicationPreferences stores user intent only.
    %% Contact availability is checked by communication/application layer using ContactBook projection.
    %% Preferences aggregate never reads CustomerContactBook.
    class CustomerPreferences {
        <<AggregateRoot>>
        -CustomerId customer_id
        -LanguageCode language
        -IanaTimezone timezone
        -NotificationPreferences notifications
        -CommunicationPreferences communication
        -AggregateVersion version
        -Vec~PendingPreferencesEvent~ pending_events
        +change_language(language, ctx) Result~ChangeOutcome_PreferencesError~
        +change_timezone(timezone, ctx) Result~ChangeOutcome_PreferencesError~
        +update_notification_preferences(preferences, ctx) Result~ChangeOutcome_PreferencesError~
        +update_communication_preferences(preferences, ctx) Result~ChangeOutcome_PreferencesError~
        +set_quiet_hours(quiet_hours, ctx) Result~ChangeOutcome_PreferencesError~
        +clear_quiet_hours(ctx) Result~ChangeOutcome_PreferencesError~
        +change_preferred_channel(channel, ctx) Result~ChangeOutcome_PreferencesError~
        +change_allowed_channels(channels, ctx) Result~ChangeOutcome_PreferencesError~
    }
    class NotificationPreferences {
        <<ValueObject>>
        +bool booking_reminders
        +bool repair_status_updates
        +bool marketing_messages
        +bool chat_messages
        +bool garage_offers
        +bool vehicle_service_reminders
    }
    class CommunicationPreferences {
        <<ValueObject>>
        +Set~ContactMethod~ allowed_channels
        +Option~ContactMethod~ preferred_channel
        +Option~QuietHours~ quiet_hours
        +MessagePriorityPolicy priority_policy
    }
    class ContactMethod {
        <<Enum>>
        communication_channel_not_login_identity
        Phone
        Email
        Telegram
        InApp
    }
    class QuietHours {
        <<ValidatedValueObject>>
        +LocalTime start
        +LocalTime end
        +DstResolutionPolicy dst_policy
    }
    class DstResolutionPolicy {
        <<Enum>>
        SkipAmbiguousLocalTime
        PreferEarlierOffset
        PreferLaterOffset
        UseUtcFallback
    }
    class MessagePriorityPolicy {
        <<Enum>>
        TransactionalCanBypassQuietHours
        CriticalCanBypassQuietHours
        NothingBypassesQuietHours
    }
    class PreferencesError {
        <<DomainErrorEnum>>
        InvalidLanguageCode
        InvalidTimezone
        InvalidQuietHours
        PreferredChannelNotAllowed
        EmptyAllowedChannels
        CommunicationChannelUnavailable
        MarketingConsentRequiredAtDispatchTime
        NoChange
    }
    class PreferencesEvent {
        <<TypedDomainEvent>>
        LanguageChangedV1
        TimezoneChangedV1
        NotificationPreferencesChangedV1
        CommunicationPreferencesChangedV1
        QuietHoursSetV1
        QuietHoursClearedV1
        PreferredChannelChangedV1
        AllowedChannelsChangedV1
    }
    class PendingPreferencesEvent {
        <<ImmutableDomainEventRecord>>
        +PreferencesEvent payload
        +DateTime~Utc~ occurred_at
        +ActorRef actor
        +Vec~PrincipalRoleSnapshot~ role_snapshots
        +CorrelationId correlation_id
        +Option~EventId~ causation_id
    }
    CustomerPreferences *-- NotificationPreferences
    CustomerPreferences *-- CommunicationPreferences
    CommunicationPreferences *-- QuietHours
    CommunicationPreferences *-- ContactMethod
    CommunicationPreferences *-- MessagePriorityPolicy
    QuietHours *-- DstResolutionPolicy
    CustomerPreferences *-- "0..*" PendingPreferencesEvent : raises
    PendingPreferencesEvent *-- PreferencesEvent
    PendingPreferencesEvent ..> ActionContext : copies audit fields at creation
    CustomerPreferences ..> PreferencesError : returns

    class PreferencesInvariants {
        <<Invariants>>
        language_code_is_valid
        iana_timezone_is_valid
        allowed_channels_is_non_empty
        preferred_channel_is_allowed
        quiet_hours_has_valid_local_time_range
        quiet_hours_defines_DST_resolution_policy
        notifications_express_intent_not_legal_permission
        communication_expresses_intent_not_contact_verification
        preferences_never_read_contact_book
        preferences_never_read_or_mutate_consent_ledger
        marketing_permission_is_checked_at_dispatch_time
        NoChange_emits_nothing_and_preserves_version
    }

    CustomerPreferences ..> PreferencesInvariants : language timezone notification communication quiet_hours channel methods
    CustomerPreferences ..> DomainResult : command result

    %% Consent bounded context
    class CustomerConsentLedger {
        <<SeparateAggregateRoot>>
        -CustomerId customer_id
        -Map~ConsentKind_EffectiveConsent~ effective
        -AggregateVersion version
        -Vec~PendingConsentEvent~ pending_events
        +grant_consent(grant, ctx) DomainResult
        +revoke_consent(kind, revocation, ctx) DomainResult
    }
    class ConsentKind {
        <<Enum>>
        TermsOfService
        PrivacyPolicy
        DataProcessing
        Marketing
    }
    class EffectiveConsent {
        <<EnumStateMachine>>
        NotGranted
        Granted(ConsentGrant)
        Revoked(ConsentGrant, ConsentRevocation)
    }
    CustomerConsentLedger *-- EffectiveConsent
    CustomerConsentLedger *-- ConsentKind

    class ConsentLedgerInvariants {
        <<Invariants>>
        grant_has_document_version_source_time_and_actor
        revocation_preserves_grant_history
        revoked_is_never_effectively_granted
        marketing_is_optional_and_independent
        effective_consent_has_no_impossible_state
        ledger_is_append_and_audit_friendly
        NoChange_emits_nothing_and_preserves_version
    }

    CustomerConsentLedger ..> ConsentLedgerInvariants : grant_consent revoke_consent
    CustomerConsentLedger ..> DomainResult : command result

    %% Event model
    %% AccessContext authorizes current action.
    %% AuthorizationDecision is transient unless explicitly persisted.
    %% ActionContext carries audit context into domain command.
    %% Aggregate never calls AccessContext.
    %% Aggregate raises immutable PendingEvents, not EventEnvelopes.
    %% EventEnvelope is created at persistence boundary.
    %% Role snapshots are historical evidence, not live permissions.
    %% Role snapshots are immutable historical evidence captured at event creation.
    %% They never participate in authorization; current permissions come from AccessContext.
    %% New roles require new RoleKey values, not EventEnvelope or ActorRef changes.
    class CustomerEvent {
        <<TypedDomainEvent>>
        CustomerCreatedV1
        CustomerActivatedV1
        CustomerSuspendedV1
        CustomerRestoredV1
        CustomerBlockedV1
        CustomerBlockReducedToSuspensionV1
        CustomerUnblockedV1
        CustomerDeletedV1
    }
    class ContactEvent {
        <<TypedDomainEvent>>
        PhoneAddedV1
        PhoneChangedV1
        PhoneRemovedV1
        EmailAddedV1
        EmailChangedV1
        EmailRemovedV1
        TelegramLinkedV1
        TelegramUnlinkedV1
        PrimaryContactChangedV1
    }
    class EventEnvelope {
        <<EventStoreEnvelope>>
        +EventId event_id
        +AggregateId aggregate_id
        +AggregateVersion aggregate_version
        +EventType event_type
        +EventSchemaVersion schema_version
        +DateTime~Utc~ occurred_at
        +ActorRef actor
        +Vec~PrincipalRoleSnapshot~ principal_role_snapshots
        +CorrelationId correlation_id
        +Option~EventId~ causation_id
        +TypedPayload payload
    }
    class PendingCustomerEvent {
        <<ImmutableDomainEventRecord>>
        +CustomerEvent payload
        +DateTime~Utc~ occurred_at
        +ActorRef actor
        +Vec~PrincipalRoleSnapshot~ role_snapshots
        +CorrelationId correlation_id
        +Option~EventId~ causation_id
    }
    class PendingContactEvent {
        <<ImmutableDomainEventRecord>>
        +ContactEvent payload
        +DateTime~Utc~ occurred_at
        +ActorRef actor
        +Vec~PrincipalRoleSnapshot~ role_snapshots
        +CorrelationId correlation_id
        +Option~EventId~ causation_id
    }
    class EventEnvelopeFactory {
        <<PersistenceBoundaryFactory>>
        +wrap_pending_event(aggregate_id, aggregate_version, pending_event) EventEnvelope
        +wrap_pending_events(aggregate_id, first_version, pending_events) Vec~EventEnvelope~
    }
    class PrincipalRoleSnapshot {
        <<ImmutableValueObject>>
        +RoleId role_id
        +RoleKey role_key
        +Option~RoleDisplayName~ display_name
    }
    class RoleId {
        <<TypeSafeNewtype>>
        +Uuid value
    }
    class RoleKey {
        <<OpenValueObject>>
        +String value
    }
    class RoleDisplayName {
        <<ValueObject>>
        +String value
    }
    class EventInvariants {
        <<Invariants>>
        event_is_immutable_after_creation
        role_snapshots_copied_at_command_time
        role_changes_never_rewrite_old_events
        role_snapshot_is_historical_evidence_only
        authorization_uses_current_AccessContext
        audit_is_understandable_without_external_lookup
        role_key_is_open_not_closed_enum
        envelope_is_created_outside_aggregate
        aggregate_raises_pending_events_only
        event_id_is_assigned_outside_aggregate
        schema_version_is_assigned_by_envelope_factory
        envelope_aggregate_version_matches_persisted_version
        occurred_at_comes_from_ActionContext_now
        actor_and_role_snapshots_are_preserved
    }
    Customer ..> CustomerEvent : raises
    Customer *-- "0..*" PendingCustomerEvent : records immutable copy
    CustomerContactBook *-- "0..*" PendingContactEvent : records immutable copy
    PendingCustomerEvent *-- CustomerEvent : typed payload
    PendingContactEvent *-- ContactEvent : typed payload
    PendingCustomerEvent ..> ActionContext : copies audit fields at creation
    PendingContactEvent ..> ActionContext : copies audit fields at creation
    EventEnvelopeFactory --> PendingCustomerEvent : wraps
    EventEnvelopeFactory --> PendingContactEvent : wraps
    EventEnvelopeFactory --> PendingPreferencesEvent : wraps
    EventEnvelopeFactory --> EventEnvelope : creates
    EventEnvelopeFactory ..> EventInvariants : preserves audit fields and assigns metadata
    EventEnvelope *-- CustomerEvent : wraps
    EventEnvelope *-- ContactEvent : wraps
    EventEnvelope *-- "0..*" PrincipalRoleSnapshot : captures at event creation
    PrincipalRoleSnapshot *-- RoleId
    PrincipalRoleSnapshot *-- RoleKey
    PrincipalRoleSnapshot *-- RoleDisplayName
    EventEnvelope ..> EventInvariants : append and replay rules

    %% Customer aggregate never reads other aggregates.
    %% CustomerActivationPolicy validates eligibility and issues permit only.
    class CustomerActivationPolicy {
        <<DomainService>>
        +issue_permit(customer_snapshot, contact_snapshot, consent_snapshot, now) Result~ActivationPermit, EligibilityError~
    }
    class CustomerActivationSnapshot {
        <<ReadOnlyDomainSnapshot>>
        +CustomerId customer_id
        +CustomerStatusKind status
        +AggregateVersion version
    }
    class ContactEligibilitySnapshot {
        <<ReadOnlyDomainSnapshot>>
        +CustomerId customer_id
        +bool has_verified_primary_contact
        +AggregateVersion version
    }
    class ConsentEligibilitySnapshot {
        <<ReadOnlyDomainSnapshot>>
        +CustomerId customer_id
        +bool required_terms_granted
        +bool privacy_policy_granted
        +bool data_processing_granted
        +AggregateVersion version
    }

    %% Cross-aggregate invariants are enforced by ApplicationService / Policy / UnitOfWork.
    class ActivationInvariants {
        <<CrossAggregateInvariants>>
        activate_requires_ActivationPermit
        permit_customer_id_matches_customer
        permit_is_not_expired
        permit_customer_version_matches_customer
        required_consents_granted_before_activation
        active_candidate_has_verified_primary_contact
        contact_and_consent_versions_checked_during_commit
        eligibility_snapshot_is_audit_evidence_not_lock
        customer_never_reads_contact_book_or_consent_ledger
    }

    %% Application layer is responsible for orchestration.
    %% It loads aggregates, invokes the policy, invokes Customer.activate,
    %% executes UnitOfWork and commits the consistency boundary.
    class CustomerActivationApplicationService {
        <<ApplicationService>>
        +activate(command) Result~ChangeOutcome, ActivationApplicationError~
    }

    %% Contact application orchestrates normalize -> reserve -> load -> domain change.
    %% UnitOfWork saves ContactBook and outbox records with optimistic locking.
    %% Outbox dispatches claim before release only after a successful commit.
    %% Failed commit leaves the reservation to explicit cancel or TTL expiry.
    class CustomerContactApplicationService {
        <<ApplicationService>>
        +remove_contact(command) Result~ChangeOutcome, ContactApplicationError~
        +add_or_replace_identity(command) Result~ChangeOutcome, ContactApplicationError~
    }
    class ContactLifecyclePolicy {
        <<CrossAggregateDomainPolicy>>
        +can_remove_contact(customer_snapshot, contact_snapshot, replacement) Result~void, CrossAggregateInvariantViolation~
        Draft_may_exist_without_verified_contact
        Active_cannot_lose_last_or_verified_primary_contact
    }
    class CustomerRepository {
        <<Port>>
        +load(customer_id) Versioned~Customer~
        +save(customer, expected_version) Result~void, OptimisticLockFailed~
    }
    class CustomerContactBookRepository {
        <<Port>>
        +load(customer_id) Versioned~CustomerContactBook~
        +save(contact_book, expected_version) Result~void, OptimisticLockFailed~
    }
    class CustomerConsentLedgerRepository {
        <<Port>>
        +load(customer_id) Versioned~CustomerConsentLedger~
    }

    %% ActivationPermit alone does not solve race conditions.
    %% UnitOfWork is responsible for consistency and optimistic locking.
    %% Aggregate boundaries must not be violated.
    class UnitOfWork {
        <<ApplicationTransactionBoundary>>
        +begin() Transaction
        +assert_expected_version(aggregate_ref, expected) Result~void, ConsistencyError~
        +save(entity, expected_version) Result~void, ConsistencyError~
        +commit() Result~void, ConsistencyError~
        +rollback()
    }
    class AggregateRef {
        <<ValueObject>>
        +AggregateType aggregate_type
        +AggregateId aggregate_id
    }
    class VersionedEntity {
        <<ApplicationAbstraction>>
        +AggregateRef aggregate_ref
        +AggregateVersion version
    }
    class OptimisticLock {
        <<ConsistencyMechanism>>
        compare_and_set_expected_version
        serializable_version_predicate
        hold_until_commit
    }

    %% Read-side rule: preferences do not synchronously read ConsentLedger or ContactBook.
    class CustomerCommunicationEligibilityProjection {
        <<ReadModelProjection>>
        +CustomerId customer_id
        +bool marketing_preference_enabled
        +bool marketing_consent_granted
        +IanaTimezone timezone
        +Option~QuietHours~ quiet_hours
        +MessagePriorityPolicy priority_policy
        +Set~ContactMethod~ allowed_channels
        +Option~ContactMethod~ preferred_channel
    }
    class CustomerContactAvailabilityProjection {
        <<ReadModelProjection>>
        +CustomerId customer_id
        +bool has_verified_phone
        +bool has_verified_email
        +bool has_verified_telegram
        +bool has_in_app_channel
    }
    class NotificationDispatchPolicy {
        <<ReadSidePolicy>>
        +can_send_marketing(eligibility, contacts, now) bool
        +can_send_transactional(eligibility, contacts, now) bool
        marketing_requires_preference_and_granted_consent
        selected_channel_must_be_allowed_and_available
        quiet_hours_use_customer_timezone_and_DST_policy
        priority_policy_controls_quiet_hours_bypass
    }

    class CustomerActivationError {
        <<DomainError>>
        ActivationPermitExpired
        ActivationPermitCustomerMismatch
        ActivationPermitCustomerVersionMismatch
        CustomerStatusDoesNotAllowActivation
    }
    class CustomerError {
        <<DomainErrorEnum>>
        Activation(CustomerActivationError)
        OtherCustomerDomainError
    }
    class EligibilityError {
        <<DomainPolicyError>>
        ActivationEligibilityNotSatisfied
    }
    class ActivationApplicationError {
        <<ApplicationErrorEnum>>
        Domain(CustomerActivationError)
        Eligibility(EligibilityError)
        StaleContactBookVersion
        StaleConsentLedgerVersion
        OptimisticLockFailed
    }
    %% IdentityRegistry owns global uniqueness.
    %% CustomerContactBook owns customer-local contact state.
    %% ReservationProof allows ContactBook to validate intent without reading IdentityRegistry.
    %% ReservationProof is an internal immutable capability with a private constructor;
    %% it is not signed because it never crosses an untrusted boundary.
    %% New identity is reserved before ContactBook mutation.
    %% Old identity is released only after ContactBook commit.
    %% Failed commit does not corrupt IdentityRegistry; reservation expires by TTL.
    %% Outbox makes claim/release eventually consistent and retryable.
    %% IdentityRegistry operations must be idempotent.
    class IdentityRegistry {
        <<SeparateBoundedContextDomainService>>
        +reserve(identity_key, customer_id, ttl, ctx) Result~IdentityReservation, IdentityRegistryError~
        +claim(reservation_id, customer_id, ctx) Result~ClaimedIdentity, IdentityRegistryError~
        +release(identity_key, customer_id, ctx) Result~ChangeOutcome, IdentityRegistryError~
        +cancel_reservation(reservation_id, ctx) Result~ChangeOutcome, IdentityRegistryError~
        +expire_reservations(now) Vec~IdentityReservationExpiredEvent~
    }
    class IdentityRegistryEntry {
        <<AggregateRootPerIdentityKey>>
        -IdentityKey identity_key
        -Option~CustomerId~ owner_customer_id
        -IdentityRegistryStatus status
        -Option~IdentityReservationId~ reservation_id
        -Option~DateTime_Utc~ reserved_until
        -AggregateVersion version
        -DateTime~Utc~ updated_at
    }
    class IdentityRegistryEntryRepository {
        <<Port>>
        +load_for_update(identity_key) Versioned~IdentityRegistryEntry~
        +save(entry, expected_version) Result~void, OptimisticLockFailed~
        +find_expired(now, limit) Vec~IdentityKey~
    }
    class IdentityKey {
        <<ValueObject>>
        +IdentityKind kind
        +NormalizedIdentityValue normalized_value
    }
    class IdentityKind {
        <<Enum>>
        Phone
        Email
        TelegramUserId
    }
    class NormalizedIdentityValue {
        <<ValidatedNewtype>>
        +String value
    }
    class IdentityRegistryStatus {
        <<Enum>>
        Available
        Reserved
        Claimed
    }
    class IdentityReservation {
        <<ImmutableValueObject>>
        +IdentityReservationId id
        +IdentityKey identity_key
        +CustomerId customer_id
        +DateTime~Utc~ reserved_at
        +DateTime~Utc~ expires_at
        +IdentityReservationStatus status
        +proof() IdentityReservationProof
    }
    class IdentityReservationId {
        <<TypeSafeNewtype>>
        +Uuid value
    }
    class IdentityReservationStatus {
        <<Enum>>
        Active
        Claimed
        Released
        Expired
        Cancelled
    }
    class ClaimedIdentity {
        <<ValueObject>>
        +IdentityKey identity_key
        +CustomerId owner_customer_id
        +DateTime~Utc~ claimed_at
    }
    class IdentityNormalizer {
        <<DomainService>>
        +normalize(raw_identity) Result~IdentityKey, IdentityRegistryError~
    }
    class IdentityRegistryInvariants {
        <<Invariants>>
        identity_key_is_normalized
        only_one_active_claim_per_identity
        only_one_active_reservation_per_identity
        reservation_has_TTL
        expired_reservation_cannot_be_claimed
        claimed_identity_cannot_be_reserved_by_another_customer
        customer_can_reclaim_own_identity_idempotently
        release_is_idempotent_for_same_customer
        release_must_not_release_identity_owned_by_another_customer
        claim_requires_active_matching_reservation
        reservation_expiry_does_not_mutate_CustomerContactBook
    }
    class IdentityRegistryError {
        <<DomainErrorEnum>>
        IdentityAlreadyClaimed
        IdentityAlreadyReserved
        ReservationExpired
        ReservationCustomerMismatch
        ReservationIdentityMismatch
        CannotReleaseIdentityOwnedByAnotherCustomer
        ReservationNotFound
        RegistryOptimisticLockFailed
    }

    class IdentityOutboxCommand {
        <<Enum>>
        ClaimReservedIdentity(reservation_id, customer_id)
        ReleasePreviousIdentity(identity_key, customer_id)
        CancelReservation(reservation_id, customer_id)
    }
    class IdentityOutbox {
        <<TransactionalOutboxPort>>
        +enqueue(command) Result~void, OutboxError~
        +retry_until_success(command) Result~void, OutboxError~
        +dead_letter_after_policy(command) Result~void, OutboxError~
    }
    class IdentityOutboxDispatcher {
        <<ApplicationWorker>>
        +dispatch_pending(now, limit) Result~DispatchSummary, OutboxError~
    }
    class OutboxInvariants {
        <<ConsistencyInvariants>>
        outbox_commands_created_only_after_successful_domain_change
        release_previous_identity_is_eventually_consistent
        claim_reserved_identity_must_be_idempotent
        release_previous_identity_must_be_idempotent
        duplicate_outbox_delivery_is_safe
        commands_are_persisted_atomically_with_contact_book_commit
    }
    class ContactApplicationError {
        <<ApplicationErrorEnum>>
        IdentityRegistry(IdentityRegistryError)
        ContactDomain(ContactError)
        OptimisticLockFailed
        ReservationProofInvalid
        OutboxEnqueueFailed
    }

    class IdentityRegistryEvent {
        <<TypedDomainEvent>>
        IdentityReservedV1
        IdentityClaimedV1
        IdentityReleasedV1
        IdentityReservationCancelledV1
        IdentityReservationExpiredV1
    }
    class IdentityReservationExpiredEvent {
        <<DomainEvent>>
        +IdentityReservationId reservation_id
        +IdentityKey identity_key
        +CustomerId customer_id
        +DateTime~Utc~ occurred_at
    }
    class AccessContext {
        <<SeparateBoundedContext>>
        +authorize(principal, action, customer_id) AuthorizationDecision
    }
    class AuthorizationDecision {
        <<TransientValueObject>>
        +PrincipalId principal_id
        +RequestedAction action
        +AuthorizationOutcome outcome
        +Vec~PrincipalRoleSnapshot~ role_snapshots
        +DateTime~Utc~ decided_at
    }
    class AuthorizationOutcome {
        <<Enum>>
        Allowed
        Denied
    }
    class ActionContextFactory {
        <<ApplicationFactory>>
        +from_authorization(decision, now, correlation_id, causation_id) Result~ActionContext, ContextError~
        +for_trusted_actor(system_or_service, now, correlation_id, causation_id) Result~ActionContext, ContextError~
        validates_outcome_is_Allowed
        maps_PrincipalId_to_ActorRef
        copies_role_snapshots
        trusted_actor_must_be_System_or_Service
    }
    class AuthContext {
        <<SeparateBoundedContext>>
        +AuthAccount
        +LoginIdentity
        +Credentials
        +MfaState
        +TrustedDevices
    }
    class IdentityAuditProjection {
        <<TemporalReadModel>>
        +PreviousPhoneContact
        +PreviousEmailContact
        +PreviousTelegramContact
    }
    class IdentityRegistryAuditProjection {
        <<TemporalReadModel>>
        +ReservationHistory
        +ClaimHistory
        +ReleaseHistory
    }

    IdentityRegistry --> IdentityRegistryEntry : loads and changes entry
    IdentityRegistry --> IdentityRegistryEntryRepository : persists with uniqueness constraint
    IdentityRegistry --> IdentityNormalizer : canonicalizes identity
    IdentityRegistryEntry *-- IdentityKey
    IdentityKey *-- IdentityKind
    IdentityKey *-- NormalizedIdentityValue
    IdentityRegistryEntry *-- IdentityRegistryStatus
    IdentityRegistryEntry ..> IdentityReservation : creates reservation
    IdentityReservation *-- IdentityReservationId
    IdentityReservation *-- IdentityReservationStatus
    IdentityRegistry ..> ClaimedIdentity : returns from claim
    IdentityReservation --> IdentityReservationProof : creates via private constructor
    IdentityRegistry ..> IdentityRegistryInvariants : reserve claim release cancel expire
    IdentityRegistry ..> IdentityRegistryError : returns
    IdentityRegistry ..> IdentityRegistryEvent : raises
    IdentityRegistry ..> IdentityReservationExpiredEvent : raises on TTL expiry
    IdentityOutbox *-- IdentityOutboxCommand
    IdentityOutbox ..> OutboxInvariants : durable delivery rules
    IdentityOutboxDispatcher --> IdentityOutbox : polls and updates
    IdentityOutboxDispatcher --> IdentityRegistry : executes idempotent commands

    CustomerActivationPolicy ..> CustomerActivationSnapshot : evaluates
    CustomerActivationPolicy ..> ContactEligibilitySnapshot : evaluates
    CustomerActivationPolicy ..> ConsentEligibilitySnapshot : evaluates
    CustomerActivationPolicy --> ActivationPermit : issues
    CustomerActivationPolicy ..> EligibilityError : returns
    CustomerActivationPolicy ..> ActivationInvariants : required consents and verified primary
    Customer ..> CustomerError : returns
    CustomerError *-- CustomerActivationError : wraps
    Customer ..> ActivationInvariants : validates permit locally in activate
    ActivationPermit ..> ActivationInvariants : carries eligibility evidence

    CustomerActivationApplicationService --> CustomerRepository : load and save
    CustomerActivationApplicationService --> CustomerContactBookRepository : load
    CustomerActivationApplicationService --> CustomerConsentLedgerRepository : load
    CustomerActivationApplicationService --> CustomerActivationPolicy : issue permit
    CustomerActivationApplicationService --> AccessContext : authorize current action
    CustomerActivationApplicationService --> ActionContextFactory : create command and audit context
    CustomerActivationApplicationService --> Customer : invoke activate with prepared context
    CustomerActivationApplicationService --> UnitOfWork : commit consistency boundary
    CustomerActivationApplicationService ..> ActivationApplicationError : returns
    CustomerActivationApplicationService ..> ActivationInvariants : orchestrates cross aggregate checks
    UnitOfWork --> OptimisticLock : enforces
    UnitOfWork --> EventEnvelopeFactory : creates envelopes at persistence boundary
    UnitOfWork --> IdentityOutbox : persists identity commands atomically
    UnitOfWork ..> AggregateRef : asserts version for
    UnitOfWork ..> VersionedEntity : saves
    UnitOfWork ..> ActivationInvariants : enforces source versions at commit

    CustomerContactApplicationService --> CustomerRepository : loads lifecycle snapshot
    CustomerContactApplicationService --> CustomerContactBookRepository : loads and saves contact book
    CustomerContactApplicationService --> ContactLifecyclePolicy : checks Active customer rule
    CustomerContactApplicationService --> UnitOfWork : commits contact replacement
    CustomerContactApplicationService --> IdentityNormalizer : normalizes requested identity
    CustomerContactApplicationService --> IdentityRegistry : reserves identity before mutation
    CustomerContactApplicationService --> IdentityOutbox : atomically enqueues claim and release
    CustomerContactApplicationService --> IdentityReservationProof : passes proof to contact book
    CustomerContactApplicationService ..> ContactApplicationError : maps use case failures
    ContactLifecyclePolicy ..> CustomerActivationSnapshot : reads lifecycle snapshot
    ContactLifecyclePolicy ..> ContactEligibilitySnapshot : reads contact snapshot
    ContactLifecyclePolicy ..> ContactBookInvariants : removing last contact when Active is cross aggregate

    CustomerCommunicationEligibilityProjection ..> CustomerPreferences : projects preferences events
    CustomerCommunicationEligibilityProjection ..> CustomerConsentLedger : projects consent events
    CustomerContactAvailabilityProjection ..> CustomerContactBook : projects verified contact events
    NotificationDispatchPolicy --> CustomerCommunicationEligibilityProjection : reads intent consent and delivery timing
    NotificationDispatchPolicy --> CustomerContactAvailabilityProjection : reads verified channel availability

    CustomerContactBook ..> IdentityReservationProof : validates reservation intent locally
    IdentityOutbox --> IdentityRegistry : eventually claims or releases
    AccessContext --> AuthorizationDecision : evaluates current permissions
    AuthorizationDecision --> ActionContextFactory : transient input
    AuthorizationDecision *-- AuthorizationOutcome
    ActionContextFactory --> ActionContext : creates immutable context
    ActionContextFactory ..> ActionContextInvariants : validates construction
    AuthorizationDecision ..> PrincipalRoleSnapshot : supplies audit snapshots
    PrincipalRoleSnapshot ..> AccessContext : never used for permission checks
    AuthContext ..> Customer : references CustomerId only
    IdentityAuditProjection ..> ContactEvent : projects typed contact changes
    IdentityAuditProjection ..> EventEnvelope : reads event metadata
    IdentityRegistryAuditProjection ..> IdentityRegistryEvent : projects registry events
```

### Activation consistency responsibilities

`CustomerActivationPolicy` получает уже загруженные read-only snapshots. Она
проверяет verified primary contact, обязательные consent и допустимость lifecycle
candidate, после чего выдает `ActivationPermit` либо
`ActivationEligibilityNotSatisfied`. Policy ничего не сохраняет и не вызывает
`Customer.activate`.

Поле `customer_is_in_draft_or_suspended` фиксирует результат общей eligibility
проверки. Оно не расширяет state machine Customer. В текущей v2-модели
`activate` разрешен из `Draft`, а восстановление из `Suspended` остается
отдельной командой `restore`; при необходимости она может использовать permit
того же формата, но создает другой domain event.

`Customer.activate` проверяет только локальные условия: совпадение
`customer_id`, срок permit, `customer_version`, допустимость текущего статуса и
семантические ограничения actor. Он не читает `CustomerContactBook` и
`CustomerConsentLedger` и не может проверить их актуальные версии.

`CustomerActivationApplicationService` загружает три агрегата и передает их
snapshots в policy. Затем внутри `UnitOfWork` вызывает `Customer.activate`,
повторно утверждает `contact_version` и `consent_version`, сохраняет Customer с
expected version и делает commit. Version assertions должны удерживаться до
commit через serializable transaction, predicate locks или эквивалентный
storage mechanism. Последовательность «сравнить версии, затем отдельно
сохранить» вне transaction boundary гонку не закрывает.

`ActivationEligibilitySnapshot` нужен для audit/debug и содержит только
логические результаты проверки. Версии находятся только в `ActivationPermit`.
Snapshot не является optimistic-lock token и не заменяет version assertions.

`PermitSignature` отсутствует намеренно. Permit создается policy через private
constructor и используется внутри одного доверенного application workflow. Если
в будущем permit начнет храниться или пересекать недоверенную границу, подпись
следует добавить как обязательный immutable token, а не как `Option`.

Domain errors: `ActivationPermitExpired`,
`ActivationPermitCustomerMismatch`,
`ActivationPermitCustomerVersionMismatch` и
`CustomerStatusDoesNotAllowActivation`. Policy возвращает
`ActivationEligibilityNotSatisfied`. Application/consistency errors:
`StaleContactBookVersion`, `StaleConsentLedgerVersion` и
`OptimisticLockFailed`.

## Architectural rationale

1. Внутри permit artifact expected versions хранятся только в
   `ActivationPermit`, потому что он является concurrency contract между
   проверкой eligibility и commit. Read-only source snapshots передают policy
   фактические версии загруженных агрегатов, но итоговый eligibility snapshot их
   не дублирует. Единственный набор expected versions исключает противоречие.
2. `ActivationEligibilitySnapshot` фиксирует только бизнес-факты, проверенные
   policy. Он нужен для audit/debug, но не участвует в optimistic locking и не
   должен дублировать версии.
3. `UnitOfWork` оперирует универсальными `AggregateRef`, `VersionedEntity` и
   `AggregateVersion`. Customer-specific проверки принадлежат application
   workflow, а transaction abstraction остается пригодной для других use cases.
4. `Customer` не читает другие агрегаты, потому что aggregate root защищает
   только собственную consistency boundary. Cross-aggregate чтение внутри
   domain method нарушило бы независимость repository и transaction semantics.
5. Orchestration живет в Application Layer: он загружает агрегаты, вызывает
   domain policy и Customer, сопоставляет версии permit с актуальными versions и
   атомарно завершает UnitOfWork.

### Identity reservation and release

1. `IdentityRegistry` является отдельным bounded context, потому что глобальная
   уникальность phone/email/Telegram не является локальным инвариантом одного
   `CustomerContactBook`. Физической consistency boundary служит
   `IdentityRegistryEntry` на один нормализованный `IdentityKey`, а не один
   глобальный registry aggregate, который стал бы hotspot.
2. `CustomerContactBook` владеет только customer-local contact state. Он не
   способен доказать глобальную уникальность без чтения чужого aggregate, поэтому
   принимает `IdentityReservationProof` и проверяет только совпадение
   `customer_id`, нормализованного key и срока действия.
3. Proof является immutable capability с private constructor и создается из
   успешной `IdentityReservation`. Подпись сознательно не используется, пока
   proof передается только внутри доверенной application/domain boundary. Если
   artifact выйдет во внешний процесс или клиент, signature должна стать
   обязательной, а не `Option`.
4. Replacement flow: application service нормализует новую identity, резервирует
   ее с TTL, загружает ContactBook, вызывает domain method с proof и сохраняет
   aggregate по expected version. Записи `ClaimReservedIdentity` и
   `ReleasePreviousIdentity` создаются после успешного domain change и
   сохраняются в transactional outbox атомарно с ContactBook commit; исполняются
   только после commit, причем claim должен предшествовать release.
5. Старая identity остается `Claimed` до подтвержденного ContactBook commit.
   Поэтому rollback или optimistic-lock conflict не освобождает identity, которую
   Customer продолжает использовать. Если release не выполнен, outbox повторяет
   его, а старая identity остается безопасно занятой.
6. TTL очищает reservation после failed commit, crash до outbox commit или
   явной отмены. TTL не меняет ContactBook. Claim dispatcher должен укладываться
   в reservation lease; истекший claim считается consistency incident и не
   должен автоматически освобождать старую identity.
7. `reserve` для того же Customer, `claim`, `release`, `cancel_reservation` и
   доставка каждой outbox command должны быть idempotent. Команда имеет стабильный
   idempotency key, поэтому at-least-once delivery не создает повторный переход и
   не увеличивает aggregate version при `NoChange`.
8. В Rust `IdentityKey`, `IdentityReservationId`, `NormalizedIdentityValue` и
   `AggregateVersion` реализуются newtype/value object; registry и reservation
   status являются исчерпывающими enum state machines. Domain API возвращает
   `Result<ChangeOutcome, IdentityRegistryError>`, repository ports применяют
   optimistic locking, а application service координирует repository,
   `UnitOfWork` и transactional outbox без прямой зависимости агрегатов друг от
   друга и без обязательной distributed transaction.

Минимальный Rust-контракт выглядит так:

```rust
pub struct IdentityReservationProof {
    reservation_id: IdentityReservationId,
    identity_key: IdentityKey,
    customer_id: CustomerId,
    expires_at: DateTime<Utc>,
}

pub enum IdentityRegistryStatus {
    Available,
    Reserved {
        reservation_id: IdentityReservationId,
        customer_id: CustomerId,
        expires_at: DateTime<Utc>,
    },
    Claimed { customer_id: CustomerId },
}

pub trait IdentityRegistryRepository {
    fn load_for_update(
        &mut self,
        key: &IdentityKey,
    ) -> Result<Versioned<IdentityRegistryEntry>, RepositoryError>;

    fn save(
        &mut self,
        entry: &IdentityRegistryEntry,
        expected: AggregateVersion,
    ) -> Result<(), OptimisticLockFailed>;
}
```

### Invariant enforcement timing

- **Synchronously inside aggregate.** `CustomerInvariants`, локальная часть
  `ContactBookInvariants`, `ConsentLedgerInvariants`, `ProfileInvariants` и
  `PreferencesInvariants` проверяются private state и именованными domain
  methods до изменения состояния. Нарушение возвращает domain error; поля
  остаются private. ContactBook синхронно проверяет reservation proof, а
  `IdentityRegistryEntry` синхронно защищает uniqueness и reservation state
  machine.
- **Before command execution.** Cross-aggregate eligibility проверяется domain
  policies на immutable snapshots, загруженных Application Service. Сюда
  относятся required consent, verified primary contact и запрет удаления
  последнего контакта активного Customer. Для identity change application
  service сначала нормализует и резервирует новый `IdentityKey` с TTL.
- **During commit.** `UnitOfWork` повторно проверяет expected versions из permit,
  применяет optimistic locking и атомарно сохраняет измененный aggregate. Для
  ContactBook тот же commit атомарно сохраняет outbox commands; здесь stale
  snapshot превращается в application consistency error.
- **Eventually through projection/audit.** История прежних контактов и другие
  temporal audit views строятся из `ContactEvent`/`EventEnvelope`, registry audit
  строится из `IdentityRegistryEvent`, а outbox idempotently выполняет claim и
  release. Они не участвуют в принятии локальной команды и могут быть eventually
  consistent.
- **Read-side constraints.** Отправка marketing notification разрешена только
  когда projection одновременно показывает включенную preference и
  `MarketingConsent = Granted`. Это dispatch/read-side правило; Preferences не
  читает ConsentLedger и не изменяет его.

В Rust локальные команды возвращают
`Result<ChangeOutcome, DomainError>`. `ChangeOutcome::NoChange` означает, что
state, `updated_at`, `AggregateVersion` и pending events остались неизменными.
`InvariantViolation` различает локальную ошибку агрегата и cross-aggregate
ошибку orchestration/policy, но application layer переводит их в собственный
use-case error contract.

### Communication preferences

1. `CommunicationPreferences` хранит пользовательский intent: разрешенные и
   предпочтительный каналы, quiet hours и правило приоритетов. Это не гарантия,
   что канал существует, подтвержден или юридически разрешен для конкретного
   сообщения.
2. Marketing preference можно включить без текущего marketing consent.
   Фактическая отправка требует одновременно enabled preference и
   `MarketingConsent = Granted`; это проверяет `NotificationDispatchPolicy` по
   read model, а не `CustomerPreferences`.
3. `CustomerPreferences` не зависит от `CustomerContactBook`. Доступность
   verified phone/email/Telegram формируется отдельной
   `CustomerContactAvailabilityProjection`. Это предотвращает cross-aggregate
   reads внутри domain command.
4. `QuietHours` является immutable Value Object с `LocalTime` start/end и
   обязательным `DstResolutionPolicy`. Он валидирует диапазон, поддерживает
   интервал через полночь, а применение к instant выполняется с
   `CustomerPreferences.timezone`.
5. Local invariants: валидные language/timezone, непустой `allowed_channels`,
   preferred channel внутри allowed set, валидные quiet hours и no-op semantics.
   Read-side/application rules: наличие verified contact, marketing consent,
   выбор доступного канала, попадание `now` в quiet hours и возможность bypass
   согласно `MessagePriorityPolicy`.

`PreferencesError::MarketingConsentRequiredAtDispatchTime` относится к
dispatch/use-case error mapping и не должен возвращаться методами изменения
preferences. Аналогично `CommunicationChannelUnavailable` возникает при выборе
маршрута доставки. `NoChange` предпочтительно представлять как
`ChangeOutcome::NoChange`; одноименный error variant следует считать только
внешней compatibility-классификацией и не использовать в новом domain API.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationPreferences {
    allowed_channels: NonEmptySet<ContactMethod>,
    preferred_channel: Option<ContactMethod>,
    quiet_hours: Option<QuietHours>,
    priority_policy: MessagePriorityPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactMethod {
    Phone,
    Email,
    Telegram,
    InApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuietHours {
    start: LocalTime,
    end: LocalTime,
    dst_policy: DstResolutionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DstResolutionPolicy {
    SkipAmbiguousLocalTime,
    PreferEarlierOffset,
    PreferLaterOffset,
    UseUtcFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriorityPolicy {
    TransactionalCanBypassQuietHours,
    CriticalCanBypassQuietHours,
    NothingBypassesQuietHours,
}
```

### Audit context pipeline

1. `AccessContext` вызывается до domain command, потому что aggregate не владеет
   актуальными assignments/permissions и не должен зависеть от Access bounded
   context.
2. Transient `AuthorizationDecision` фиксирует результат проверки конкретного
   `RequestedAction`. Отдельный ID не нужен, пока decision не сохраняется как
   самостоятельная audit record с retention policy.
3. `ActionContextFactory` принимает только `Allowed` decision, преобразует
   `PrincipalId` в `ActorRef` и копирует role snapshots. Поэтому
   `ActionContext` одновременно является command context и immutable audit
   context.
4. Domain method принимает готовый `ActionContext`; aggregate никогда не
   вызывает `AccessContext` и не использует role snapshots для повторной
   авторизации.
5. Aggregate создает `PendingCustomerEvent`, немедленно копируя `occurred_at`,
   actor, role snapshots, correlation и causation. Он не создает
   `EventEnvelope`, `EventId` или schema metadata.
6. `EventEnvelopeFactory` на persistence boundary назначает `EventId`,
   `EventType`, `EventSchemaVersion` и связывает event с фактически сохраняемой
   `AggregateVersion`. `UnitOfWork` использует factory перед атомарным append.
7. Pipeline имеет единственное направление:
   `AccessContext -> AuthorizationDecision -> ActionContext -> PendingCustomerEvent -> EventEnvelope`.
   После создания каждого следующего объекта audit fields не перечитываются из
   внешних систем и не изменяются.

### Principal role snapshots

`ActorRef` фиксирует стабильную identity инициатора и не содержит роли. Перед
командой `AccessContext` проверяет актуальные role assignments и возвращает
transient `AuthorizationDecision`. `ActionContextFactory` копирует участвовавшие
в решении роли в `ActionContext`, а aggregate фиксирует их в immutable
`PendingCustomerEvent`. `EventEnvelopeFactory` переносит эту копию в envelope
без повторного обращения к AccessContext.

Snapshot существует исключительно для audit: он позволяет понять историческое
событие без запроса к AccessContext, даже если роль позже переименована, удалена
или отозвана. Изменение текущих ролей никогда не переписывает старые events.
Permission checks никогда не используют snapshots из event store.

`RoleKey` является валидируемым открытым newtype, а не закрытым enum. Поэтому
роли вроде `regional_support_manager`, `insurance_agent`,
`partner_employee`, `marketplace_moderator`, `external_auditor` или custom
enterprise role добавляются в AccessContext без изменения `ActorRef`,
`EventEnvelope` и исторических event schemas. `RoleId` сохраняет стабильную
identity роли, а `RoleDisplayName` нужен только для audit/debug.

Для `System` и `Service` actor список role snapshots обычно пуст. Для principal
в envelope следует фиксировать только роли, релевантные authorization decision,
а не безусловно копировать все назначения пользователя.

```rust
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoleId(Uuid);

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoleKey(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrincipalRoleSnapshot {
    role_id: RoleId,
    role_key: RoleKey,
    display_name: Option<RoleDisplayName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContext {
    actor: ActorRef,
    role_snapshots: Vec<PrincipalRoleSnapshot>,
    now: DateTime<Utc>,
    correlation_id: CorrelationId,
    causation_id: Option<EventId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCustomerEvent {
    payload: CustomerEvent,
    occurred_at: DateTime<Utc>,
    actor: ActorRef,
    role_snapshots: Vec<PrincipalRoleSnapshot>,
    correlation_id: CorrelationId,
    causation_id: Option<EventId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationDecision {
    principal_id: PrincipalId,
    action: RequestedAction,
    outcome: AuthorizationOutcome,
    role_snapshots: Vec<PrincipalRoleSnapshot>,
    decided_at: DateTime<Utc>,
}

pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub actor: ActorRef,
    pub principal_role_snapshots: Vec<PrincipalRoleSnapshot>,
    pub payload: E,
    // Existing event metadata omitted here.
}

pub trait EventEnvelopeFactory {
    fn wrap_pending_event(
        &self,
        aggregate_id: CustomerId,
        aggregate_version: AggregateVersion,
        pending: PendingCustomerEvent,
    ) -> EventEnvelope<CustomerEvent>;
}
```

## Updated Customer aggregate root

```rust
#[derive(Debug)]
pub struct Customer {
    id: CustomerId,
    status: CustomerStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: AggregateVersion,
    pending_events: Vec<PendingCustomerEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerStatus {
    Draft,
    Active,
    Suspended(Suspension),
    Blocked(Block),
    Deleted(Deletion),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suspension {
    reason: SuspensionReason,
    actor: ActorRef,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    reason: BlockReason,
    actor: ActorRef,
    occurred_at: DateTime<Utc>,
}

impl Customer {
    pub fn activate(
        &mut self,
        permit: ActivationPermit,
        ctx: ActionContext,
    ) -> Result<ChangeOutcome, CustomerError> {
        permit.validate_local(self.id, self.version, ctx.now)?;

        match &self.status {
            CustomerStatus::Draft => {
                self.raise(CustomerEvent::Activated(CustomerActivatedV1 {
                    permit_id: permit.id(),
                }), ctx);
                Ok(ChangeOutcome::Changed)
            }
            CustomerStatus::Active => Ok(ChangeOutcome::NoChange),
            status => Err(CustomerError::CannotActivate(status.kind())),
        }
    }
}
```

Поля private. Восстановление из persistence выполняется через repository-owned
rehydration API (`pub(crate)`), а не публичный конструктор, допускающий нарушение
инвариантов. Event-sourced aggregate изменяет состояние только через `apply`.

## Event and concurrency rules

1. `AggregateVersion` — monotonic `u64` newtype. Нулевой stream имеет version 0.
2. Каждое принятое событие увеличивает version ровно на 1.
3. Append выполняется с `expected_version`; mismatch возвращает concurrency error.
4. `event_id` обеспечивает idempotent consumption; `(aggregate_id,
   aggregate_version)` уникален и задает порядок stream.
5. `schema_version` относится к payload schema, а не aggregate sequence.
6. `correlation_id` связывает workflow, `causation_id` указывает непосредственное
   событие-причину.
7. Domain event и integration event не обязаны совпадать один к одному.
8. Старые serialized events immutable. Изменение чтения выполняется upcaster-ами;
   destructive migration event store запрещена.
9. Snapshot является cache и всегда содержит stream version. Его можно удалить и
   полностью восстановить aggregate из events.
10. State-based repository использует ту же CAS-семантику, но не обязан хранить
    domain events как источник истины.

## Cross-aggregate activation semantics

`ActivationPermit` не является магическим обходом consistency. Application
workflow должен:

1. Загрузить `CustomerContactBook` и `CustomerConsentLedger` с версиями.
2. Загрузить `Customer` и сохранить его expected version.
3. Построить permit из проверенных snapshots всех трех агрегатов.
4. Перед append `CustomerActivated` проверить, что customer, contact и consent
   versions не изменились.
5. Атомарно записать activation event и consumption permit либо выполнить
   serializable transaction с version predicates.

Если storage не поддерживает такую транзакцию, допустимы только два честных
варианта: saga с компенсирующим возвратом в `Draft` или более слабая семантика,
при которой `Active` означает завершение onboarding в прошлом, а текущая
eligibility проверяется policy при каждой защищенной операции.

Если бизнес требует инвариант «Active всегда имеет verified contact и действующие
consent» без временного окна, разделение этих данных на разные aggregates
неправильно: минимальные eligibility facts придется вернуть в Customer
consistency boundary. Это осознанный trade-off, а не деталь repository.

## Recommended file structure

```text
crates/domain/src/
├── customer/
│   ├── mod.rs
│   ├── aggregate.rs
│   ├── error.rs
│   ├── event.rs
│   ├── repository.rs
│   ├── state.rs
│   └── value_objects/
│       ├── actor.rs
│       ├── context.rs
│       ├── id.rs
│       ├── permit.rs
│       └── reasons.rs
├── customer_contacts/
│   ├── aggregate.rs
│   ├── command.rs
│   ├── error.rs
│   ├── event.rs
│   └── value_objects.rs
├── customer_profile/
│   ├── aggregate.rs
│   ├── event.rs
│   └── value_objects.rs
├── customer_preferences/
│   ├── aggregate.rs
│   ├── event.rs
│   └── value_objects.rs
├── customer_consent/
│   ├── aggregate.rs
│   ├── event.rs
│   ├── policy.rs
│   └── value_objects.rs
└── eventing/
    ├── envelope.rs
    ├── metadata.rs
    └── version.rs
```

Auth, Access и Identity Registry не должны быть подмодулями `customer`; это
отдельные bounded contexts/crates или как минимум sibling domain modules с
явными ports.

## Must change now

- Удалить `CustomerIdentity` и `LoginIdentityRef` из Customer model.
- Разделить Customer на lifecycle, contacts, profile, preferences и consent
  aggregates.
- Удалить generic `ChangePayload`.
- Добавить `AggregateVersion` и expected-version repository contract.
- Разделить typed event payload и event envelope.
- Добавить `event_id`, `schema_version`, `aggregate_version`, `correlation_id` и
  `causation_id`.
- Убрать roles из `ActorRef` и authorization policy из Customer domain.
- Кодировать status transitions только методами Customer.
- Обеспечить atomic contact uniqueness reservation и запрет удаления primary.
- Зафиксировать event compatibility/upcasting policy до появления production
  events.

## Can be deferred

- Физическое разделение bounded contexts на отдельные crates.
- Event sourcing как persistence strategy; версия и typed events нужны уже
  сейчас, но state storage допустим.
- Snapshotting и compaction event streams.
- Quiet hours до появления notification scheduler, при условии что временный
  boolean API не публикуется как долгоживущий контракт.
- Полная Identity Audit projection до появления compliance requirement.
- Signed/cryptographically verifiable activation permits; сначала достаточно
  versioned permit с коротким TTL и транзакционной проверкой.
