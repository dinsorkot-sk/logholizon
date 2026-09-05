# Agent Instructions

## Project

Nuxt 4 + Nuxt UI 4 ERP gateway. Rust core owns SQLite. Follow the plan in [`../../docs/plans/2026-09-05-rust-core-erp.md`](../../docs/plans/2026-09-05-rust-core-erp.md). Follow UI rules in [`design.md`](design.md).

## Commands

- Install: `pnpm install`
- Develop: `pnpm run dev`
- Build: `pnpm run build`
- Preview: `pnpm run preview`
- Test: `pnpm run test`
- Rust core: `cargo run -p logholizon-cli -- migrate`, `cargo run -p logholizon-cli -- seed`

Use pnpm only. `package.json` pins `pnpm@11.9.0`.

## Structure

- `app/pages/`: Nuxt routes and page UI
- `app/components/`: reusable UI components
- `app/composables/`: client-side state and logic
- `server/api/`: thin Nitro gateway routes to Rust core
- `server/core/`: Rust core HTTP client
- `tests/`: Vitest tests

## Rules

- TypeScript in app, Rust in core. No domain SQL in Nitro handlers.
- Preserve metadata-driven design; avoid hardcoded ERP entities.
- Use Nuxt UI components where specified by `design.md`; prefer raw `<table>` over `UTable`.
- Use `USlideover` for contextual editing; avoid unnecessary modal/page navigation.
- Keep workflow linear and list-based; no drag-and-drop/canvas.
- Keep MVP scope: no auth, field-level permissions, branching workflows, or multi-sheet Excel unless explicitly requested.
- Rust core owns SQLite schema, migrations, seed, backup, and restore.
- Validate trust-boundary input. Add focused tests for non-trivial logic.
- Run `pnpm run build` after configuration or production-impacting changes.

## Known Pitfalls

- Do not add NuxtHub, Drizzle, or libsql dependencies. Persistence lives in `packages/core`.
- The Rust database defaults to `.data/core.db` via `CORE_DATABASE_URL`.
- `pnpm run test` may report no test files until tests are added.
- Nuxt may emit non-blocking rolldown declaration or Node export warnings during builds; distinguish warnings from failures.
