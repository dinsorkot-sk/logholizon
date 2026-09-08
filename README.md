# LOGHOLIZON

Metadata-driven ERP Runtime / Business Application Platform. LOGHOLIZON provides the runtime and builders; users create the business modules they need instead of relying on built-in ERP modules.

## Product Direction

LOGHOLIZON is **not a pre-built ERP with built-in Accounting, Inventory, Sales, HR, CRM, Manufacturing, or POS modules**. These are examples of modules a user can create.

The core product is a runtime for building ERP and business applications:

```text
User → Module Builder → User-Defined Modules → LOGHOLIZON Core Runtime → SQLite
```

A user should be able to create a module such as `Accounting`, `Dormitory`, `Vehicle Management`, or `Rental` without changing Rust for normal no-code use.

## Architecture

```text
┌──────────────────────────────────────────────────────────┐
│                    LOGHOLIZON CORE                       │
│                                                          │
│ Metadata │ Document │ Workflow │ Permission │ Formula    │
│ Automation │ Event │ Report │ API │ Audit │ Storage      │
└─────────────────────────┬────────────────────────────────┘
                          │
                    Module Runtime
                          │
                    Module Builder
                          │
          ┌───────────────┼───────────────┐
          ↓               ↓               ↓
      Accounting      Inventory       Dormitory
      User Module     User Module     User Module
```

- **`packages/core`** — Rust library + Axum HTTP service. Owns generic runtime capabilities, persistence, migrations, validation, transactions, and domain engines when deterministic invariants require native Rust.
- **`packages/cli`** — Rust CLI for `migrate`, `seed`, `backup`, `restore`, `check`.
- **`packages/app`** — Nuxt 4 + Nuxt UI. UI and thin Nitro gateway only; calls Rust over HTTP.

### Architectural Rules

- Core must remain business-domain agnostic. Do not hardcode Accounting, Inventory, Sales, HR, CRM, or another ERP module into Core.
- Every business module uses the same Module Runtime and metadata contracts.
- User-created modules are first-class tenant-scoped data, not source-code changes.
- Module definitions must support draft/publish, versioning, validation, and upgrade-safe changes.
- Normal module creation must require no Rust code.
- Native Rust engines are allowed only for deterministic domain invariants that cannot safely be expressed as generic metadata/rules.
- Modules communicate through stable commands, events, and contracts rather than direct Core/domain coupling.
## Features

- **Module Builder** — create business modules from metadata without writing Rust.
- **Entity Manager** — create entities and fields with configurable types, options, status, relations, and computed values.
- **Workflow Builder** — define lifecycle states and transitions per entity.
- **Dynamic Views & Forms** — search, filter, sort, pagination, saved views, form layout, validation, and audit history.
- **Permissions** — role-based entity and field access with hidden-field redaction.
- **Auth** — login, first-run admin setup, user management, and role-based UI.
- **Audit Log** — global record history with entity/action/search filters.
- **Excel/CSV** — single-entity CSV export/import plus multi-sheet `.xlsx` workbooks.
- **Backups** — manual and scheduled SQLite backups with staged restore.
- **Reporting & Analytics** — saved reports, charts, and aggregation through the runtime.
- **Notifications & Automation** — webhook-first notifications, rules, outbox, retry, and delivery history.

## Module Model

There are **no built-in ERP modules** in the target architecture. A module is a user-owned business definition consumed by the runtime.

```text
Module
├── Entities
├── Fields
├── Relations
├── Forms
├── Views
├── Workflows
├── Permissions
├── Roles
├── Reports
├── Dashboards
├── Actions
├── Formulas
├── Automations
├── Notifications
├── Webhooks
└── Settings
```

Example user-created modules:

```text
Accounting
├── Account
├── Journal Entry
└── Journal Line

Dormitory
├── Building
├── Room
├── Tenant
├── Contract
├── Meter
└── Rent Invoice

Vehicle Management
├── Vehicle
├── Driver
├── Maintenance
└── Fuel Record
```

These examples are **not shipped modules**. They demonstrate what users can build with the platform.

### Runtime vs Domain Engine

Most business behavior should be expressed through metadata, formulas, workflows, validations, actions, and automation. When a domain has deterministic invariants that require stronger guarantees, the module may use a native Rust domain engine behind a stable Core contract.

For example, a user-defined accounting module may have a Rust accounting engine that enforces balanced postings. The Accounting module itself is still user-created; the engine is an optional runtime extension, not a built-in ERP module.

## Quickstart (development)

Prerequisites: Rust, Node 22+, pnpm.

```powershell
pnpm install
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed --demo   # optional demo data
pnpm run dev                                  # core :8787 + app :3000
```

Open http://localhost:3000 — first run shows the admin setup form, or log in with the demo account (`demo` / `demo1234`).
## Development Commands

```powershell
# Rust
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed
cargo run -p logholizon-cli -- backup <path>
cargo run -p logholizon-cli -- restore <path> --force
cargo run -p logholizon-cli -- check

# App
pnpm --dir packages/app run dev
pnpm --dir packages/app run build
pnpm --dir packages/app run test
pnpm --dir packages/app run check
```

## Configuration

| Env | Default | Description |
|---|---|---|
| `CORE_HOST` | `127.0.0.1` | Bind host |
| `CORE_PORT` | `8787` | Bind port |
| `CORE_DATABASE_URL` | `sqlite://<root>/.data/core.db` | SQLite location |
| `CORE_BACKUP_INTERVAL_HOURS` | `24` | Scheduled backup interval; `0` disables it |
| `CORE_BACKUP_KEEP` | `7` | Number of backups to keep |

App uses `CORE_URL` (default `http://127.0.0.1:8787`).

## Design

See [`packages/app/design.md`](packages/app/design.md) for the UI specification and [`docs/plans/`](docs/plans/) for implementation plans.

## Roadmap

The roadmap prioritizes the **User-Defined Module Runtime**, not a collection of built-in ERP applications.

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
- [x] Multi-user collaboration polish
- [x] Field-level permissions
- [x] Multi-sheet Excel
- [x] Visual form layout designer
- [x] Notifications and webhook delivery
- [x] Reporting and analytics
- [x] Reference fields and relations
- [x] Computed fields
- [x] Module grouping

### Next: User-Defined Module Runtime

- [ ] **Module Definition Model** — canonical schema for modules, entities, fields, relations, forms, views, workflows, permissions, reports, menus, actions, automations, and settings.
- [ ] **Module Registry** — create, validate, enable, disable, publish, version, archive, and restore user modules.
- [ ] **Module Builder** — no-code UI for creating a module from scratch.
- [ ] **Dynamic Runtime API** — generic CRUD, queries, actions, and metadata APIs generated from module definitions.
- [ ] **Dynamic UI Runtime** — menus, lists, forms, dashboards, reports, and actions rendered from definitions.
- [ ] **Business Rules** — formulas, defaults, validations, numbering, conditions, and document policies.
- [ ] **Command/Action Engine** — transactional actions with permission, validation, audit, and event integration.
- [ ] **Event & Automation Engine** — lifecycle events, triggers, notifications, and cross-module automation.
- [ ] **Tenant Isolation** — module ownership, data isolation, permission enforcement, and safe references.
- [ ] **Module Versioning** — draft/publish lifecycle, definition migrations, compatibility checks, and rollback strategy.
- [ ] **Extension API** — optional Rust extensions for domain invariants that cannot be safely represented by generic rules.

### Acceptance Test

The first end-to-end milestone is that a user can create a **Vehicle Management** module without Rust:

```text
Create Module
  ↓
Create Vehicle entity
  ↓
Add fields: code, plate number, type, status
  ↓
Create Driver entity and relation
  ↓
Build form + list view
  ↓
Define workflow: Active → Maintenance → Retired
  ↓
Define roles and permissions
  ↓
Publish module
  ↓
Use generated UI/API to manage real records
```

## Docker

```bash
docker compose up --build
```

- App: http://localhost:3000
- Core: http://localhost:8787
- SQLite data persists in the `logholizon-data` volume at `/data/core.db`.

## Repository Boundaries

```text
packages/
├── core/       # Generic Rust runtime, persistence, engines, HTTP API
├── cli/        # Rust operational CLI
└── app/        # Nuxt 4 UI + thin Nitro gateway
```

Business modules must not become hardcoded folders inside `packages/core`. User-defined module definitions belong to the runtime's persisted metadata model. Optional native extensions may live behind explicit Core contracts.

## Design Principle

> **LOGHOLIZON provides the engine, not the business. The user defines the business.**

The success criterion is not how many ERP modules LOGHOLIZON ships. It is how many different business applications users can build from the same Core Runtime safely, predictably, and without source-code changes.
