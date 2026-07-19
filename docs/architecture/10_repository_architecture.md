# 10. Архитектура репозиториев

## Назначение

Показать устройство слоя доступа к данным: порты, их реализации, контракт
оптимистичной блокировки и способ разделения адаптера между задачами.

## Что представлено

Три порта, три in-memory-адаптера, три мока из тестов. PostgreSQL-адаптер
**не реализован** и показан только как незанятое место в контракте.

## Как читать

`<|..` означает «реализует трейт». Пунктирная рамка — тип, которого в коде
нет.

## Порты и реализации

```mermaid
classDiagram
  direction TB

  class CustomerRepository {
    <<trait>>
    Send + Sync
    +save(customer) Result~unit~
    +find_by_id(id) Result~Option~
  }
  class VehicleRepository {
    <<trait>>
    Send + Sync
    +save(vehicle) Result~unit~
    +find_by_id(id) Result~Option~
  }
  class VehicleOwnershipRepository {
    <<trait>>
    Send + Sync
    +has_open_ownership(vehicle_id) Result~bool~
    +save(ownership) Result~unit~
    +find_by_id(id) Result~Option~
  }

  class InMemoryCustomerRepository {
    -customers: Mutex~HashMap~
    +new() Self$
  }
  class InMemoryVehicleRepository {
    -vehicle: Mutex~HashMap~
    +new() Self$
  }
  class InMemoryVehicleOwnershipRepository {
    -vehicle_ownership: Mutex~HashMap~
    +new() Self$
  }

  class MockCustomerRepository {
    <<test only>>
    -saved_ids: Mutex~Vec~
    -save_error: Option~RepositoryError~
  }
  class MockVehicleRepository {
    <<test only>>
    -saved_ids: Mutex~Vec~
    -save_error: Option~RepositoryError~
  }
  class MockOwnershipRepository {
    <<test only>>
    -has_active: bool
    -has_active_error: Option
    -save_error: Option
    -saved_ids: Mutex~Vec~
  }

  class SqlxCustomerRepository {
    <<НЕ РЕАЛИЗОВАН>>
  }

  CustomerRepository <|.. InMemoryCustomerRepository
  CustomerRepository <|.. MockCustomerRepository
  CustomerRepository <|.. SqlxCustomerRepository: место в контракте свободно

  VehicleRepository <|.. InMemoryVehicleRepository
  VehicleRepository <|.. MockVehicleRepository

  VehicleOwnershipRepository <|.. InMemoryVehicleOwnershipRepository
  VehicleOwnershipRepository <|.. MockOwnershipRepository
```

## О PostgreSQL

**Реализации нет.** В коде отсутствуют: адаптер на SQLx, миграции, пул
соединений, строка подключения, любые SQL-запросы.

Что действительно есть:

- `sqlx` объявлена в `[workspace.dependencies]` корневого `Cargo.toml`
  с фичами `postgres`, `migrate`, `runtime-tokio`, `tls-rustls`, `uuid`, `chrono`
- ни один крейт эту зависимость не подключает
- порты объявлены `async` и возвращают `RepositoryError`, то есть контракт под
  сетевое хранилище уже готов
- в доках агрегата `VehicleOwnership` упоминается частичный уникальный индекс
  как окончательный гарант инварианта

Иначе говоря: **интерфейс готов принять SQLx-адаптер, но самого адаптера не
существует**. Узел `SqlxCustomerRepository` на диаграмме отмечает свободное
место в контракте, а не запланированный код.

## Контракт оптимистичной блокировки

Все три in-memory-адаптера реализуют его одинаково:

Логика вынесена в общий `pub(crate) fn check_version` в
`infrastructure/lib.rs`, чтобы правило было записано один раз, а не
продублировано в трёх адаптерах:

```mermaid
flowchart TD
  A["save(aggregate)"]
  B["lock() — блокирующий Mutex"]
  C{"отравлена?"}
  D["StorageFailure(e)"]
  E["check_version(stored, incoming)"]
  G{"в хранилище есть запись?"}
  H["Ok — первая вставка"]
  V{"incoming == 1?"}
  AE["AlreadyExists<br/>дубликат создания"]
  I{"stored.next() == incoming?"}
  J["VersionConflict<br/>expected, actual"]
  K["insert(id, clone)"]
  L["Ok(())"]

  A --> B --> C
  C -->|"да"| D
  C -->|"нет"| E --> G
  G -->|"нет"| H --> K
  G -->|"да"| V
  V -->|"да"| AE
  V -->|"нет"| I
  I -->|"нет"| J
  I -->|"да"| K --> L
```

**Дубликат создания и устаревшая запись теперь различаются.** До PR #9 обе
ситуации давали `VersionConflict`. Сейчас `check_version` разделяет их по
входящей версии: агрегат, только что созданный, всегда приходит с версией 1,
поэтому «версия 1 при непустом хранилище» — это повторное создание
(`AlreadyExists`), а любое другое расхождение — устаревшее обновление
(`VersionConflict`).

Различие не косметическое: клиенту стоит повторить устаревшую запись после
перечитывания, но повтор занятого идентификатора не удастся никогда. На уровне
HTTP это два разных кода при одном статусе 409 — см.
[03_customer.md](03_customer.md) и таблицу кодов в `backend/src/error.rs`.

**Допущение, на котором держится различение.** «Версия 1 = только что создан»
верно лишь потому, что каждый `create` порождает ровно одно событие. Адаптер
или агрегат, чей `create` поднимет два события, сломает эту классификацию:
подлинное устаревшее обновление, случайно пришедшее с версией 1, будет принято
за дубликат. Допущение зафиксировано в doc-комментарии `check_version`.

## Разделение адаптера между задачами

```mermaid
flowchart LR
  MAIN["main.rs<br/>Arc::new(InMemory...)"]
  STATE["AppState<br/>Arc dyn Trait"]
  T1["Задача запроса 1"]
  T2["Задача запроса 2"]
  T3["Задача запроса N"]
  ADP["Один экземпляр адаптера<br/>Mutex HashMap"]

  MAIN --> STATE
  STATE -->|"Clone = +1 к счётчику"| T1
  STATE -->|"Clone = +1 к счётчику"| T2
  STATE -->|"Clone = +1 к счётчику"| T3
  T1 --> ADP
  T2 --> ADP
  T3 --> ADP
```

axum клонирует `AppState` на каждый запрос. `Arc` делает это увеличением
счётчика ссылок, а не копированием хранилища, — иначе каждый запрос получал бы
собственную пустую `HashMap`, и данные не сохранялись бы между вызовами.

## Выбор примитива синхронизации

Используется `std::sync::Mutex`, а не `tokio::sync::Mutex`. Это корректно
**ровно потому**, что охранник блокировки нигде не удерживается через `.await`:
каждый метод захватывает блокировку, выполняет синхронную работу и освобождает
её до возврата.

Добавление `.await` внутрь заблокированного участка сделало бы это рассуждение
неверным и создало бы риск остановки рабочего потока Tokio. Это единственное
изменение, которое ломает текущую корректность, — и оно неочевидно при беглом
чтении, поэтому вынесено сюда.

## Известное ограничение

`InMemoryVehicleOwnershipRepository` индексирует записи по
`VehicleOwnershipId`, поэтому правило «одно открытое владение на автомобиль»
нельзя выразить ограничением ключа. Оно проверяется методом
`has_open_ownership` **перед** записью, что оставляет окно между чтением и
записью. Под настоящей конкурентностью два запроса могут оба увидеть свободный
автомобиль и оба записаться.

Закрыть это окно должен частичный уникальный индекс в БД, которой нет.
