# Dependencies

Этот файл фиксирует зависимости проекта и причину их наличия. При добавлении новой зависимости нужно обновлять этот документ.

## Rust workspace

Источник: `Cargo.toml`.

| Dependency | Version | Назначение |
| --- | --- | --- |
| `async-trait` | `0.1` | async methods в trait contracts, если понадобится для repository/service traits |
| `axum` | `0.8.9` | HTTP framework для backend |
| `chrono` | `0.4` + `serde` | дата и время |
| `dotenvy` | `0.15` | загрузка `.env` конфигурации |
| `serde` | `1` + `derive` | serialization/deserialization на границах системы |
| `sqlx` | `0.8` | PostgreSQL access, migrations, async runtime integration |
| `thiserror` | `2.0.18` | structured error enums |
| `tokio` | `1` | async runtime |
| `tracing` | `0.1` | structured logging/tracing |
| `tracing-subscriber` | `0.3` | настройка tracing subscriber |
| `uuid` | `1` + `serde`, `v4` | UUID и typed identifiers |

## Rust crates

### `crates/domain`

Текущие зависимости:

- `chrono`
- `serde`
- `thiserror`
- `uuid`

Назначение: чистая доменная модель, typed identifiers, value objects и domain errors.

### `crates/application`

Текущих зависимостей нет.

Планируемое назначение: use cases, command/query handlers, transaction boundaries.

### `crates/infrastructure`

Текущих зависимостей нет.

Планируемое назначение: PostgreSQL repositories, external integrations, persistence mapping.

### `backend`

Текущих зависимостей нет.

Планируемое назначение: composition root для HTTP backend.

## Frontend

Источник: `frontend/package.json`.

### Runtime dependencies

| Dependency | Version | Назначение |
| --- | --- | --- |
| `react` | `^19.2.6` | UI library |
| `react-dom` | `^19.2.6` | React DOM renderer |

### Development dependencies

| Dependency | Version | Назначение |
| --- | --- | --- |
| `@eslint/js` | `^10.0.1` | ESLint base rules |
| `@types/node` | `^24.12.3` | Node.js TypeScript types |
| `@types/react` | `^19.2.14` | React TypeScript types |
| `@types/react-dom` | `^19.2.3` | React DOM TypeScript types |
| `@vitejs/plugin-react` | `^6.0.1` | Vite React plugin |
| `eslint` | `^10.3.0` | linting |
| `eslint-plugin-react-hooks` | `^7.1.1` | React Hooks lint rules |
| `eslint-plugin-react-refresh` | `^0.5.2` | React Refresh lint rules |
| `globals` | `^17.6.0` | predefined global variables for ESLint |
| `prettier` | `^3.6.2` | formatting |
| `typescript` | `~6.0.2` | TypeScript compiler |
| `typescript-eslint` | `^8.59.2` | TypeScript ESLint integration |
| `vite` | `^8.0.12` | frontend build tool/dev server |
