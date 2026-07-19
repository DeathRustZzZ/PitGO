# 01. Обзор системы

## Назначение

Показать сквозной путь запроса через все слои: от HTTP-клиента до доменного
агрегата и обратно. Это входная точка в набор диаграмм — остальные файлы
детализируют отдельные её участки.

## Что представлено

Все шесть реализованных HTTP-маршрутов, слой приложения, порты репозиториев,
in-memory-адаптеры, доменные агрегаты и разделяемое ядро. Внешние системы
показаны только существующие.

## Как читать

Слева направо — направление потока запроса. Направление **зависимостей**
противоположно на участке порт/адаптер: `infrastructure` зависит от
`application`, а не наоборот (см. [06_dependency_graph.md](06_dependency_graph.md)).

Пунктир означает связь, заложенную в коде, но не задействованную.

```mermaid
flowchart LR

  subgraph client["Клиенты"]
    HTTP["HTTP-клиент<br/>curl / Postman"]
    FE["Frontend React<br/>localhost:5173"]
  end

  subgraph backend["backend (bin)"]
    CORS["CorsLayer<br/>origin: localhost:5173"]
    Router["axum Router<br/>main.rs"]
    RC["routers::customer"]
    RV["routers::vehicle"]
    RO["routers::vehicle_ownership"]
    Health["GET /health"]
    ApiErr["ApiError<br/>error.rs"]
  end

  subgraph app["application"]
    HCC["CreateCustomerHandler"]
    HGC["GetCustomerHandler"]
    HCV["CreateVehicleHandler"]
    HGV["GetVehicleHandler"]
    HSO["StartVehicleOwnershipHandler"]
    PortC["trait CustomerRepository"]
    PortV["trait VehicleRepository"]
    PortO["trait VehicleOwnershipRepository"]
    AppErr["ApplicationError<br/>RepositoryError"]
  end

  subgraph infra["infrastructure"]
    MemC["InMemoryCustomerRepository<br/>Mutex HashMap"]
    MemV["InMemoryVehicleRepository<br/>Mutex HashMap"]
    MemO["InMemoryVehicleOwnershipRepository<br/>Mutex HashMap"]
  end

  subgraph dom["domain"]
    AggC["Customer"]
    AggV["Vehicle"]
    AggO["VehicleOwnership"]
    Snap["OwnershipEligibilitySnapshot"]
    DomErr["OwnershipError"]
  end

  subgraph sh["shared"]
    Ver["AggregateVersion"]
    Outcome["ChangeOutcome"]
    Pend["PendingEvent"]
  end

  HTTP --> CORS
  FE -.->|"не вызывается:<br/>фронт на моках"| CORS
  CORS --> Router
  Router --> Health
  Router --> RC
  Router --> RV
  Router --> RO

  RC --> HCC
  RC --> HGC
  RV --> HCV
  RV --> HGV
  RO --> HSO

  HCC --> PortC
  HGC --> PortC
  HCV --> PortV
  HGV --> PortV
  HSO --> PortO

  PortC -.->|"impl"| MemC
  PortV -.->|"impl"| MemV
  PortO -.->|"impl"| MemO

  HCC --> AggC
  HCV --> AggV
  HSO --> AggO
  HSO --> Snap
  Snap --> AggO

  AggC --> Ver
  AggV --> Ver
  AggO --> Ver
  AggO --> Outcome
  AggC --> Pend
  AggV --> Pend
  AggO --> Pend

  MemC --> Ver
  MemV --> Ver
  MemO --> Ver

  AggO --> DomErr
  DomErr --> AppErr
  AppErr --> ApiErr
  ApiErr --> HTTP

  classDef unused stroke-dasharray: 5 5
  class FE unused
```

## Реализованные маршруты

| Метод | Путь | Обработчик | Слой приложения |
|---|---|---|---|
| GET | `/health` | `health` | — (не проходит через слои) |
| POST | `/customers` | `create_customer` | `CreateCustomerHandler` |
| GET | `/customers/{id}` | `get_customer` | `GetCustomerHandler` |
| POST | `/vehicles` | `create_vehicle` | `CreateVehicleHandler` |
| GET | `/vehicles/{id}` | `get_vehicle` | `GetVehicleHandler` |
| POST | `/vehicles/{vehicle_id}/ownerships` | `create_vehicle_ownership` | `StartVehicleOwnershipHandler` |
| GET | `/vehicles/{vehicle_id}/ownerships/{ownership_id}` | `get_vehicle_ownership` | `GetVehicleOwnershipHandler` |

## Внешние системы

**Их нет.** Ни базы данных, ни брокера сообщений, ни внешних HTTP-сервисов.
Всё состояние живёт в `HashMap` внутри процесса и теряется при перезапуске.

Про фронтенд: в `main.rs` настроен CORS на `http://localhost:5173`, то есть
связь подготовлена. Но `frontend/src/features/*/api/*.ts` — это моки с
`setTimeout`, а `apiFetch` из `shared/api/client.ts` не вызывается ни из
одного места. Поэтому стрелка от фронтенда пунктирная: **сейчас React-часть
и Rust-часть не общаются**.
