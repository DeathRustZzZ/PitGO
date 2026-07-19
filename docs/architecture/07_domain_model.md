# 07. Доменная модель

## Назначение

Показать структуру доменных типов: поля агрегатов, объекты-значения,
идентификаторы, события, ошибки и связи между ними.

## Что представлено

Все публичные типы крейтов `domain` и `shared`. Приватные поля агрегатов
показаны — они существуют в коде, хотя снаружи недоступны.

## Как читать

- `-->` — композиция или владение полем
- `..>` — ссылка по идентификатору (объект не загружается)
- Приватные поля помечены `-`, публичные `+`

Обратите внимание: `VehicleOwnership` **ссылается** на `Customer` и `Vehicle`
по id, но не содержит их. Это граница агрегатов, а не техническая деталь.

## Агрегаты и их состав

```mermaid
classDiagram
  direction TB

  class Customer {
    -id: CustomerId
    -status: CustomerStatus
    -created_at: DateTime
    -updated_at: DateTime
    -version: AggregateVersion
    -pending_events: Vec~PendingEvent~
    +create(id, now) Customer$
    +activate(permit, now) Result~ChangeOutcome~
    +pull_pending_events() Vec~PendingEvent~
    +id() CustomerId
    +status() CustomerStatus
    +version() AggregateVersion
  }

  class Vehicle {
    -id: VehicleId
    -status: VehicleStatus
    -created_at: DateTime
    -updated_at: DateTime
    -version: AggregateVersion
    -pending_events: Vec~PendingEvent~
    +create(id, now) Vehicle$
    +activate(permit, now) Result~ChangeOutcome~
    +pull_pending_events() Vec~PendingEvent~
  }

  class VehicleOwnership {
    -id: VehicleOwnershipId
    -vehicle_id: VehicleId
    -owner_customer_id: CustomerId
    -ownership_type: OwnershipType
    -status: OwnershipStatus
    -period: OwnershipPeriod
    -version: AggregateVersion
    -pending_events: Vec~PendingEvent~
    +start(...) Result~VehicleOwnership~$
    +verify(now) Result~ChangeOutcome~
    +end(now) Result~ChangeOutcome~
    +pull_pending_events() Vec~PendingEvent~
  }

  class CustomerStatus {
    <<enumeration>>
    Draft
    Active
    +kind() CustomerStatusKind
  }

  class VehicleStatus {
    <<enumeration>>
    Draft
    Active
    +kind() VehicleStatusKind
  }

  class OwnershipStatus {
    <<enumeration>>
    PendingVerification
    Active
    Ended
    +is_open() bool
    +kind() OwnershipStatusKind
  }

  class OwnershipPeriod {
    <<value object>>
    +started_at: DateTime
    +ended_at: Option~DateTime~
    +new(started_at) OwnershipPeriod$
    +is_open() bool
    +close(now) Option~OwnershipPeriod~
  }

  class OwnershipType {
    <<enumeration>>
    Private
    Company
    Leasing
    Fleet
    Unknown
  }

  Customer --> CustomerStatus
  Vehicle --> VehicleStatus
  VehicleOwnership --> OwnershipStatus
  VehicleOwnership --> OwnershipPeriod
  VehicleOwnership --> OwnershipType

  VehicleOwnership ..> Customer: ссылка по CustomerId
  VehicleOwnership ..> Vehicle: ссылка по VehicleId
```

## Capability-объекты и снимки

```mermaid
classDiagram
  direction LR

  class ActivationPermit {
    <<capability>>
    -customer_id: CustomerId
    -customer_version: AggregateVersion
    -issued_at: DateTime
    -expires_at: DateTime
    +new(...) ActivationPermit$
    +validate_local(id, version, now) Result
  }

  class VehicleActivationPermit {
    <<capability>>
    -vehicle_id: VehicleId
    -issued_at: DateTime
    -expires_at: DateTime
    +new(...) VehicleActivationPermit$
    +validate_local(id, now) Result
  }

  class OwnershipEligibilitySnapshot {
    <<capability>>
    -vehicle_id: VehicleId
    -has_active_ownership: bool
    +new(vehicle_id, flag) Snapshot$
    +vehicle_id() VehicleId
    +no_active_ownership_exists() bool
  }

  ActivationPermit ..> Customer: предъявляется в activate()
  VehicleActivationPermit ..> Vehicle: предъявляется в activate()
  OwnershipEligibilitySnapshot ..> VehicleOwnership: предъявляется в start()

  class Customer
  class Vehicle
  class VehicleOwnership
```

Отличие permit от снимка: permit фиксирует **разрешение**, выданное policy, а
снимок передаёт **факт**, прочитанный сервисом приложения. `ActivationPermit`
дополнительно привязан к версии агрегата, `VehicleActivationPermit` — нет
(обоснование см. в [04_vehicle.md](04_vehicle.md)).

## Идентификаторы

```mermaid
classDiagram
  class define_id {
    <<macro>>
    repr transparent
    Copy + Eq + Hash
    +new() Self$
    +from_uuid(uuid) Self$
    +as_uuid() Uuid
  }

  class CustomerId
  class VehicleId
  class VehicleOwnershipId
  class ActorId
  class CorrelationId
  class CausationId
  class EventId

  define_id <|-- CustomerId
  define_id <|-- VehicleId
  define_id <|-- VehicleOwnershipId
  define_id <|-- ActorId
  define_id <|-- CorrelationId
  define_id <|-- CausationId
  define_id <|-- EventId
```

`CustomerId`, `VehicleId`, `VehicleOwnershipId` объявлены в `domain::ids`.
Остальные четыре — в `shared::ids`. Из них **в коде используется только
`CustomerId`, `VehicleId`, `VehicleOwnershipId`**: `ActorId`, `CorrelationId`,
`CausationId`, `EventId` нигде не конструируются.

## События

```mermaid
classDiagram
  direction TB

  class CustomerEvent {
    <<enumeration>>
    Created(CustomerCreatedV1)
    Activated(CustomerActivatedV1)
  }
  class VehicleEvent {
    <<enumeration>>
    Created(VehicleCreatedV1)
    Activated(VehicleActivatedV1)
  }
  class VehicleOwnershipEvent {
    <<enumeration>>
    Started(VehicleOwnershipStartedV1)
    Verified(VehicleOwnershipVerifiedV1)
    Ended(VehicleOwnershipEndedV1)
  }

  class PendingEvent {
    +payload: E
    +occurred_at: DateTime
  }

  class EventEnvelope {
    +event_id: EventId
    +aggregate_type: String
    +aggregate_id: String
    +aggregate_version: AggregateVersion
    +event_type: String
    +payload: E
    +correlation_id: CorrelationId
    +causation_id: CausationId
    +actor_id: ActorId
    +occurred_at: DateTime
    +stored_at: DateTime
  }

  CustomerEvent --> PendingEvent
  VehicleEvent --> PendingEvent
  VehicleOwnershipEvent --> PendingEvent
  PendingEvent ..> EventEnvelope: НЕ РЕАЛИЗОВАНО
```

`EventEnvelope` определён в `shared::event`, но **нигде не конструируется**.
Преобразование `PendingEvent → EventEnvelope` в коде отсутствует, фабрики
конвертов нет. См. [13_gaps.md](13_gaps.md).

## Ошибки

```mermaid
classDiagram
  direction TB

  class CustomerError {
    <<enumeration>>
    Activation(CustomerActivationError)
  }
  class CustomerActivationError {
    <<enumeration>>
    PermitExpired
    PermitCustomerIdMismatch
    PermitVersionMismatch
    StatusDoesNotAllow(kind)
  }
  class VehicleError {
    <<enumeration>>
    Activation(VehicleActivationError)
  }
  class VehicleActivationError {
    <<enumeration>>
    PermitVehicleIdMismatch
    PermitExpired
    StatusDoesNotAllow(kind)
  }
  class OwnershipError {
    <<enumeration>>
    ActiveOwnershipAlreadyExists
    StatusDoesNotAllow(kind)
    PeriodEndBeforeStart
  }
  class ApplicationError {
    <<enumeration>>
    Repository(RepositoryError)
    Ownership(OwnershipError)
  }
  class RepositoryError {
    <<enumeration>>
    VersionConflict
    StorageFailure(String)
  }
  class ApiError {
    -status: StatusCode
    -body: ErrorBody
  }

  CustomerError --> CustomerActivationError
  VehicleError --> VehicleActivationError
  ApplicationError --> RepositoryError
  ApplicationError --> OwnershipError
  ApplicationError --> ApiError: From
```

**Асимметрия, заметная на диаграмме:** `ApplicationError` знает про
`OwnershipError`, но не про `CustomerError` и `VehicleError`. Это прямое
следствие того, что `activate()` нигде не вызывается — доменные ошибки
клиента и автомобиля физически не могут возникнуть в слое приложения, поэтому
для них нет и варианта преобразования.
