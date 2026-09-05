# Rust Core ERP Implementation Plan

วันที่: 2026-09-05  
สถานะ: Active

## Boundary

- `packages/core`: Rust HTTP service, domain rules, SQLite schema/migrations, transactions.
- `packages/app`: Nuxt UI + public HTTP gateway. ไม่มี domain rule ซ้ำ.
- Contract: JSON versioned under `/v1`; UTC RFC 3339 timestamps; stable error codes.
- Rust ถือ SQLite คนเดียว. ตัด Drizzle/NuxtHub database tasks จากแผนเดิม.

## Phase 1 — Core foundation

1. Cargo crate + Axum server.
2. `GET /health`, `GET /v1/version`.
3. Config: `CORE_HOST`, `CORE_PORT`, `CORE_DATABASE_URL`.
4. SQLite connection/migration baseline.
5. Domain error envelope.

Gate:

```powershell
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Phase 2 — Metadata + Documents

1. Entity, field, option models.
2. Metadata CRUD.
3. JSON document payload validation.
4. Document CRUD/list/pagination.
5. Transactional audit primitives.

## Phase 3 — Workflow

1. Workflow/state/transition models.
2. Linear transition validation.
3. State read/transition endpoints.
4. Immutable audit history.

## Phase 4 — Nuxt gateway + UI

1. Single HTTP client in `packages/app/server/core/client.ts`.
2. Thin Nitro routes map public API to core `/v1`.
3. Entity manager + field editor.
4. Dynamic form/table.

## Phase 5 — PM vertical slice

1. Seed `work_order`, `pm_schedule`.
2. Create → submit → approve → done.
3. Dashboard counts.
4. Audit viewer.

## Phase 6 — Excel

1. One-sheet export/import.
2. 1000-row ceiling.
3. Preview/confirm.
4. Atomic import rollback.

## Constraints

- No real auth, D1, cloud deploy, branching workflow, visual builder, multi-sheet Excel.
- Rust service HTTP is local-first MVP, not microservice infrastructure.
- `ponytail:` Core starts separately for now; add one-command orchestration when app/core both have useful dev loops.
