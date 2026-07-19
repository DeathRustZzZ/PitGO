# 12. Асинхронная архитектура

## Назначение

Показать, где в системе проходит граница между асинхронным и синхронным кодом,
где стоят точки `.await` и почему выбран именно блокирующий мьютекс.

## Что представлено

Полный асинхронный путь от Tokio-рантайма до агрегата, с явной отметкой
границы sync/async.

## Как читать

Красная граница на диаграмме — место, где асинхронный мир заканчивается.
Ниже неё нет ни одной `async fn`: весь домен синхронный, и это осознанное
решение.

## Граница async и sync

```mermaid
flowchart TD

  subgraph rt["Tokio runtime — multi-thread"]
    MAIN["#[tokio::main] async fn main"]
    LISTEN["TcpListener::bind().await"]
    SERVE["axum::serve().await"]
  end

  subgraph async_zone["АСИНХРОННАЯ ЗОНА"]
    ROUTE["async fn create_vehicle_ownership"]
    HANDLE["async fn handle(cmd)"]
    PORT["trait метод async fn<br/>через #[async_trait]"]
    ADP["async fn в адаптере"]
  end

  subgraph sync_zone["СИНХРОННАЯ ЗОНА — весь domain"]
    LOCK["Mutex::lock()<br/>блокирующий"]
    MAP["HashMap get / insert"]
    AGG["VehicleOwnership::start()"]
    VER["AggregateVersion::next()"]
    SNAP["OwnershipEligibilitySnapshot"]
  end

  MAIN --> LISTEN
  MAIN --> SERVE
  SERVE -->|"await"| ROUTE
  ROUTE -->|"await"| HANDLE
  HANDLE -->|"await"| PORT
  PORT -->|"dyn dispatch"| ADP
  ADP ==>|"ГРАНИЦА sync/async"| LOCK
  LOCK --> MAP
  HANDLE ==>|"ГРАНИЦА sync/async"| AGG
  AGG --> VER
  AGG --> SNAP

  classDef syncStyle fill:#f5f5f5,stroke:#333,stroke-width:2px
  class LOCK,MAP,AGG,VER,SNAP syncStyle
```

## Все точки await в кодовой базе

| Файл | Вызов | Реально приостанавливается |
|---|---|---|
| `main.rs` | `TcpListener::bind(addr).await` | да — сетевая операция |
| `main.rs` | `axum::serve(listener, app).await` | да — бесконечно |
| `routers/customer.rs` | `handler.handle(cmd).await` | нет |
| `routers/vehicle.rs` | `handler.handle(cmd).await` | нет |
| `routers/vehicle_ownership.rs` | `handler.handle(cmd).await` | нет |
| `customer/handlers.rs` | `self.repository.save(...).await` | нет |
| `customer/handlers.rs` | `self.repository.find_by_id(...).await` | нет |
| `vehicle/handlers.rs` | `save` / `find_by_id` `.await` | нет |
| `ownership/handlers.rs` | `has_open_ownership(...).await` | нет |
| `ownership/handlers.rs` | `save(...).await` | нет |

**Важное следствие.** Кроме двух вызовов в `main.rs`, ни один `await` в системе
не приводит к настоящей приостановке. За портом стоит `HashMap`, future
готов немедленно, задача продолжает выполняться на том же рабочем потоке.

Асинхронность здесь — **форма контракта**, рассчитанная на будущий PostgreSQL,
а не работающая конкурентность. Это не недостаток: порт должен иметь форму,
задаваемую требовательной реализацией, а не удобной. Но при чтении профиля
производительности об этом стоит помнить.

## Почему `#[async_trait]`

```mermaid
flowchart LR
  A["async fn в трейте<br/>нативная поддержка"]
  B["не поддерживает<br/>dyn-диспетчеризацию"]
  C["#[async_trait]<br/>переписывает в Box dyn Future"]
  D["dyn CustomerRepository<br/>работает"]
  E["Arc dyn Trait в AppState"]

  A --> B
  B -->|"поэтому"| C
  C --> D
  D --> E
```

Порты должны быть object-safe, потому что `AppState` хранит их как
`Arc<dyn Trait>` — иначе обработчики пришлось бы параметризовать типом
адаптера, и `backend` снова узнал бы про конкретную реализацию.

## Почему `std::sync::Mutex`, а не `tokio::sync::Mutex`

```mermaid
flowchart TD
  Q{"Удерживается ли охранник<br/>блокировки через .await?"}
  NO["НЕТ — так в текущем коде"]
  YES["ДА — гипотетически"]
  S1["std::sync::Mutex корректен<br/>дешевле, без аллокаций"]
  S2["Нужен tokio::sync::Mutex<br/>иначе рабочий поток встанет"]

  Q -->|"нет"| NO --> S1
  Q -->|"да"| YES --> S2

  classDef danger stroke-dasharray: 5 5
  class YES,S2 danger
```

Каждый метод адаптера захватывает блокировку, выполняет **исключительно
синхронную** работу (`get`, `insert`, `values().any()`) и освобождает её до
возврата. Обычная угроза — заблокировать рабочий поток Tokio на время
ожидания — здесь возникнуть не может.

**Это условие, а не свойство.** Добавление `.await` внутрь заблокированного
участка немедленно делает выбор неверным. Единственное изменение, ломающее
текущую корректность, — и его легко внести случайно при переходе на SQLx, где
запрос естественно захочется выполнить под блокировкой.

## Требования к потокобезопасности портов

```mermaid
flowchart LR
  P["trait: Send + Sync"]
  R1["Arc dyn Trait<br/>между задачами"]
  R2["Задачи мигрируют<br/>между worker-потоками"]
  R3["Методы принимают &self<br/>не &mut self"]
  R4["Реализация обязана сама<br/>обеспечить внутреннюю<br/>изменяемость"]

  P --> R1 --> R2
  P --> R3 --> R4
```

`&self` вместо `&mut self` — не стилистика: несколько задач держат один и тот
же `Arc` одновременно, эксклюзивную ссылку получить невозможно. Отсюда
`Mutex` внутри каждого адаптера.

## Где конкурентность реально проявится

Сейчас — практически нигде, поскольку всё в памяти. Но окно между чтением и
записью в `StartVehicleOwnershipHandler` существует уже сегодня:

```mermaid
sequenceDiagram
  participant T1 as Задача A
  participant T2 as Задача B
  participant R as Репозиторий

  T1->>R: has_open_ownership → false
  T2->>R: has_open_ownership → false
  Note over T1,T2: обе видят свободный автомобиль
  T1->>R: save → Ok
  T2->>R: save → Ok
  Note over R: ДВА открытых владения<br/>на один автомобиль
```

Между `has_open_ownership` и `save` блокировка **не удерживается** — она
берётся и отпускается внутри каждого метода отдельно. Инвариант защищён от
добросовестной ошибки, но не от гонки. Закрыть окно должен частичный
уникальный индекс в БД, которой нет.
