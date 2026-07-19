# 09. Жизненный цикл запроса

## Назначение

Показать во времени, что происходит при обработке HTTP-запроса: порядок
вызовов, точки `await`, места принятия решений и формирование ответа.

## Что представлено

Три сценария: успешное создание, доменный отказ и чтение с результатом
«не найдено».

## Как читать

Пометка `await` на стрелке означает точку, где задача может уступить рабочий
поток Tokio. `sync` означает синхронный вызов без приостановки — весь домен
синхронный.

## Сценарий 1: успешное создание владения

Самый содержательный путь — единственный, где обработчик выполняет и чтение,
и запись.

```mermaid
sequenceDiagram
  autonumber
  actor Client as HTTP-клиент
  participant Axum as axum Router
  participant Route as create_vehicle_ownership
  participant H as StartVehicleOwnershipHandler
  participant Port as VehicleOwnershipRepository
  participant Adp as InMemory адаптер
  participant Agg as VehicleOwnership

  Client->>Axum: POST /vehicles/{id}/ownerships
  Axum->>Axum: CorsLayer
  Axum->>Route: State + Path + Json

  Note over Route: serde десериализует тело<br/>OwnershipTypeDto → OwnershipType
  Route->>Route: собирает StartVehicleOwnershipCommand
  Route->>H: handle(cmd).await

  H->>H: now = Utc::now()
  H->>Port: has_open_ownership(vehicle_id).await
  activate Port
  Port->>Adp: динамическая диспетчеризация
  Adp->>Adp: lock() — блокирующий Mutex
  Adp->>Adp: перебор по vehicle_id И is_open()
  Adp-->>Port: Ok(false)
  deactivate Port
  Port-->>H: false

  Note over H: упаковывает факт в снимок
  H->>Agg: start(..., snapshot, now) — sync
  activate Agg
  Agg->>Agg: no_active_ownership_exists()? да
  Agg->>Agg: status = PendingVerification
  Agg->>Agg: raise(Started) — версия 0 → 1
  Agg-->>H: Ok(VehicleOwnership)
  deactivate Agg

  H->>Port: save(&ownership).await
  activate Port
  Port->>Adp: динамическая диспетчеризация
  Adp->>Adp: lock()
  Adp->>Adp: проверка версии: в хранилище пусто → вставка
  Adp-->>Port: Ok(())
  deactivate Port
  Port-->>H: Ok(())

  H-->>Route: Ok(())
  Route-->>Axum: 201 CREATED + JSON
  Axum-->>Client: HTTP 201

  Note over Agg: pending_events остаются<br/>в агрегате и теряются:<br/>pull_pending_events не вызывается
```

## Сценарий 2: доменный отказ — автомобиль занят

```mermaid
sequenceDiagram
  autonumber
  actor Client as HTTP-клиент
  participant Route as create_vehicle_ownership
  participant H as StartVehicleOwnershipHandler
  participant Port as VehicleOwnershipRepository
  participant Agg as VehicleOwnership
  participant Err as ApiError

  Client->>Route: POST /vehicles/{id}/ownerships
  Route->>H: handle(cmd).await
  H->>Port: has_open_ownership(vehicle_id).await
  Port-->>H: true — уже занят

  H->>Agg: start(..., snapshot, now) — sync
  Agg--xH: Err(ActiveOwnershipAlreadyExists)

  Note over H: оператор ? прерывает выполнение<br/>save() НЕ вызывается

  H-->>Route: Err(ApplicationError::Ownership)
  Route->>Err: From<ApplicationError>
  Err->>Err: conflict("Active ownership already exists.")
  Err-->>Client: HTTP 409 + JSON
```

Существенная деталь: запись не происходит вовсе. Оператор `?` в обработчике
прерывает выполнение до `save`, поэтому отклонённая команда не оставляет
следов в хранилище.

## Сценарий 3: чтение, объект не найден

```mermaid
sequenceDiagram
  autonumber
  actor Client as HTTP-клиент
  participant Route as get_customer
  participant H as GetCustomerHandler
  participant Port as CustomerRepository
  participant Adp as InMemory адаптер

  Client->>Route: GET /customers/{id}
  Route->>H: handle(customer_id).await
  H->>Port: find_by_id(id).await
  Port->>Adp: lock() + get().cloned()
  Adp-->>Port: None
  Port-->>H: Ok(None)

  Note over H: Ok(None) — не ошибка.<br/>Слой приложения не знает про HTTP

  H-->>Route: Ok(None)
  Route->>Route: match → ApiError::not_found
  Route-->>Client: HTTP 404 + JSON
```

Обратите внимание, где принимается решение: `Ok(None)` доходит до маршрута
нетронутым, и только транспортный слой превращает отсутствие в `404`. Слой
приложения отсутствие ошибкой не считает.

## Сводка точек await

| Участок | Приостановка | Обоснование |
|---|---|---|
| axum → маршрут | да | Транспорт, чтение тела запроса |
| Маршрут → обработчик | да | `handle(cmd).await` |
| Обработчик → порт | да | Порт объявлен `async` |
| Порт → in-memory адаптер | **фактически нет** | `HashMap` отвечает сразу, future готов немедленно |
| Обработчик → агрегат | **нет** | Домен полностью синхронный |

В текущем виде ни один `await` не приводит к реальной приостановке: за портом
стоит `HashMap`, а не сеть. Асинхронность здесь — форма контракта под будущий
PostgreSQL, а не работающая конкурентность. Подробнее в
[12_async_architecture.md](12_async_architecture.md).
