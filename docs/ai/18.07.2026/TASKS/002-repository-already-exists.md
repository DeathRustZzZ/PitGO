# Task 002 — Явная ошибка AlreadyExists в репозиториях

## Goal

Повторное создание агрегата с тем же ID сейчас отклоняется, но косвенно — как `VersionConflict {expected: 2, actual: 1}`. Нужно ввести явный вариант `RepositoryError::AlreadyExists` и возвращать его во всех трёх in-memory репозиториях, когда сохраняется *свежесозданный* агрегат поверх уже существующего. Конфликт версий остаётся для настоящих stale-обновлений.

## Why this matters

«Ресурс уже существует» и «кто-то обновил ресурс раньше тебя» — разные ситуации для клиента API: первая — ошибка запроса (повторный POST), вторая — сигнал перечитать и повторить. Пока обе прячутся за одним вариантом, честный HTTP-маппинг (задача 003) невозможен. Задача учит моделированию ошибок как части контракта порта и показывает скрытое допущение текущего кода («каждый save увеличивает версию ровно на 1»), которое при переходе на SQL придётся выражать явно.

## Current context

- `RepositoryError` (два варианта: `VersionConflict`, `StorageFailure`) — [crates/application/src/error.rs](../../../crates/application/src/error.rs). Обрати внимание: doc-комментарий у `VersionConflict` врёт («entity already exists») — след этой самой немоделированной ситуации.
- Три одинаковых `save()` с version-check: [crates/infrastructure/src/customer_repository.rs](../../../crates/infrastructure/src/customer_repository.rs), [vehicle_repository.rs](../../../crates/infrastructure/src/vehicle_repository.rs), [vehicle_ownership_repository.rs](../../../crates/infrastructure/src/vehicle_ownership_repository.rs).
- Свежесозданный агрегат имеет версию 1 (`create` поднимает одно событие поверх `AggregateVersion::INITIAL` = 0).
- Существующие тесты на дубликат ожидают `VersionConflict` — [crates/infrastructure/src/tests/](../../../crates/infrastructure/src/tests/) (по одному на репозиторий).
- `From<ApplicationError> for ApiError` в [backend/src/error.rs](../../../backend/src/error.rs) матчит `RepositoryError` исчерпывающе — новый вариант потребует минимальной ветки, иначе компиляция упадёт.

## Scope

Allowed to change:

- `crates/application/src/error.rs` (новый вариант + исправить врущий doc-комментарий `VersionConflict`)
- Три файла репозиториев в `crates/infrastructure/src/`
- Тесты infrastructure (обновить существующие, добавить новые)
- `backend/src/error.rs` — только минимальная ветка для нового варианта (например, 409 с текущим generic-сообщением); семантические коды — задача 003

Not allowed to change:

- Сигнатуры портов (`save`/`find_by_id` остаются как есть — см. Hints, почему)
- Домен (агрегаты, версии, события)
- Роутеры, DTO, формат тела ответа
- `frontend/`

## Step-by-step plan

1. Добавь вариант `AlreadyExists` в `RepositoryError` (реши: нужен ли ему payload — например, строковый ID — и зачем; посмотри, что несут соседние варианты).
2. Исправь doc-комментарий `VersionConflict`.
3. Почини компиляцию `backend/src/error.rs`: `AlreadyExists` → 409 (generic-сообщение, коды — в задаче 003).
4. В каждом `save()` раздели два случая: (а) в хранилище уже есть агрегат, а пришёл свежесозданный (версия 1) → `AlreadyExists`; (б) в хранилище есть агрегат, пришло обновление с непоследовательной версией → `VersionConflict`. Вынеси проверку так, чтобы три репозитория не разъехались (см. Hints).
5. Зафиксируй допущение «+1 за save» одной строкой doc-комментария там, где проверка живёт.
6. Обнови существующие тесты дубликата: теперь ожидается `AlreadyExists`.
7. Добавь тесты на настоящий `VersionConflict`: сохрани агрегат, «обнови» устаревшую копию (создай агрегат, клонируй, примени команду к оригиналу и сохрани, затем попробуй сохранить клон с командой) — или проще: сохрани версию 1, затем попробуй сохранить ещё раз объект версии 3 (см. Hints).
8. Добавь тест: повторный `save` того же объекта без изменений — реши, что должно произойти, и закрепи тестом (это осознанное решение, не случайность).
9. Прогони все проверки.

## Acceptance criteria

- `cargo fmt --all --check` проходит
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- `cargo test --workspace` проходит
- Для каждого из трёх репозиториев есть тесты: дубликат create → `AlreadyExists`; stale-обновление → `VersionConflict`
- Никакого несвязанного рефакторинга
- Границы сохранены: `RepositoryError` остаётся в application (контракт порта), детали хранения — в infrastructure

## Review checklist

- Решение простое? Три репозитория используют одну и ту же логику проверки, а не три чуть разные копии?
- Доменные правила не утекли в репозиторий? (репозиторий сравнивает версии, но не знает про статусы/команды)
- Application-логика не протекла в API-хендлеры?
- Ошибки явные и осмысленные? `AlreadyExists` и `VersionConflict` невозможно перепутать по имени и сообщению?
- Тесты поведенческие? (проверяют «какая ошибка», а не «как устроен HashMap»)
- Нет избыточной абстракции? (не нужно generic-трейта «VersionedStore» ради дедупликации трёх функций — достаточно приватной функции-хелпера или одинакового кода с общим тестовым паттерном)

## Hints

- Как отличить create от update без изменения сигнатуры порта: у свежесозданного агрегата версия всегда 1. `stored.exists() && incoming.version == 1` ⇒ это повторное создание. Это *эвристика*, работающая, пока `create` поднимает ровно одно событие — потому и нужен doc-комментарий из шага 5. Альтернатива — разделить порт на `insert`/`update` — честнее, но трогает порты, хендлеры и моки во всех срезах; обсуди в ревью, но в этой задаче не делай.
- Хелпер для проверки версии удобно положить рядом с репозиториями (приватный `fn check_version(stored: Option<&AggregateVersion>, incoming: AggregateVersion) -> Result<(), RepositoryError>` или аналог). Но осторожно: у трёх репозиториев разные типы агрегатов — обобщай по `AggregateVersion`, а не по агрегату.
- Для теста stale-обновления вспомни, что `Customer::activate` требует permit (см. `crates/domain/src/customer/aggregate_tests.rs`, там есть `valid_permit`) — с `VehicleOwnership` проще: `verify()` не требует permit.
- Существующий тест `rejects_duplicate_customer_create` уже почти готов — меняется только ожидаемая ошибка.

## Files likely involved

- crates/application/src/error.rs
- crates/infrastructure/src/customer_repository.rs
- crates/infrastructure/src/vehicle_repository.rs
- crates/infrastructure/src/vehicle_ownership_repository.rs
- crates/infrastructure/src/tests/customer_repository.rs
- crates/infrastructure/src/tests/vehicle_repository.rs
- crates/infrastructure/src/tests/vehicle_ownership_repository.rs
- backend/src/error.rs

## Suggested commit message

feat(application): introduce AlreadyExists repository error
