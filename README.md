# LOGHOLIZON

Metadata-driven ERP platform. Define entities and fields in the UI, then manage records with workflows, permissions, saved views, audit history, and CSV import/export — backed by a Rust core.

## Architecture

```
┌─────────────────────┐      HTTP /v1       ┌──────────────────────┐
│  Nuxt 4 + Nuxt UI   │ ──────────────────► │  Rust (Axum + SQLite) │
│  (UI + thin gateway)│ ◄────────────────── │  (domain + persistence)│
└─────────────────────┘      JSON           └──────────────────────┘
```

- **`packages/core`** — Rust library + Axum HTTP service. Owns domain rules, SQLite schema/migrations, seed, backup/restore, repositories.
- **`packages/cli`** — Rust CLI for `migrate`, `seed`, `backup`, `restore`, `check`.
- **`packages/app`** — Nuxt 4 + Nuxt UI. UI and thin Nitro gateway only; calls Rust over HTTP.

## Features

- **Entity Manager** — create entities, fields (text/number/date/select), options, status field
- **Workflow Builder** — linear state machine (states + transitions) per entity
- **Dynamic list** — search, filter, sort, pagination, bulk actions, column visibility, saved views
- **Dynamic form** — dirty-state protection, validation, audit history per record
- **Permissions** — per-entity view/edit toggles by role (admin/user)
- **Auth** — login, first-run admin setup, user management, role-based UI
- **Audit Log** — global history with entity/action/search filters
- **Excel/CSV** — one-sheet export/import with preview and atomic rollback
- **Backups** — manual + scheduled (`VACUUM INTO`), staged restore, download
- **PM Dashboard** — open/overdue/done-this-week summary cards

## Quickstart (development)

Prerequisites: Rust, Node 22+, pnpm.

```powershell
pnpm install
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed --demo   # optional demo data
pnpm run dev                                  # core :8787 + app :3000
```

Open http://localhost:3000 — first run shows the admin setup form, or log in with the demo account (`demo` / `demo1234`).

## Quickstart (Docker)

```bash
docker compose up --build
```

- App: http://localhost:3000
- Core: http://localhost:8787
- SQLite data persists in the `logholizon-data` volume (`/data/core.db`)

## Commands

```powershell
# Rust
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed            # base entities/workflows
cargo run -p logholizon-cli -- seed --demo     # + demo users & records
cargo run -p logholizon-cli -- backup <path>
cargo run -p logholizon-cli -- restore <path> --force
cargo run -p logholizon-cli -- check

# App
pnpm --dir packages/app run dev
pnpm --dir packages/app run build
pnpm --dir packages/app run test
pnpm --dir packages/app run check
```

## Configuration (core)

| Env | Default | Description |
|---|---|---|
| `CORE_HOST` | `127.0.0.1` | Bind host |
| `CORE_PORT` | `8787` | Bind port |
| `CORE_DATABASE_URL` | `sqlite://<root>/.data/core.db` | SQLite location |
| `CORE_BACKUP_INTERVAL_HOURS` | `24` | Scheduled backup interval (0 = off) |
| `CORE_BACKUP_KEEP` | `7` | Number of backups to keep |

App: `CORE_URL` (default `http://127.0.0.1:8787`).

## Design

See [`packages/app/design.md`](packages/app/design.md) for the UI spec and [`docs/plans/`](docs/plans/) for the implementation plans.

## Roadmap

- [x] Core foundation (Rust + SQLite + migrations)
- [x] Metadata + documents + workflow
- [x] Nuxt gateway + dynamic UI
- [x] Excel/CSV export-import
- [x] Backup/restore + settings
- [x] Workflow builder + PM dashboard
- [x] Auth + roles + user management
- [x] Global audit log
- [x] Permissions + saved views
- [x] Scheduled backups + demo seed
- [x] Docker + CI
- [ ] Multi-user collaboration polish
- [ ] Field-level permissions
- [ ] Multi-sheet Excel