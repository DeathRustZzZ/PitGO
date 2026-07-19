# 11. Тестовая архитектура

## Назначение

Показать, как устроено тестирование: где юнит-тесты, где интеграционные, где
моки и где настоящие адаптеры — и что именно проверяет каждый уровень.

## Что представлено

Все 89 тестов, распределённые по четырём уровням, с указанием тестового
двойника.

## Как читать

Слева направо — от самого изолированного уровня к самому связному. Чем правее,
тем больше настоящих компонентов участвует.

## Три уровня тестирования

```mermaid
flowchart LR

  subgraph L1["Уровень 1 — Домен (50 тестов)"]
    D1["customer/aggregate_tests.rs"]
    D2["vehicle/aggregate_tests.rs"]
    D3["vehicle_ownership/aggregate_tests.rs"]
    D4["vehicle_ownership/state.rs<br/>inline tests"]
    DR["Без рантайма<br/>Без репозитория<br/>Без async"]
  end

  subgraph L2["Уровень 2 — Приложение (17 тестов)"]
    A1["tests/customer.rs<br/>5 тестов"]
    A2["tests/vehicle.rs<br/>5 тестов"]
    A3["tests/ownership.rs<br/>7 тестов"]
    MOCK["Мок-репозитории<br/>с внедрением сбоев"]
  end

  subgraph L3["Уровень 3 — Инфраструктура (13 тестов)"]
    I1["tests/customer_repository.rs"]
    I2["tests/vehicle_repository.rs"]
    I3["tests/vehicle_ownership_repository.rs"]
    REAL["Настоящие адаптеры<br/>+ настоящие агрегаты"]
  end

  subgraph L4["Уровень 4 — HTTP-граница (9 тестов)"]
    B1["tests/error.rs<br/>маппинг ApplicationError в ApiError"]
    WIRE["into_response + разбор JSON<br/>контракт как у клиента"]
  end

  D1 --> DR
  D2 --> DR
  D3 --> DR
  D4 --> DR

  A1 --> MOCK
  A2 --> MOCK
  A3 --> MOCK

  I1 --> REAL
  I2 --> REAL
  I3 --> REAL

  B1 --> WIRE
```

## Кто с чем работает

```mermaid
flowchart TD

  subgraph mocks["Моки (только в application)"]
    MC["MockCustomerRepository<br/>saved_ids, save_error"]
    MV["MockVehicleRepository<br/>saved_ids, save_error"]
    MO["MockOwnershipRepository<br/>has_active, has_active_error,<br/>save_error, saved_ids"]
  end

  subgraph ports["Порты"]
    PC["CustomerRepository"]
    PV["VehicleRepository"]
    PO["VehicleOwnershipRepository"]
  end

  subgraph real["Настоящие адаптеры"]
    RC["InMemoryCustomerRepository"]
    RV["InMemoryVehicleRepository"]
    RO["InMemoryVehicleOwnershipRepository"]
  end

  subgraph tested["Что проверяется"]
    HC["CreateCustomerHandler"]
    HV["CreateVehicleHandler"]
    HO["StartVehicleOwnershipHandler"]
    LOCK["Оптимистичная блокировка"]
    INV["Инвариант владения"]
  end

  MC -.->|"impl"| PC
  MV -.->|"impl"| PV
  MO -.->|"impl"| PO
  RC -.->|"impl"| PC
  RV -.->|"impl"| PV
  RO -.->|"impl"| PO

  PC --> HC
  PV --> HV
  PO --> HO

  RC --> LOCK
  RV --> LOCK
  RO --> LOCK
  RO --> INV
```

## Что проверяет каждый уровень

**Уровень 1 — домен.** Инварианты агрегатов в изоляции: одна команда = одно
событие = +1 к версии; идемпотентность (`NoChange` вместо ошибки при повторе);
запрещённые машиной состояний переходы; проверки permit. Выполняются без Tokio
и без хранилища — практическая выгода от того, что домен свободен от
ввода-вывода.

**Уровень 2 — приложение.** Оркестрация с подставными репозиториями. Моки
позволяют то, чего не даёт настоящее хранилище: **внедрение сбоев**. Тесты
`handle_propagates_storage_failure` и `handle_propagates_version_conflict`
проверяют, что обработчик пробрасывает ошибку репозитория, а не поглощает её;
`handle_does_not_save_on_repository_error` — что при доменном отказе запись не
выполняется вовсе.

**Уровень 3 — инфраструктура.** Настоящий адаптер и настоящие агрегаты вместе.
Проверяет то, чего не может подтвердить ни один из предыдущих уровней:
совпадает ли арифметика версий в адаптере с версиями, которые действительно
порождают агрегаты.

## Ключевой интеграционный тест

```mermaid
sequenceDiagram
  autonumber
  participant T as second_start_on_same_vehicle_is_rejected_by_real_repository
  participant H as StartVehicleOwnershipHandler
  participant R as InMemoryVehicleOwnershipRepository (настоящий)
  participant A as VehicleOwnership

  T->>R: Arc::new(настоящий репозиторий)
  T->>H: new(Arc::clone as Arc dyn Trait)

  T->>H: handle(первая команда).await
  H->>R: has_open_ownership → false
  H->>A: start → Ok
  H->>R: save → Ok
  Note over A: статус PendingVerification

  T->>H: handle(вторая команда, тот же vehicle_id).await
  H->>R: has_open_ownership
  R->>R: is_open() для PendingVerification → true
  R-->>H: true
  H->>A: start(snapshot занят)
  A--xH: ActiveOwnershipAlreadyExists
  H--xT: ApplicationError::Ownership
```

Это регрессионный тест исходного дефекта задачи 001. **Мок его поймать бы не
смог**: ошибка была в том, как настоящий репозиторий классифицировал
ожидающую запись, а мок возвращал бы заранее заданный `has_active`. Поэтому
тест намеренно использует подлинный адаптер, а не двойник.

## Структура тестовых модулей

| Крейт | Расположение | Подключение |
|---|---|---|
| `domain` | рядом с агрегатом, `#[path]` + `#[cfg(test)]` | `mod aggregate_tests` |
| `application` | `src/tests/`, `#[cfg(test)] mod tests` в `lib.rs` | корректно закрыт гейтом |
| `backend` | `src/tests/`, `#[cfg(test)] mod tests` в `main.rs` | корректно закрыт гейтом |
| `infrastructure` | `src/tests/`, `pub mod tests` в `lib.rs` | **гейта нет** |

Расхождение в последней строке: `infrastructure` объявляет `pub mod tests;`
без `#[cfg(test)]`. Тестовый код в релизные сборки не попадает — внутренние
модули закрыты гейтом, — но пустой публичный модуль `tests` экспортируется из
крейта в любой сборке. `application` и `backend` делают это правильно.

**Побочный эффект выноса тестов из модуля.** Тесты `backend` живут в
`src/tests/error.rs`, а не внутри `error.rs`, и потому физически не могут
дотянуться до приватных полей `ApiError`. Статус читается из готового
`Response`, а не из поля структуры, — то есть проверяется в точности то, что
увидит клиент. Ограничение видимости здесь работает как проектное средство, а
не как помеха.

## Чего в тестах нет

**Сквозных HTTP-тестов нет.** Контракт ошибок покрыт (9 тестов уровня 4), но
ни один тест не поднимает axum-роутер и не проверяет маршрут целиком. Непокрыты
остаются: `main.rs` со сборкой роутера и CORS, десериализация тел запросов,
конвертация `OwnershipTypeDto → OwnershipType`, коды успешных ответов и сами
`routers/*`. Всё это проверено только компилятором.

Заполнить этот пробел — предмет задачи 005 из `docs/ai/18.07.2026/TASKS/`.
Тесты уровня 4 намеренно сделаны через `into_response()` с разбором JSON, чтобы
подготовить приём, который там понадобится.

**Тестов на конкурентность нет.** Окно между чтением и записью в
`StartVehicleOwnershipHandler` никак не проверяется. Тест, запускающий два
одновременных `start` на один автомобиль, скорее всего показал бы, что оба
проходят, — но такого теста нет.
