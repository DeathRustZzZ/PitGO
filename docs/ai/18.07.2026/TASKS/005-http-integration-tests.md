# Task 005 — Интеграционные HTTP-тесты полного флоу

## Goal

Backend-крейт не имеет ни одного теста уровня HTTP: роуты, экстракторы, JSON-контракт и коды ошибок ничем не закреплены. Нужно вынести построение `Router` из `main()` в переиспользуемую функцию, добавить библиотечную цель крейта и написать интеграционные тесты через `tower::ServiceExt::oneshot`, покрывающие полный флоу (create → get) и ключевые ошибочные пути.

## Why this matters

Это финальный «замок» Фазы 1: всё, что стабилизировано в задачах 001–004 (инвариант, `already_exists`, семантические коды, GET владения), закрепляется тестами на уровне публичного контракта. После этого любой рефакторинг внутренних слоёв, ломающий API, будет пойман автоматически. Попутно — ключевой приём экосистемы: axum-приложение как тестируемый `tower::Service`, без запуска TCP-сервера.

## Current context

- `main.rs` делает всё сразу: собирает `AppState`, `Router`, CORS и запускает сервер — [backend/src/main.rs](../../../backend/src/main.rs). `AppState` объявлен там же и приватен.
- Роутеры — `backend/src/routers/`, ошибки — `backend/src/error.rs`. Всё это модули *бинарной* цели: интеграционные тесты из `backend/tests/` их не видят, пока нет `src/lib.rs`.
- Хранилище in-memory и создаётся в `main` — в тестах каждый Router получает свежие пустые репозитории (изоляция тестов бесплатно).
- Зависимости для тестов (`tower` с ServiceExt, `http-body-util`, `serde_json`) в workspace отсутствуют — их нужно добавить.
- Предполагается, что задачи 001–004 завершены (коды `already_exists`, `active_ownership_already_exists`, `*_not_found` существуют).

## Scope

Allowed to change:

- Новый `backend/src/lib.rs` (модули `error`, `routers`, `AppState`, функция сборки приложения) и сокращённый `backend/src/main.rs`
- Видимость (`pub`/`pub(crate)`) в `backend/src/*` — минимально необходимая
- Новый файл(ы) тестов `backend/tests/`
- `backend/Cargo.toml` (`[dev-dependencies]`) и корневой `Cargo.toml` (`[workspace.dependencies]`: tower, http-body-util, serde_json)

Not allowed to change:

- `crates/*` — вообще (если тест требует менять application/domain — это находка для отдельной задачи, запиши и не делай)
- Поведение эндпоинтов и формат ответов (тесты *фиксируют* текущий контракт, а не меняют его)
- Маршруты, статусы, коды ошибок
- `frontend/`

## Step-by-step plan

1. Создай `backend/src/lib.rs`: перенеси туда объявления `mod error; mod routers;`, `AppState` и новую функцию `pub fn app(state: AppState) -> Router` (роуты + CORS). Реши, что делает конструктор `AppState` (например, `AppState::in_memory()`), чтобы тесты не знали про конкретные репозитории.
2. Сократи `main.rs` до: init tracing → state → `app(state)` → bind → serve.
3. Убедись, что `cargo run -p backend` работает как раньше (ручной smoke: `GET /health`).
4. Добавь dev-зависимости: `tower` (для `ServiceExt::oneshot`), `http-body-util` (собрать тело ответа), `serde_json` (разобрать JSON). Версии — в `[workspace.dependencies]`, в крейте — `workspace = true`.
5. Напиши `backend/tests/api.rs` (хелперы: собрать приложение, выполнить запрос, распарсить JSON-тело). Минимальный набор сценариев:
   - `GET /health` → 200
   - Полный флоу: `POST /customers` → 201; `GET /customers/{id}` → 200, поля `customer_id`/`status: "Draft"` корректны
   - `GET /customers/{random}` → 404, тело `{error: "customer_not_found", ...}`
   - Повторный `POST /customers` с тем же id → 409, `error: "already_exists"`
   - Полный флоу владения: `POST /vehicles` → `POST /vehicles/{id}/ownerships` → 201; второй `POST ownerships` на тот же автомобиль → 409, `error: "active_ownership_already_exists"` (это HTTP-подтверждение задачи 001!)
   - `GET /vehicles/{vehicle_id}/ownerships/{id}` → 200; тот же ownership с чужим vehicle_id → 404 (закрепляет решение задачи 004)
6. Один тест на контракт ошибок экстрактора: `POST /customers` с невалидным JSON → какой статус и тело? Зафиксируй *текущее* поведение как характеризационный тест с комментарием, что унификация тела — кандидат в Фазу 2.
7. Прогони все проверки.

## Acceptance criteria

- `cargo fmt --all --check` проходит
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- `cargo test --workspace` проходит; в backend появились интеграционные тесты (минимум — сценарии шага 5)
- `cargo run -p backend` по-прежнему поднимает сервер (ручная проверка)
- Каждый тест собирает своё приложение (нет разделяемого состояния между тестами)
- Никакого несвязанного рефакторинга; поведение эндпоинтов не изменилось
- Границы сохранены: тесты ходят только через HTTP-интерфейс (`Request`/`Response`), не через прямые вызовы хендлеров application

## Review checklist

- Решение простое? (`lib.rs` — тонкий: без новых абстракций, только перенос + одна функция сборки)
- Доменные правила в домене? (тесты не дублируют доменные проверки, а проверяют их *видимость* через HTTP)
- Application-логика не протекла в API-хендлеры при переносе?
- Ошибки явные? (ассерты проверяют и статус, и `error`-код — не только статус)
- Тесты поведенческие и читаются как сценарии пользователя API?
- Нет лишней абстракции? (хелперы тестов — функции, не фреймворк; DSL не нужен)

## Hints

- Паттерн запроса: `app.clone().oneshot(Request::builder().method("POST").uri("/customers").header("content-type", "application/json").body(Body::from(json)).unwrap())`. `oneshot` потребляет сервис — поэтому `.clone()` (Router дешёв в клонировании) или собирай приложение заново.
- Тело ответа: `let bytes = response.into_body().collect().await.unwrap().to_bytes();` (из `http_body_util::BodyExt`), затем `serde_json::from_slice::<serde_json::Value>`.
- В тестах `unwrap`/`expect` уместны — это не production-код (см. REVIEW_CHECKLIST).
- CORS-layer в тестах не мешает; но подумай, где ему место — в `app()` или в `main()` — и почему (подсказка: захочешь ли ты в тестах проверять CORS-заголовки? Это тоже контракт).
- Общие хелперы нескольких тестовых файлов кладут в `tests/common/mod.rs` — но пока файл один, не создавай структуру заранее.
- `AppState` придётся сделать `pub`; поля можно оставить приватными, если есть конструктор.

## Files likely involved

- backend/src/lib.rs (новый)
- backend/src/main.rs
- backend/src/error.rs, backend/src/routers/*.rs (только видимость)
- backend/tests/api.rs (новый)
- backend/Cargo.toml
- Cargo.toml (workspace dev-deps)

## Suggested commit message

test(backend): add http integration tests via testable router
