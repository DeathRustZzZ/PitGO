# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PitGO is a car service station (СТО) management platform. Rust/TypeScript monorepo with a DDD backend and React frontend. Early-stage educational project — domain models are documented but mostly unimplemented.

## Commands

### Rust (from project root)

```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all tests
cargo test -p domain             # Run tests for a single crate
cargo test -p application ownership  # Run tests matching "ownership" in the application crate
cargo fmt --all --check          # Check formatting
cargo fmt --all                  # Apply formatting
cargo clippy --workspace --all-targets --all-features -- -D warnings  # Lint
cargo run -p backend             # Run backend
```

### Frontend (from project root)

```bash
npm --prefix frontend install    # Install dependencies
npm --prefix frontend run dev    # Dev server (Vite)
npm --prefix frontend run build  # Production build (tsc + vite)
npm --prefix frontend run lint   # ESLint
npm --prefix frontend run format # Prettier (write)
npm --prefix frontend run format:check  # Prettier (check only)
npm --prefix frontend run check  # lint + format:check + build (used by pre-commit)
```

### Pre-commit hook

Configured in `.githooks/pre-commit`. Runs: `cargo fmt --check`, `cargo clippy`, `cargo test`, and `npm run check` for frontend. Activate with:

```bash
git config core.hooksPath .githooks
```

## Architecture

### Layer structure (Hexagonal / Ports & Adapters with DDD)

```
crates/
  shared/          # AggregateVersion, ChangeOutcome, PendingEvent, EventEnvelope, ID primitives
  domain/          # Aggregates, value objects, domain events, repository port traits
  application/     # Use-case handlers, commands, repository port interfaces (re-exported)
  infrastructure/  # PostgreSQL repos (SQLx) — adapters implementing domain traits
backend/           # Axum HTTP entry point
frontend/          # React 19 + Vite + TypeScript
```

Dependencies flow inward: `infrastructure → application → domain → shared`. The `domain` crate has zero infrastructure dependencies. `ARCHITECTURE.md` describes a future multi-service target; the current code is a single workspace with flat crates.

### Domain model (designed, partially implemented)

The domain is documented extensively in `docs/domain/`. Key bounded contexts:

- **Customer** — lifecycle state machine: `Customer` (onboarding/activation), `CustomerContactBook`, `CustomerProfile`, `CustomerPreferences`, `CustomerConsentLedger`
- **Vehicle** — `Vehicle` (specs/identity) and `VehicleOwnership` (ownership records)
- **Identity Registry** — separate bounded context enforcing global uniqueness of identity keys (phone, email, etc.)

### Module layout convention

Every domain concept follows this structure:

```
domain/src/<context>/
  aggregate.rs        # Aggregate root (private fields, command methods, raise())
  state.rs            # Status enum, value objects, sub-structs
  event.rs            # Domain event variants (versioned, e.g. VehicleOwnershipStartedV1)
  error.rs            # Domain error enum (thiserror)
  snapshot.rs         # Capability/eligibility objects passed into commands
  permit.rs           # Activation permits (if the context has one)
  aggregate_tests.rs  # Unit tests, included via #[path = "aggregate_tests.rs"]
  mod.rs

application/src/<context>/
  handlers.rs         # Use-case handlers (own Arc<dyn Repository>)
  commands.rs         # Command DTOs
  ports.rs            # Repository trait re-exported for this context
  mod.rs
```

Integration-style tests live in `crates/application/src/tests/<context>.rs` and use in-memory repository fakes.

### Key patterns

- **Type-safe IDs**: UUID newtypes via `define_id!` macro in `crates/domain/src/ids.rs`. All entity IDs use this pattern.
- **Optimistic locking**: `AggregateVersion` (`shared::aggregate`) for concurrency control. The version increments by exactly one per raised event.
- **Event buffering**: Aggregates buffer events as `PendingEvent<E>` (from `shared::event`) via a private `raise()` method. `raise()` pushes the event and advances the version atomically — every state change must go through it. Events are drained with `pull_pending_events()` by the persistence layer after save.
- **`EventEnvelope`**: Assigned at the persistence boundary, not inside the aggregate. It adds `event_id`, `aggregate_version`, `correlation_id`, `causation_id`, `actor_id`, and `stored_at`.
- **`ChangeOutcome`**: Commands that may be no-ops return `Result<ChangeOutcome, E>`. `ChangeOutcome::NoChange` means the aggregate was already in the requested state — no event raised, version unchanged, safe to retry.
- **Snapshot/capability pattern**: Cross-aggregate facts (e.g. "vehicle has no open ownership") are read by the application handler and packaged into a typed snapshot (`OwnershipEligibilitySnapshot`), which is passed into the aggregate command. The domain never performs I/O; the database's partial unique index is the ultimate concurrency guard.
- **Transactional outbox**: Reliable event publishing coordinated in the same transaction as aggregate save.
- **Reservation pattern**: Identity keys reserved before changes, released after confirmation.

### Workspace dependencies

All dependency versions are centralized in the root `Cargo.toml` under `[workspace.dependencies]`. Crate `Cargo.toml` files use `dep.workspace = true` to inherit versions. Key deps: axum, sqlx (PostgreSQL), tokio, serde, chrono, uuid, thiserror, tracing.

## Code Conventions

- Aggregate roots have private fields; state changes only through command methods, all routed through `raise()`.
- Repository interfaces are traits defined in the `application` layer (`ports.rs`) and implemented in `infrastructure`.
- Frontend uses ESLint + Prettier with format-on-save. TypeScript strict mode enabled (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`).
