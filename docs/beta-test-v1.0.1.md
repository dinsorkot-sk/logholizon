# Beta Test — v1.0.1

**Branch**: `v1.0.1` @ `80eb82a`  
**Verified**: 2026-10-05 (local, WSL Ubuntu, Rust 1.88.0, Node 22.17.0, pnpm 11.9.0)

## Readiness Gates

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✅ Clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Clean |
| `cargo test --workspace -- --test-threads=1` | ✅ 157 passed, 0 failed |
| `pnpm install --frozen-lockfile` | ✅ Clean |
| `pnpm --dir packages/app run test` (Vitest) | ✅ 6 files, 33 tests passed |
| `pnpm --dir packages/app run check` (vue-tsc) | ✅ Clean |
| `pnpm --dir packages/app run build` | ✅ Built (8.74 MB) |
| `cargo build --workspace --release` | ✅ Built |
| E2E Playwright (`E2E_APP_MODE=preview`) | ✅ 11/11 passed |
| CLI smoke on a fresh DB | ✅ See below |

CLI smoke (fresh `CORE_DATABASE_URL`): `migrate` (twice, idempotent) →
`seed --demo` → `check` → `backup` (`VACUUM INTO`) → `restore` without
`--force` refused → `restore --force` (rollback copy preserved under
`backups/pre-restore-*.db`) → `check` = `database ok`.

## Beta Setup

### Docker (recommended for hosted beta)

```bash
docker compose up --build
```

Open http://localhost:3000. A fresh volume shows the first-run admin setup
form; no demo users are created. Data lives in the `logholizon-data` volume.

### Local

```bash
pnpm install
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed --demo
pnpm run dev
```

### Desktop (Tauri)

See `packages/desktop/README.md`. First run creates `admin` / `admin12345`
plus demo data.

## Test Accounts

| Source | Username | Password | Role |
|--------|----------|----------|------|
| `seed --demo` | `admin` | `admin123` | admin |
| `seed --demo` | `demo` | `demo1234` | user |
| Desktop first run | `admin` | `admin12345` | admin |

These are public defaults. Do not use `seed --demo` on any instance reachable
from the internet; reset them with `logholizon-cli reset-password` before
sharing a desktop build.

## What to Test

1. **Module Builder** — create a new domain (e.g. Vehicle Management):
   entities, fields, relations, forms, views, actions → publish → enable.
2. **Runtime CRUD** — create/edit/delete records, search, filter, sort,
   saved views, CSV/Excel import and export.
3. **Permissions** — create a custom role, assign it, confirm denied actions
   and hidden fields for non-admin users.
4. **Workflow** — run records through linear transitions, approve/reject,
   check state history.
5. **Automation / notifications / webhooks** — trigger on record and workflow
   events; check execution logs.
6. **Reports and dashboards** — build a report with grouping/aggregation and a
   dashboard with KPI and chart widgets.
7. **Ready-made packages** — install an ERP package from `packages/erp/`.
8. **Admin** — users, audit log, observability, backup/restore.

## Known Limitations

- Workflow is linear only (no branching or canvas), by design.
- The UI v2 redesign (`docs/design/design-v2.md`) is a draft and not part of
  this beta.
- Latest GitHub Actions status for `v1.0.1` was not checked here; confirm CI is
  green on GitHub before inviting testers.

## Reporting Issues

Open a GitHub issue with: version/branch, deployment (Docker, local, or
desktop), steps to reproduce, expected vs actual result, and the request ID
from the error response or the admin Observability page.
