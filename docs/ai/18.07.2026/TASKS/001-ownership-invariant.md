# Task 001 — Закрыть дыру инварианта «одно активное владение»

## Goal

Инвариант «не более одного активного владения на автомобиль» сейчас проверяет только статус `Active`, но новое владение рождается в `PendingVerification`. Нужно, чтобы «занимающим» автомобиль считалось любое *открытое* владение (`PendingVerification` или `Active`), и чтобы это правило было выражено в домене, а не в фильтре репозитория.

## Why this matters

Это единственный найденный баг, ломающий бизнес-правило: можно последовательно создать два владения на один автомобиль (оба pending), затем подтвердить оба и получить два `Active`. Задача учит главному в DDD: инвариант — знание домена; репозиторий лишь отвечает на вопрос, сформулированный доменом. Заодно — первый опыт согласованного изменения через все слои (domain → application → infrastructure) с поведенческими тестами.

## Current context

- `VehicleOwnership::start` проверяет `OwnershipEligibilitySnapshot::no_active_ownership_exists()` — [crates/domain/src/vehicle_ownership/aggregate.rs](../../../crates/domain/src/vehicle_ownership/aggregate.rs), [snapshot.rs](../../../crates/domain/src/vehicle_ownership/snapshot.rs).
- Snapshot строит `StartVehicleOwnershipHandler` из ответа порта `has_active_ownership` — [crates/application/src/ownership/handlers.rs](../../../crates/application/src/ownership/handlers.rs), [ports.rs](../../../crates/application/src/ownership/ports.rs).
- In-memory репозиторий фильтрует строго по `OwnershipStatus::Active` — [crates/infrastructure/src/vehicle_ownership_repository.rs](../../../crates/infrastructure/src/vehicle_ownership_repository.rs) (`has_active_ownership`).
- Статусы: `PendingVerification`, `Active`, `Ended` — [crates/domain/src/vehicle_ownership/state.rs](../../../crates/domain/src/vehicle_ownership/state.rs).

## Scope

Allowed to change:

- `crates/domain/src/vehicle_ownership/state.rs` (новый метод-классификатор статуса + тест)
- `crates/domain/src/vehicle_ownership/snapshot.rs` (терминология «open» вместо «active», если решишь переименовать)
- `crates/application/src/ownership/ports.rs`, `handlers.rs` (имя/семантика метода порта)
- `crates/infrastructure/src/vehicle_ownership_repository.rs` (фильтр)
- Тесты: `crates/application/src/tests/ownership.rs`, `crates/infrastructure/src/tests/vehicle_ownership_repository.rs`, `crates/domain/src/vehicle_ownership/aggregate_tests.rs`

Not allowed to change:

- Агрегаты Customer и Vehicle и их срезы
- `backend/` (HTTP-контракт этой задачей не меняется; текст сообщения ошибки — можно)
- Публичный формат ответов API
- `frontend/`

## Step-by-step plan

1. Напиши в домене падающий тест, фиксирующий правило: владение в `PendingVerification` «занимает» автомобиль (например, тест на новый метод статуса).
2. Добавь в `OwnershipStatus` метод-классификатор (например, `is_open()`), возвращающий `true` для `PendingVerification | Active`, `false` для `Ended`. Используй exhaustive match — чтобы будущий статус (`Disputed`, `Rejected`) заставил компилятор напомнить о решении.
3. Переименуй метод порта `has_active_ownership` → отражающий новую семантику (например, `has_open_ownership`), обнови doc-комментарий: что именно считается «открытым» и почему.
4. Обнови in-memory репозиторий: фильтр через доменный метод из шага 2, а не через сравнение с конкретным вариантом.
5. Обнови `StartVehicleOwnershipHandler` и snapshot (терминологию — по вкусу, поведение snapshot не меняется: он по-прежнему отвечает «можно ли начать владение»).
6. Тесты infrastructure: pending-владение блокирует создание второго; `Ended` — не блокирует; владение на *другой* автомобиль — не блокирует.
7. Тест application: два `start` подряд на один `vehicle_id` через реальный in-memory репозиторий или мок → второй получает `OwnershipError::ActiveOwnershipAlreadyExists`.
8. Прогони все проверки из Acceptance criteria.

## Acceptance criteria

- `cargo fmt --all --check` проходит
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- `cargo test --workspace` проходит
- Добавлены тесты: классификатор статуса (domain), фильтр репозитория (infrastructure, минимум 3 случая из шага 6), сценарий «второй start отклонён» (application)
- Никакого несвязанного рефакторинга
- Границы Clean Architecture сохранены: правило «какие статусы занимают автомобиль» живёт в domain, репозиторий его только применяет

## Review checklist

- Решение простое? (ожидаемый diff — десятки строк, не сотни)
- Правило классификации статусов — в domain, а не в infrastructure?
- Application-логика не протекла в API-хендлеры?
- Ошибки явные и осмысленные? (имя `ActiveOwnershipAlreadyExists` можно оставить или уточнить — реши осознанно и одинаково везде)
- Тесты проверяют поведение (второй start отклонён), а не реализацию (какой вариант enum в фильтре)?
- Нет лишней абстракции? (не нужен отдельный «policy-объект» ради одного метода)

## Hints

- Начни с домена: `is_open()` на `OwnershipStatus` — зеркало уже существующего `OwnershipPeriod::is_open()`. Подумай, не выразить ли одно через другое, и почему статус и период — разные источники истины (это стоит одной строки в doc-комментарии).
- Переименование метода порта — это изменение *контракта* между application и infrastructure: компилятор сам покажет все места. Обрати внимание, как мало их окажется — это и есть ценность портов.
- В snapshot переименовывать `no_active_ownership_exists` не обязательно — но если оставляешь старое имя, проверь, что doc-комментарий не врёт о новой семантике.
- Будущий SQL-эквивалент фильтра: `WHERE vehicle_id = $1 AND status IN ('pending_verification', 'active')` — держи это в голове при выборе имени метода.

## Files likely involved

- crates/domain/src/vehicle_ownership/state.rs
- crates/domain/src/vehicle_ownership/snapshot.rs
- crates/application/src/ownership/ports.rs
- crates/application/src/ownership/handlers.rs
- crates/application/src/tests/ownership.rs
- crates/infrastructure/src/vehicle_ownership_repository.rs
- crates/infrastructure/src/tests/vehicle_ownership_repository.rs

## Suggested commit message

fix(ownership): treat pending ownership as occupying the vehicle
