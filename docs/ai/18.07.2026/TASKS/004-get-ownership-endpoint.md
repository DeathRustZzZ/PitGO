# Task 004 — GET-эндпоинт для VehicleOwnership

## Goal

Срез VehicleOwnership не доведён: порт `find_by_id` существует, но нет ни application-хендлера, ни HTTP-роута — созданное владение невозможно прочитать. Нужно добавить `GetVehicleOwnershipHandler`, роут `GET /vehicles/{vehicle_id}/ownerships/{ownership_id}` и DTO ответа, по образцу уже готовых GET-срезов Customer/Vehicle.

## Why this matters

Это упражнение «доведи срез до симметрии» на самом богатом агрегате: в отличие от Customer/Vehicle, здесь DTO содержит enum (`OwnershipType`), статус из трёх значений и период с опциональной датой конца — придётся принять решения о сериализации, которые Customer-срез принять не заставлял. Плюс nested-маршрут поднимает вопрос, которого нет у плоских ресурсов: что делать, если владение найдено, но принадлежит *другому* автомобилю.

## Current context

- Образец готового GET-среза: `GetCustomerHandler` ([crates/application/src/customer/handlers.rs](../../../crates/application/src/customer/handlers.rs)) + `get_customer`/`CustomerResponse` ([backend/src/routers/customer.rs](../../../backend/src/routers/customer.rs)).
- Порт `VehicleOwnershipRepository::find_by_id` уже есть — [crates/application/src/ownership/ports.rs](../../../crates/application/src/ownership/ports.rs); in-memory реализация тоже — [crates/infrastructure/src/vehicle_ownership_repository.rs](../../../crates/infrastructure/src/vehicle_ownership_repository.rs).
- Данные агрегата: id, vehicle_id, owner_customer_id, ownership_type, status, period (started_at, ended_at: Option), version, created_at, updated_at — [crates/domain/src/vehicle_ownership/aggregate.rs](../../../crates/domain/src/vehicle_ownership/aggregate.rs).
- В роутере уже есть `OwnershipTypeDto` — но только `Deserialize` — [backend/src/routers/vehicle_ownership.rs](../../../backend/src/routers/vehicle_ownership.rs).
- Роуты регистрируются в [backend/src/main.rs](../../../backend/src/main.rs).
- `OwnershipStatusKind`/`OwnershipTypе` имеют/не имеют `Display` — проверь, что есть у статуса (`kind().to_string()` — так делает Customer).

## Scope

Allowed to change:

- `crates/application/src/ownership/handlers.rs` (новый хендлер)
- `crates/application/src/tests/ownership.rs` (тесты хендлера)
- `backend/src/routers/vehicle_ownership.rs` (роут-функция, Response-DTO, сериализация типа/статуса)
- `backend/src/main.rs` (регистрация роута)

Not allowed to change:

- Домен (`crates/domain`) — все нужные геттеры уже есть
- Порты и репозитории (`find_by_id` уже существует)
- Существующие эндпоинты и их контракты
- `frontend/`

## Step-by-step plan

1. Добавь `GetVehicleOwnershipHandler` в application по образцу `GetCustomerHandler`: принимает `VehicleOwnershipId`, возвращает `Result<Option<VehicleOwnership>, ApplicationError>`.
2. Тесты хендлера (мок уже есть в `tests/ownership.rs` — расширь его: `find_by_id` должен уметь возвращать заданное владение): найдено → `Some`, не найдено → `None`, ошибка репозитория → propagation.
3. Спроектируй `VehicleOwnershipResponse`: ownership_id, vehicle_id, owner_customer_id, ownership_type, status, started_at, ended_at (null для открытого периода), created_at, updated_at. Реши формат enum-полей — snake_case-строки, симметрично `OwnershipTypeDto` на входе.
4. Сериализация `OwnershipType` наружу: сделай выходной путь для DTO (см. Hints — не вешай `Serialize` на доменный enum).
5. Роут-функция `get_vehicle_ownership`: `Path<(Uuid, Uuid)>` для nested-пути; найдено → 200; не найдено → 404 (код из задачи 003, например `ownership_not_found` — добавь фабрику по образцу).
6. Реши случай «владение существует, но `ownership.vehicle_id() != vehicle_id` из пути»: правильный REST-ответ — 404 (ресурс `/vehicles/A/ownerships/X` не существует, даже если `/vehicles/B/ownerships/X` существует). Реализуй и закрепи это тестом (в этой задаче — на уровне выбора в код-ревью; HTTP-тест добавится в 005).
7. Зарегистрируй роут в `main.rs`.
8. Проверь руками: `cargo run -p backend`, затем POST владения и GET по обоим путям (свой/чужой vehicle_id).
9. Прогони все проверки.

## Acceptance criteria

- `cargo fmt --all --check` проходит
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- `cargo test --workspace` проходит
- Тесты application-хендлера: found / not found / repository error
- Случай «чужой vehicle_id в пути» реализован как 404 и отражён в doc-комментарии роут-функции
- Никакого несвязанного рефакторинга
- Границы сохранены: DTO и сериализация — только в backend; домен без serde

## Review checklist

- Решение простое? (хендлер — тонкая обёртка над портом, как у Customer)
- Доменные правила в домене? (роутер не интерпретирует статусы, только отображает)
- Application-логика не протекла в API? (проверка `vehicle_id` из пути — это HTTP-забота о форме URL или application-правило? Обоснуй выбор — оба ответа защитимы, важна осознанность)
- Ошибки явные? (404 неотличим для «нет такого id» и «чужой автомобиль» — это осознанное решение против enumeration, напиши об этом строку)
- Тесты поведенческие?
- Нет лишней абстракции? (не нужен mapper-слой ради одного DTO)

## Hints

- Не добавляй `serde` в domain ради ответа. Зеркальный путь к `OwnershipTypeDto::into_domain`: метод `from_domain(&OwnershipType) -> Self` на DTO (или отдельный выходной enum с `Serialize`). Для статуса хватит `status().kind().to_string()` — так уже делает Customer-срез; но сравни регистры: `Display` у `OwnershipStatusKind` даёт PascalCase («PendingVerification»), а вход принимает snake_case — реши, что отдаёшь наружу, и будь консистентен (это заметит фронтенд в Фазе 4).
- `Path<(Uuid, Uuid)>` извлекает оба сегмента кортежем — порядок совпадает с порядком в шаблоне пути.
- `ended_at: Option<DateTime<Utc>>` сериализуется в `null` автоматически — проверь, что тебя устраивает `"ended_at": null` (а не отсутствие поля).
- Не забудь: `period()` возвращает ссылку — поля периода публичные, бери напрямую.

## Files likely involved

- crates/application/src/ownership/handlers.rs
- crates/application/src/tests/ownership.rs
- backend/src/routers/vehicle_ownership.rs
- backend/src/error.rs (фабрика ownership_not_found)
- backend/src/main.rs

## Suggested commit message

feat(ownership): add get vehicle ownership endpoint
