# 03. Модуль Customer

## Назначение

Показать вертикальный срез контекста «Клиент» — от HTTP-маршрута до агрегата
и обратно, с реальными именами методов.

## Что представлено

Два маршрута (`POST /customers`, `GET /customers/{id}`), два обработчика, один
порт, один адаптер, агрегат `Customer` с его состояниями, событиями и ошибками.

## Как читать

Подписи на стрелках — фактические имена вызываемых методов. Пунктирные узлы
и стрелки означают код, который существует, но ни из одного маршрута не
достижим.

## Поток вызовов

```mermaid
flowchart TD

  R1["POST /customers"]
  R2["GET /customers/{id}"]

  DTO1["CreateCustomerRequest<br/>customer_id: Uuid"]
  DTO2["CustomerResponse<br/>id, status, created_at, updated_at"]

  CMD["CreateCustomerCommand<br/>customer_id: CustomerId"]

  H1["CreateCustomerHandler"]
  H2["GetCustomerHandler"]

  PORT["trait CustomerRepository<br/>Send + Sync"]
  ADP["InMemoryCustomerRepository<br/>Mutex HashMap CustomerId Customer"]

  AGG["Customer<br/>агрегат"]
  ST["CustomerStatus<br/>Draft | Active"]
  EV["CustomerEvent<br/>Created | Activated"]
  BUF["pending_events<br/>буфер"]

  ERR["CustomerError<br/>CustomerActivationError"]
  PERMIT["ActivationPermit"]
  APIERR["ApiError<br/>404 / 409 / 500"]

  R1 --> DTO1
  DTO1 --> CMD
  CMD --> H1
  H1 -->|"Customer::create(id, now)"| AGG
  H1 -->|"save(&customer).await"| PORT
  PORT --> ADP
  ADP -->|"version() / next()"| AGG

  R2 --> H2
  H2 -->|"find_by_id(id).await"| PORT
  H2 --> DTO2
  ADP -->|"Option Customer"| DTO2

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
    Версия = 1 сразу после create:
    создание само является событием
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
| `Customer::create` | `domain/customer/aggregate.rs` | да, `POST /customers` |
| `Customer::activate` | `domain/customer/aggregate.rs` | **нет** |
| `CustomerRepository::save` | `application/customer/ports.rs` | да |
| `CustomerRepository::find_by_id` | `application/customer/ports.rs` | да, `GET /customers/{id}` |
| `ActivationPermit` | `domain/customer/permit.rs` | **нет** |
| `CustomerEvent::Activated` | `domain/customer/event.rs` | **нет** |

## Что стоит знать

**Активация недостижима.** `Customer::activate` и весь механизм
`ActivationPermit` реализованы и покрыты юнит-тестами, но ни один обработчик
приложения их не вызывает. Клиент, созданный через API, навсегда остаётся в
статусе `Draft`. Обработчика `ActivateCustomerHandler` и маршрута
`POST /customers/{id}/activate` в коде нет.

**События никуда не уходят.** `raise()` складывает событие в `pending_events`,
но `pull_pending_events()` вызывается только из доменных тестов — ни репозиторий,
ни обработчик его не дренируют. Буфер событий заполняется и умирает вместе с
агрегатом. Подробнее в [13_gaps.md](13_gaps.md).

**Дубликат создания отклоняется через версию.** Отдельной проверки «уже
существует» нет: второй `create` приходит с версией 1, в хранилище уже лежит
версия 1, ожидается 2 — получается `VersionConflict` → HTTP 409.
