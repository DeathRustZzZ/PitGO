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
  domain/          # Aggregates, value objects, domain events, repository ports (traits)
  application/     # Use-case orchestration, authorization, UnitOfWork coordination
  infrastructure/  # PostgreSQL repos (SQLx), event store, outbox dispatcher
  shared/          # Cross-cutting utilities
backend/           # Axum HTTP entry point
frontend/          # React 19 + Vite + TypeScript
```

Dependencies flow inward: `infrastructure → application → domain`. The `domain` crate has zero infrastructure dependencies.

### Domain model (designed, partially implemented)

The domain is documented extensively in `docs/domain/`. Key bounded contexts:

- **Customer** — lifecycle state machine with multiple aggregates: `Customer` (onboarding/activation), `CustomerContactBook` (phone/email/Telegram), `CustomerProfile`, `CustomerPreferences`, `CustomerConsentLedger`
- **Vehicle** — `Vehicle` (specs/identity) and `VehicleOwnership` (ownership records)
- **Identity Registry** — separate bounded context enforcing global uniqueness of identity keys (phone, email, etc.)

### Key patterns from the domain docs

- **Type-safe IDs**: UUID newtypes via `define_id!` macro in `crates/domain/src/ids.rs`. All entity IDs use this pattern.
- **Optimistic locking**: `AggregateVersion` for concurrency control on all aggregates.
- **Event envelopes**: Domain events wrapped in `EventEnvelope` with metadata (event_id, aggregate_version, actor, correlation_id, causation_id).
- **Transactional outbox**: Reliable event publishing coordinated with identity registry.
- **Reservation pattern**: Identity keys reserved before changes, released after confirmation.
- **Activation permits**: Short-lived versioned tokens for cross-aggregate eligibility checks.
- **No-op semantics**: Commands producing no state change return `NoChange` without incrementing aggregate version.

### Workspace dependencies

All dependency versions are centralized in the root `Cargo.toml` under `[workspace.dependencies]`. Crate `Cargo.toml` files use `dep.workspace = true` to inherit versions. Key deps: axum, sqlx (PostgreSQL), tokio, serde, chrono, uuid, thiserror, tracing.

## Code Conventions

- Comments and documentation in Russian.
- Aggregate roots have private fields; state changes only through command methods.
- Repository interfaces are traits defined in the domain layer.
- Frontend uses ESLint + Prettier with format-on-save. TypeScript strict mode enabled (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`).
