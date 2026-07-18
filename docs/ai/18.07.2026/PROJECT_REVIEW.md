# PROJECT_REVIEW — PitGO

Дата ревью: 2026-07-18. Read-only аудит по состоянию `main` (HEAD `3a31193`, рабочее дерево чистое).
Предыдущее ревью: [docs/pitgo_review_fable5.md](../pitgo_review_fable5.md) (2026-07-15) — часть его замечаний уже закрыта, см. раздел «Прогресс с прошлого ревью».

---

## 1. Высокоуровневое резюме

PitGO — учебная платформа управления СТО. Монорепозиторий: Rust-бэкенд (Hexagonal / Ports & Adapters + DDD) и React/TypeScript-фронтенд. Реализованы три вертикальных среза — Customer, Vehicle, VehicleOwnership — от домена до HTTP, на in-memory хранилище. Домен глубоко спроектирован в `docs/domain/` и реализован осознанно узким MVP-срезом (стратегия scope-freeze).

**Общий вердикт: фундамент здоровый.** Направление зависимостей чистое, домен свободен от инфраструктуры, агрегаты инкапсулированы и хорошо оттестированы, все проверки (fmt, clippy `-D warnings`, tests, frontend check) зелёные. Главные долги — один содержательный баг инварианта, недомоделированные ошибки репозитория, нулевое покрытие HTTP-слоя и полная отвязанность фронтенда от бэкенда.

## 2. Обзор текущей архитектуры

```
crates/
  shared/          # AggregateVersion, ChangeOutcome, PendingEvent/EventEnvelope,
                   # ActionContext, define_id! (типобезопасные UUID-обёртки)
  domain/          # Агрегаты Customer, Vehicle, VehicleOwnership; статусы, события,
                   # permits, snapshot, доменные ошибки (thiserror)
  application/     # Команды, хендлеры use-case'ов, порты репозиториев (traits),
                   # ApplicationError / RepositoryError
  infrastructure/  # In-memory репозитории (Mutex<HashMap>) с optimistic locking
backend/           # Axum: main.rs (composition root, CORS), routers/*, error.rs (ApiError)
frontend/          # React 19 + Vite + TS; полностью на моках
```

Направление зависимостей (проверено по манифестам и `use`): `domain → shared`; `application → domain`; `infrastructure → application + domain`; `backend` — composition root. Нарушений нет.

Замечание о размещении портов: repository-трейты лежат в **application** (`crates/application/src/*/ports.rs`), а `CLAUDE.md` и `docs/domain/rust-mapping.md` обещают их в domain. Само размещение — легитимный вариант гексагональной архитектуры; проблема в расхождении документации с кодом.

## 3. Обнаруженные крейты/модули

| Крейт | Содержимое | Тесты |
|---|---|---|
| `shared` | ids (`define_id!`), aggregate (версия/ChangeOutcome), event (PendingEvent/EventEnvelope), audit (ActionContext) | 0 |
| `domain` | customer (aggregate, state, error, event, permit), vehicle (то же), vehicle_ownership (+ snapshot) | 49 |
| `application` | customer/vehicle/ownership: commands, handlers, ports; error.rs | 17 |
| `infrastructure` | 3 in-memory репозитория | 3 |
| `backend` | main.rs, error.rs, routers/{customer,vehicle,vehicle_ownership} | **0** |
| `frontend` | features (customer, vehicle, landing, schedule), pages (CRM-раздел на моках), shared/ui | **0** |

## 4. Реализованные фичи

Backend (HTTP, in-memory):
- `GET /health`
- `POST /customers`, `GET /customers/{id}` (создание в Draft, чтение с 404)
- `POST /vehicles`, `GET /vehicles/{id}`
- `POST /vehicles/{vehicle_id}/ownerships` (start ownership с проверкой инварианта через snapshot)
- Маппинг ошибок в JSON `{error, message}`: 409 / 422 / 404 / 500
- CORS для `http://localhost:5173`

Домен (реализовано, но не всё подключено к API):
- Полный lifecycle-паттерн: приватные поля, команды, `raise()` с инкрементом версии, no-op семантика (`ChangeOutcome::NoChange`), буфер `pending_events`
- `Customer::activate` / `Vehicle::activate` с `ActivationPermit` (валидация id/версии/срока) — **нет application-хендлеров и роутов**
- `VehicleOwnership::verify` / `::end` с `OwnershipPeriod` — **нет application-хендлеров и роутов**

Frontend: лендинг, регистрация клиента, гараж, CRM-раздел `/app` (dashboard, orders, clients, cars, inventory, reminders) — **всё на моках**, к бэкенду не подключено.

## 5. Сильные стороны

1. Чистое направление зависимостей; домен без serde/HTTP/SQL-артефактов (проверено grep).
2. Образцовая инкапсуляция агрегатов: приватные поля, изменение только именованными командами, идемпотентность (`NoChange`), версия +1 на событие.
3. Доменные тесты (49) — поведенческие, покрывают переходы, ошибки permit, события, идемпотентность.
4. Application-тесты (17) с ручными моками — проверяют и happy path, и propagation ошибок, и «не сохраняем при ошибке».
5. Разделение `PendingEvent` (в агрегате) и `EventEnvelope` (на границе персистентности) — грамотный event-дизайн.
6. Типобезопасные ID через `define_id!`, `#[repr(transparent)]`.
7. Дисциплина процесса: pre-commit hook (fmt+clippy+test+frontend check), conventional commits, PR-флоу, всё зелёное.
8. Осознанная документация: scope-freeze, style guide, полные доменные модели.

## 6. Прогресс с прошлого ревью (2026-07-15)

Закрыто: GET-роуты + 404-путь; optimistic locking во всех трёх in-memory репозиториях; явный маппинг `OwnershipError` по вариантам (blanket `_ → 422` устранён); nested route `/vehicles/{id}/ownerships`; CORS; async-порты через `async-trait` (репозитории готовы к SQLx); тест на дубликат создания.

Осталось из прошлого ревью: семантические коды ошибок; дыра инварианта «одно активное владение»; события не извлекаются; `AlreadyExists` отсутствует (дубль ловится, но косвенно — см. 7.3); backend без тестов; гигиена манифестов; фронт на моках.

## 7. Слабые стороны и проблемы по слоям

### 7.1 Домен

- **Дыра инварианта (главный содержательный баг, совместно с infrastructure).** `OwnershipEligibilitySnapshot` строится из `has_active_ownership`, который считает только `OwnershipStatus::Active`. Новое владение рождается в `PendingVerification` → можно последовательно создать *два* владения на один автомобиль (оба pending), затем `verify()` оба и получить два `Active`. Инвариант «не более одного активного владения» дыряв уже сейчас, а после подключения `verify` к API станет доступен снаружи. → Задача 001.
- `Clone` на агрегатах — задокументированный техдолг для in-memory-хранилища (снять при переходе на Postgres).
- Доменные события никогда не извлекаются в проде: `pull_pending_events()` вызывается только в тестах; события остаются внутри сохранённой копии агрегата. → Фаза 2.
- Activation-флоу спроектирован (permit, ошибки), но политик (`CustomerActivationPolicy`) нет — permit создать некому, кроме тестов. → Фаза 2.

### 7.2 Application

- `ApplicationError` знает только `Repository` и `Ownership`: `CustomerError`/`VehicleError` подключить некуда — activation невозможно провести через слой без расширения enum. → Фаза 2.
- TOCTOU в `StartVehicleOwnershipHandler`: `has_active_ownership` и `save` — два отдельных обращения к репозиторию; между ними другой запрос может создать владение. Для in-memory это окно мало, для Postgres решается уникальным частичным индексом + обработкой ошибки БД. Осознанно отложить до Фазы 3, но зафиксировать.
- `GetCustomerHandler`/`GetVehicleHandler` не покрыты тестами (тестируются только create-хендлеры).
- Нет use-case'ов: get ownership (порт `find_by_id` есть, хендлера нет), verify/end ownership, activate customer/vehicle.

### 7.3 Infrastructure

- **Повторное создание маскируется под конфликт версий.** В `save()` нет понятия «создание»: дубликат POST с тем же UUID даёт `VersionConflict {expected: 2, actual: 1}` → клиент получает 409 с сообщением про «конфликт при обработке», хотя семантика — «ресурс уже существует». Варианта `RepositoryError::AlreadyExists` нет. → Задача 002.
- Логика version-check предполагает «каждый save = ровно +1 к версии»: команда, поднявшая 2 события за один вызов (версия +2), будет ошибочно отвергнута. Сейчас таких команд нет, но допущение нигде не зафиксировано. → Задача 002 (зафиксировать/ослабить).
- Покрытие минимально: по 1 тесту на репозиторий (только дубликат create). Не покрыты: `find_by_id` (found/not found), `has_active_ownership` (единственный адаптер с настоящей логикой фильтрации!), конфликт при stale-обновлении. → Задачи 001/002.
- `lib.rs` экспортирует `pub mod tests` — тестовый модуль виден в публичном API крейта (мелочь, но лишнее).

### 7.4 Backend / API

- **0 тестов.** Роутеры, маппинг ошибок, формат JSON-тела, 404/409-пути не проверяются ничем. `main.rs` не разделяет построение `Router` и запуск сервера — приложение нетестируемо через `tower::ServiceExt::oneshot`. → Задача 005.
- Коды ошибок дублируют HTTP-статус (`"conflict"`, `"not_found"`) вместо семантики (`active_ownership_already_exists`, `version_conflict`): клиент не может различить причины в пределах одного статуса. → Задача 003.
- 201 возвращает `Json<String>` («Customer created successfully») — ни представления ресурса, ни заголовка `Location`. → Фаза 2.
- Ошибки axum-экстракторов (невалидный JSON, Content-Type) отдают plain-text дефолт, а не `{error, message}` — контракт ошибок неполный.
- Хендлеры создаются на каждый запрос (`CreateCustomerHandler::new(...)` внутри роутера) — работает, но состояние могло бы держать готовые хендлеры; не приоритет.
- Адрес/порт захардкожены (`127.0.0.1:3000`), конфигурации нет (`dotenvy` объявлен и не используется).

### 7.5 Frontend

- Полностью на моках: `apiFetch` из `shared/api/client.ts` реально не используется; vite-прокси на бэкенд нет; `VITE_API_BASE_URL` по умолчанию `/api`, бэкенд слушает `:3000` — связка не настроена (CORS на бэкенде уже готов).
- Контракт не совпадает: фронтовый `Customer` (fullName/phone/email, статус `onboarding`) ≠ бэкендовый (голый `customer_id`, `Draft/Active`). Интеграция потребует согласования DTO. → Фаза 4.
- Два параллельных UI-кита: `shared/ui/Badge/` (CSS-модули, PascalCase) и `shared/ui/badge.tsx` (shadcn-стиль, Tailwind) — и две структуры страниц (`features/*/pages` и `pages/*`). Дрейф структуры.
- Нет ни одного теста (vitest не подключён), нет `npm test`.
- Бандл 542 kB (> лимита предупреждения) — нет code splitting.

### 7.6 Гигиена workspace

- Edition расщеплён: `workspace.package` объявляет `2021`, но backend/application/infrastructure хардкодят `2024` (и код инфраструктуры использует let-chains, т.е. фактически требует 2024). Унифицировать на 2024 через workspace.
- Мёртвые зависимости: `serde` в `domain` и `shared` не используется (и открывает дверь случайному `#[derive(Serialize)]` на агрегате); `sqlx`, `dotenvy` объявлены в workspace и не используются нигде.
- Локальные пути объявляются то через `workspace = true`, то напрямую `{ path = ... }`; `infrastructure` дублирует `chrono = "0.4"` в dev-deps вместо workspace-наследования.

## 8. Пробелы тестирования (сводно)

| Область | Состояние |
|---|---|
| domain: агрегаты | ✅ 49 тестов, сильное покрытие |
| application: create/start-хендлеры | ✅ 17 тестов |
| application: Get-хендлеры | ❌ 0 |
| infrastructure: дубликат create | ✅ по 1 тесту |
| infrastructure: `find_by_id`, `has_active_ownership`, stale-update conflict | ❌ 0 |
| backend: роутеры, маппинг ошибок, JSON-контракт | ❌ 0 |
| Интеграционные тесты полного флоу (HTTP) | ❌ 0 |
| frontend | ❌ 0 (vitest отсутствует) |

## 9. Результаты сборки и проверок (2026-07-18, локально)

| Команда | Результат |
|---|---|
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ |
| `cargo test --workspace` | ✅ 69 passed (domain 49, application 17, infrastructure 3, backend 0) |
| `npm --prefix frontend run check` (lint + prettier + tsc + vite build) | ✅ (предупреждение: чанк 542 kB > 500 kB) |

Падений нет — отдельная задача «починить сборку» не требуется.

## 10. Архитектурные риски

1. **Инвариант владения дыряв** (7.1) — единственный баг, ломающий бизнес-правило; дешевле закрыть до расширения Ownership-флоу.
2. **Семантика ошибок репозитория недомоделирована** (7.3) — при переходе на SQLx каждое неявное допущение (`+1 к версии», «create = update») придётся выражать в SQL; лучше сделать контракт явным на in-memory, пока он дёшев.
3. **HTTP-контракт ничем не закреплён** — без тестов backend любой рефакторинг слоёв может незаметно поменять публичное API.
4. **Событийная модель наполовину** — события копятся, но не извлекаются; при переходе к outbox это станет несущей конструкцией, а пока это мёртвый код, создающий ложное чувство готовности.
5. **Разрыв контракта фронт/бэк** — чем дольше фронт живёт на своих моках, тем дороже стыковка.

## 11. Рекомендованное направление развития

Порядок: **стабилизировать существующий срез → расширить домен на готовых рельсах → персистентность → фронт.**

- **Фаза 1 (стабилизация)**: закрыть инвариант владения; явная ошибка `AlreadyExists`; семантические коды ошибок API; довести Ownership-срез (GET); интеграционные HTTP-тесты с выносом `Router` из `main`. Итог: закреплённый тестами публичный контракт и честные ошибки.
- **Фаза 2 (безопасное расширение домена)**: activation-флоу Customer (policy → permit → handler → route, расширение `ApplicationError`), verify/end ownership через API, извлечение доменных событий при save (+ in-memory event log), представление ресурса в 201 + Location, гигиена манифестов.
- **Фаза 3 (персистентность)**: PostgreSQL/SQLx-репозиторий для одного агрегата (Customer), миграции, конфигурация через env, версионный UPDATE ... WHERE version, частичный unique index для инварианта владения (закрывает TOCTOU), снятие `Clone` с агрегатов.
- **Фаза 4 (интеграция фронтенда)**: vite-прокси/базовый URL, переключение customer-среза с мока на `apiFetch`, согласование DTO, vitest + первые тесты, унификация UI-кита.

Детализация — в [ROADMAP.md](ROADMAP.md); задачи Фазы 1 — в [TASKS/](TASKS/).
