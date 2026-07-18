# Task 003 — Семантические коды ошибок в API

## Goal

Поле `error` в JSON-ответах сейчас дублирует HTTP-статус (`"conflict"`, `"not_found"`, …) и не несёт информации. Нужно ввести стабильные machine-readable коды, различающие *причину* в пределах одного статуса (например, `already_exists` и `version_conflict` — оба 409), и закрепить весь маппинг `ApplicationError → ApiError` первыми unit-тестами в backend-крейте.

## Why this matters

Коды ошибок — публичный контракт: фронтенд по ним ветвит UX («такой клиент уже есть» vs «данные устарели, обновите страницу»), и менять их после публикации дорого. Задача учит проектировать error-контракт HTTP-слоя отдельно от внутренних enum'ов (не 1:1!), и впервые вводит тесты в backend-крейт — фундамент для задачи 005.

## Current context

- `ApiError`/`ErrorBody` и конструкторы со статусами — [backend/src/error.rs](../../../backend/src/error.rs). Сейчас код жёстко зашит в конструкторы: `conflict()` всегда пишет `error: "conflict"`.
- `From<ApplicationError> for ApiError` там же — исчерпывающий match по `RepositoryError` (включая `AlreadyExists` из задачи 002) и `OwnershipError`.
- `not_found` вызывается напрямую из роутеров ([backend/src/routers/customer.rs](../../../backend/src/routers/customer.rs), [vehicle.rs](../../../backend/src/routers/vehicle.rs)).
- В backend-крейте нет ни одного теста.
- Зависит от задачи 002 (вариант `AlreadyExists` уже существует и замаплен на 409 с generic-кодом).

## Scope

Allowed to change:

- `backend/src/error.rs` (сигнатуры конструкторов `ApiError`, маппинг, тесты)
- `backend/src/routers/*.rs` — только места вызова конструкторов `ApiError` (передача кода/сообщения)
- `backend/Cargo.toml` — dev-dependencies, если понадобятся для тестов (например, `serde_json`; версию — через workspace)
- Корневой `Cargo.toml` — только добавление в `[workspace.dependencies]`, если нужно

Not allowed to change:

- `crates/*` (application/domain/infrastructure не трогаем — это задача про HTTP-границу)
- Набор HTTP-статусов (какая ошибка каким статусом — уже решено и оттестировано руками; меняем только поле `error`)
- Успешные ответы (201-тела — отдельная задача Фазы 2)
- `frontend/`

## Step-by-step plan

1. Составь таблицу кодов (в doc-комментарии `error.rs`): для каждого варианта `ApplicationError` и каждого `not_found`-места — стабильный snake_case-код. Ориентир:
   - `RepositoryError::AlreadyExists` → 409, `already_exists`
   - `RepositoryError::VersionConflict` → 409, `version_conflict`
   - `RepositoryError::StorageFailure` → 500, `internal`
   - `OwnershipError::ActiveOwnershipAlreadyExists` → 409, `active_ownership_already_exists`
   - `OwnershipError::StatusDoesNotAllow` → 409, `ownership_status_does_not_allow`
   - `OwnershipError::PeriodEndBeforeStart` → 422, `ownership_period_invalid`
   - not found → 404, `customer_not_found` / `vehicle_not_found`
2. Реши, как конструкторы получают код: параметром (`ApiError::conflict(code, message)`) или отдельными фабриками на каждую ошибку. Выбери вариант, при котором невозможно забыть код.
3. Обнови маппинг и вызовы в роутерах.
4. Напиши unit-тесты маппинга в `backend/src/error.rs` (`#[cfg(test)] mod tests`): для каждой строки таблицы — проверка статуса и кода. Понадобится доступ к внутренностям `ApiError` из тестов — реши как (см. Hints).
5. Убедись, что сообщения (`message`) по-прежнему безопасны: не содержат внутренних деталей (текстов ошибок Mutex, версий и т.п.) — есть тест хотя бы на один такой случай (`StorageFailure("secret detail")` → в message нет "secret detail").
6. Прогони все проверки.

## Acceptance criteria

- `cargo fmt --all --check` проходит
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- `cargo test --workspace` проходит
- В backend-крейте появились unit-тесты: каждый вариант ошибки → ожидаемые (статус, код); утечка внутренних деталей в message исключена тестом
- Таблица кодов зафиксирована в doc-комментарии
- Никакого несвязанного рефакторинга
- Границы сохранены: коды — сущность HTTP-слоя; application/domain по-прежнему ничего не знают об HTTP

## Review checklist

- Решение простое? (нет отдельного крейта «error-codes», нет макросов)
- Доменные правила остались в домене? (backend только *переводит* ошибки, не интерпретирует бизнес-логику)
- Application-логика не протекла в API-хендлеры?
- Ошибки явные и осмысленные? Можно ли по коду однозначно понять причину, не глядя на message?
- Тесты поведенческие? (проверяют контракт «ошибка → статус+код», а не внутреннее устройство `ApiError`)
- Нет ли соблазна сделать коды 1:1 с вариантами enum автоматически (через `Display`/`strum`)? Это связало бы публичный контракт с внутренними именами — осознанно не делаем.

## Hints

- Для тестов есть два честных пути: (а) дать `ApiError` методы-геттеры `#[cfg(test)]`; (б) прогнать `into_response()` и разобрать тело — это ближе к реальному контракту, но требует извлечь body (`axum::body::to_bytes`, features уже есть) и `serde_json` в dev-deps. Вариант (б) заодно подготовит тебя к задаче 005.
- «Невозможно забыть код» проще всего достигается, если убрать публичные generic-конструкторы (`conflict(message)`) и оставить только специфичные фабрики (`ApiError::already_exists()`, `ApiError::customer_not_found()` …) с зашитыми кодом и сообщением. Меньше свободы у вызывающего — крепче контракт.
- `StatusDoesNotAllow(status)` интерполирует статус в message — это нормально (статусы публичны), но код должен оставаться стабильным, без интерполяции.
- Подумай (и напиши одну строку в doc-комментарии): что должен получить клиент при `already_exists` на POST — можно ли ему считать операцию идемпотентно успешной? Ответ повлияет на фронт в Фазе 4.

## Files likely involved

- backend/src/error.rs
- backend/src/routers/customer.rs
- backend/src/routers/vehicle.rs
- backend/src/routers/vehicle_ownership.rs
- backend/Cargo.toml (dev-deps, при выборе варианта (б))

## Suggested commit message

feat(backend): add semantic machine-readable error codes to api responses
