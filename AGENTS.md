# Agent Instructions

## Architecture

- `packages/core`: Rust library + Axum wrapper. Owns domain rules, SQLite, migrations, seed, backup, restore, repositories.
- `packages/cli`: Rust CLI. Calls core in-process for `migrate`, `seed`, `backup`, `restore`, `check`.
- `packages/app`: Nuxt 4 + Nuxt UI. UI and thin Nitro gateway only; calls Rust over HTTP.
- Root `Cargo.toml`: Rust workspace. Root `package.json` + `turbo.json`: JS task orchestration.
- Roadmap: [`docs/plans/2026-09-05-rust-core-erp.md`](docs/plans/2026-09-05-rust-core-erp.md).

## Commands

```powershell
pnpm install
pnpm run build
pnpm run test
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed
cargo run -p logholizon-cli -- check
```

## Rules

- Rust owns all persistence and domain logic. Never add SQL or business rules to Nitro handlers.
- Nuxt calls Rust through `packages/app/server/core/client.ts`.
- Keep API contracts versioned under `/v1`; map Rust errors consistently.
- Validate all external input at HTTP and CLI boundaries.
- Use parameterized SQL and transactions.
- Migrations are embedded and forward-only; never edit an applied migration.
- Backup SQLite with `VACUUM INTO`; never copy a live database file.
- Restore is destructive: require explicit `--force`, validate integrity, preserve rollback path.
- Keep entities metadata-driven; do not hardcode ERP modules in reusable UI.
- Keep workflow linear; no auth, D1, branching, canvas, or multi-sheet Excel without explicit scope change.
- Do not add NuxtHub, Drizzle, libsql, or a Rust SDK crate unless architecture changes explicitly.
- Use `pnpm` for Node tasks, `cargo` for Rust tasks. Commit lockfiles.
- Add one focused test for non-trivial logic. Run relevant gates after changes.

## Generated and local files

Do not commit `target/`, `.data/`, `.nuxt/`, `.output/`, `node_modules/`, `.turbo/`, or `.env*`.

## Scope boundaries

- Core changes belong under `packages/core`.
- CLI command changes belong under `packages/cli`.
- App UI belongs under `packages/app/app`; gateway routes under `packages/app/server/api`.
- Keep gateway handlers thin: parse, validate, call client, map response.
