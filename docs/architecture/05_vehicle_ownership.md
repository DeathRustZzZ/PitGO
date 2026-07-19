# 05. Модуль VehicleOwnership

## Назначение

Показать самый содержательный контекст проекта: единственный, где есть
кросс-агрегатный инвариант, машина состояний из трёх узлов и нетривиальная
координация между слоями.

## Что представлено

Маршрут `POST /vehicles/{vehicle_id}/ownerships`, обработчик, порт с тремя
методами, адаптер, агрегат и полная машина состояний.

## Как читать

Обратите внимание на **порядок** в обработчике: сначала чтение
(`has_open_ownership`), затем упаковка ответа в снимок, затем доменное решение,
и только потом запись. Именно этот порядок обеспечивает инвариант.

## Поток вызовов

```mermaid
flowchart TD

  R["POST /vehicles/{vehicle_id}/ownerships"]
  DTO["CreateVehicleOwnershipRequest<br/>ownership_id, owner_customer_id, ownership_type"]
  TDTO["OwnershipTypeDto<br/>serde snake_case"]
  CMD["StartVehicleOwnershipCommand<br/>4 поля"]

  H["StartVehicleOwnershipHandler"]

  PORT["trait VehicleOwnershipRepository"]
  M1["has_open_ownership(vehicle_id)"]
  M2["save(&ownership)"]
  M3["find_by_id(ownership_id)"]

  ADP["InMemoryVehicleOwnershipRepository<br/>Mutex HashMap"]

  SNAP["OwnershipEligibilitySnapshot<br/>vehicle_id, has_active_ownership"]
  AGG["VehicleOwnership::start(...)"]
  ISOPEN["OwnershipStatus::is_open()"]

  ERR["OwnershipError"]
  APIERR["ApiError 409 / 422"]

  R --> DTO
  DTO --> TDTO
  TDTO -->|"into_domain()"| CMD
  DTO --> CMD
  CMD --> H

  H -->|"1 - await"| M1
  M1 --> PORT
  PORT --> ADP
  ADP -->|"фильтр по vehicle_id<br/>И status.is_open()"| ISOPEN
  ADP -->|"bool"| SNAP

  H -->|"2 - строит"| SNAP
  SNAP -->|"3 - передаёт как доказательство"| AGG
  AGG -->|"no_active_ownership_exists()?"| SNAP
  AGG -->|"нет - ActiveOwnershipAlreadyExists"| ERR
  AGG -->|"да - агрегат создан"| M2

  H -->|"4 - await"| M2
  M2 --> PORT

  M3 -.->|"порт объявлен,<br/>ни один обработчик не вызывает"| PORT

  ERR --> APIERR

  classDef unused stroke-dasharray: 5 5
  class M3 unused
```

## Машина состояний

```mermaid
stateDiagram-v2
  [*] --> PendingVerification: start(...) + снимок свободен

  PendingVerification --> Active: verify(now)
  PendingVerification --> PendingVerification: start при занятом ТС → ошибка
  Active --> Ended: end(now)

  Active --> Active: verify → NoChange
  Ended --> Ended: end → NoChange

  PendingVerification --> PendingVerification: end() → StatusDoesNotAllow
  Ended --> Ended: verify() → StatusDoesNotAllow

  Ended --> [*]

  note right of PendingVerification
    is_open() = true
    Уже ЗАНИМАЕТ автомобиль.
    Это и был исходный дефект:
    ожидающая запись считалась
    свободной
  end note

  note right of Active
    is_open() = true
  end note

  note right of Ended
    is_open() = false
    Терминальное состояние,
    оживить нельзя
  end note
```

## Как обеспечивается инвариант «одно открытое владение на автомобиль»

Правило охватывает несколько агрегатов, поэтому реализовано послойно:

```mermaid
flowchart LR
  L1["Слой 1<br/>OwnershipEligibilitySnapshot<br/>внутри агрегата"]
  L2["Слой 2<br/>StartVehicleOwnershipHandler<br/>читает репозиторий"]
  L3["Слой 3<br/>частичный уникальный индекс<br/>в БД"]

  L1 -->|"даёт точную доменную ошибку"| OK["Инвариант"]
  L2 -->|"поставляет актуальный факт"| OK
  L3 -.->|"НЕ РЕАЛИЗОВАН:<br/>PostgreSQL отсутствует"| OK

  classDef missing stroke-dasharray: 5 5
  class L3 missing
```

**Существенное ограничение текущего состояния.** Слой 3 не существует, потому
что нет базы данных. Между чтением `has_open_ownership` и записью `save`
остаётся окно: два конкурентных запроса могут оба увидеть свободный автомобиль
и оба успешно записаться. In-memory-адаптер индексирует записи по
`VehicleOwnershipId`, поэтому выразить правило ограничением ключа он не может.

Иными словами: инвариант защищён от добросовестной ошибки, но **не защищён от
гонки**. Это осознанный компромисс периода разработки, а не упущение — но
знать о нём нужно.

## Фактическое покрытие

| Элемент | Достижим по HTTP |
|---|---|
| `VehicleOwnership::start` | да, `POST /vehicles/{id}/ownerships` |
| `VehicleOwnership::verify` | **нет** — вызывается только из тестов |
| `VehicleOwnership::end` | **нет** — вызывается только из тестов |
| `has_open_ownership` | да, косвенно через `start` |
| `save` | да |
| `find_by_id` | **нет** — обработчика чтения владения не существует |

Через API владение можно только **создать**. Подтвердить, завершить или
прочитать его нельзя: соответствующих обработчиков и маршрутов в коде нет.
