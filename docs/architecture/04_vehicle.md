# 04. Модуль Vehicle

## Назначение

Показать вертикальный срез контекста «Автомобиль» — от HTTP-маршрута до
агрегата, с реальными именами методов.

## Что представлено

Два маршрута (`POST /vehicles`, `GET /vehicles/{id}`), два обработчика, один
порт, один адаптер, агрегат `Vehicle` с состояниями, событиями и ошибками.

## Как читать

Структура намеренно совпадает с [03_customer.md](03_customer.md) — контексты
устроены одинаково. Пунктир означает код, не достижимый ни из одного маршрута.

## Поток вызовов

```mermaid
flowchart TD

  R1["POST /vehicles"]
  R2["GET /vehicles/{id}"]

  DTO1["CreateVehicleRequest<br/>vehicle_id: Uuid"]
  DTO2["VehicleResponse<br/>id, status, created_at, updated_at"]

  CMD["CreateVehicleCommand<br/>vehicle_id: VehicleId"]

  H1["CreateVehicleHandler"]
  H2["GetVehicleHandler"]

  PORT["trait VehicleRepository<br/>Send + Sync"]
  ADP["InMemoryVehicleRepository<br/>Mutex HashMap VehicleId Vehicle"]

  AGG["Vehicle<br/>агрегат"]
  ST["VehicleStatus<br/>Draft | Active"]
  EV["VehicleEvent<br/>Created | Activated"]
  BUF["pending_events<br/>буфер"]

  ERR["VehicleError<br/>VehicleActivationError"]
  PERMIT["VehicleActivationPermit"]
  APIERR["ApiError<br/>404 / 409 / 500"]

  R1 --> DTO1
  DTO1 --> CMD
  CMD --> H1
  H1 -->|"Vehicle::create(id, now)"| AGG
  H1 -->|"save(&vehicle).await"| PORT
  PORT --> ADP
  ADP -->|"version() / next()"| AGG

  R2 --> H2
  H2 -->|"find_by_id(id).await"| PORT
  H2 --> DTO2
  ADP -->|"Option Vehicle"| DTO2

  AGG --> ST
  AGG -->|"raise()"| EV
  EV --> BUF

  AGG -.->|"activate(permit, now)<br/>НЕ вызывается ни из одного маршрута"| PERMIT
  PERMIT -.->|"validate_local()"| ERR
  ERR -.-> APIERR
  ADP -->|"RepositoryError"| APIERR

  classDef unused stroke-dasharray: 5 5
  class PERMIT,ERR unused
```

## Состояния агрегата

```mermaid
stateDiagram-v2
  [*] --> Draft: create(id, now)
  Draft --> Active: activate(permit, now)
  Active --> Active: activate → NoChange (идемпотентно)

  note right of Draft
    Хранится только identity.
    VehicleSpecs, VIN, госномер
    в коде отсутствуют
  end note

  note right of Active
    Переход реализован в домене,
    но не подключён ни к одному
    HTTP-маршруту
  end note
```

## Фактическое покрытие

| Элемент | Файл | Достижим по HTTP |
|---|---|---|
| `Vehicle::create` | `domain/vehicle/aggregate.rs` | да, `POST /vehicles` |
| `Vehicle::activate` | `domain/vehicle/aggregate.rs` | **нет** |
| `VehicleRepository::save` | `application/vehicle/ports.rs` | да |
| `VehicleRepository::find_by_id` | `application/vehicle/ports.rs` | да, `GET /vehicles/{id}` |
| `VehicleActivationPermit` | `domain/vehicle/permit.rs` | **нет** |
| `VehicleEvent::Activated` | `domain/vehicle/event.rs` | **нет** |

## Отличие от модуля Customer

Единственное содержательное различие — в permit. `VehicleActivationPermit`
хранит `vehicle_id`, `issued_at`, `expires_at`, но **не хранит версию
агрегата**, тогда как `ActivationPermit` у клиента версию фиксирует и
сверяет.

Обоснование в коде: идентифицирующие сведения об автомобиле (VIN, госномер)
не меняются так, чтобы обесценить решение о пригодности, поэтому привязка к
версии давала бы только ложные отказы. У клиента иначе — там пригодность
зависит от подтверждённого контакта и согласия, которые могут быть отозваны.

## Чего в модуле нет

`VehicleSpecs`, VIN и госномер как объекты-значения в коде отсутствуют.
Агрегат `Vehicle` сейчас хранит только `id`, `status`, `created_at`,
`updated_at`, `version` и буфер событий. Тело автомобиля как предметной
сущности ещё не смоделировано.
