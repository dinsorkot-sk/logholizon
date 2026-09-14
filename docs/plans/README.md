# LOGHOLIZON — 100% Master Implementation Plan

## Product Goal

LOGHOLIZON is a universal metadata-driven Business Application Platform. It provides the runtime engine, not pre-built ERP business modules.

Users must be able to create Accounting, Inventory, HR, CRM, Sales, Vehicle Management, Dormitory, Manufacturing, POS, or any other business application through the Module Builder without changing Rust Core for normal no-code use.

## 100% Definition

The platform reaches 100% when a user can create a previously unknown business application end-to-end:

`Module -> Entities -> Fields -> Relations -> Forms -> Views -> Permissions -> Workflow -> Actions -> Formula/Validation -> Automation -> Reports -> Dashboard -> Publish -> Runtime`

No domain-specific Rust implementation is required for the normal user-created module path.

## Architecture Contract

```text
Nuxt 4 / Vue
    -> HTTP /v1 JSON
Rust Axum Runtime
    -> SQLite
```

- `packages/core`: Rust runtime, persistence, domain engine, API contracts.
- `packages/app`: Nuxt UI and thin Nitro gateway.
- `packages/cli`: CLI and administration.
- Rust owns persistence and domain logic.
- Core must remain business-domain neutral.
- Do not add hardcoded Accounting, Inventory, Sales, HR, CRM, POS, Manufacturing, Vehicle, Dormitory, or other ERP modules.
- User-created modules are first-class product data and use the same runtime contracts.
- Use parameterized SQL and transactions.
- Use forward-only migrations.

## Master Roadmap

### Phase 0 — Architecture Freeze — 5%
- Freeze Core/App/CLI boundaries.
- Freeze `/v1` API conventions.
- Freeze metadata ownership and persistence model.
- Freeze module lifecycle and tenant model.
- Update architecture documentation and `AGENTS.md` where required.

### Phase 1 — Module Definition & Registry — 10%
- Implement canonical `ModuleDefinition`.
- Module ID, name, description, icon, owner, tenant.
- Version, status, settings, dependencies.
- Draft / Review / Published / Enabled / Disabled / Archived lifecycle.
- Registry and lookup APIs.

### Phase 2 — Entity & Metadata Runtime — 20%
- Generic Entity definition.
- Generic Field definition.
- Text, long text, number, decimal, currency, percentage, integer.
- Boolean, date, datetime, time, select, multi-select.
- Email, phone, URL, JSON, file, image.
- Reference, computed, formula, auto-number.
- Required, unique, default, readonly, hidden, searchable, sortable, filterable, indexed.
- Precision, min/max, regex, help text.

### Phase 3 — Universal Relation Engine — 25%
- One-to-one, one-to-many, many-to-one, many-to-many.
- Self-reference and reverse relations where required.
- Relation metadata and foreign keys.
- Lookup/autocomplete and related records UI.
- Cascade/delete protection and relation validation.

### Phase 4 — Dynamic CRUD/API Runtime — 35%
- Generic create/read/update/delete for user-defined entities.
- Pagination, sorting, filtering, search.
- Bulk operations.
- Transactions and validation.
- Relation loading and computed fields.
- Stable `/v1` error and response contracts.
- No dependency on hardcoded business entities.

### Phase 5 — Dynamic UI Runtime — 45%
- Generic module/entity routes.
- Dynamic form renderer.
- Dynamic field renderers.
- Sections, tabs, columns and conditional UI.
- Dynamic list/table renderer.
- Search, filter, sort, pagination and saved views.
- Record detail, related records, timeline and audit.
- Generic actions and export/import integration.

### Phase 6 — Module Builder — 55%
- Module Builder.
- Entity Builder.
- Field Builder.
- Relation Builder.
- Form Builder.
- View Builder.
- Module settings.
- Preview mode.
- Publish flow.
- Builder must operate entirely on metadata.

### Phase 7 — RBAC & Permissions — 63%
- Role management.
- Module/entity CRUD permissions.
- Export/import permissions.
- Action/approval permissions.
- Field-level permissions.
- Record-level permissions.
- Permission evaluation must work for every user-created module.

### Phase 8 — Validation, Formula & Computed Runtime — 68% — COMPLETE (100%)
- Safe expression engine.
- Type checking and null handling.
- Arithmetic, comparison and boolean operators.
- Date and string functions.
- Conditional expressions.
- Formula dependency resolution.
- Circular dependency detection.
- Validation messages and execution safety.

### Phase 9 — Workflow Engine — 73% — COMPLETE (100%)
- Generic states and transitions.
- Conditions and role requirements.
- Approve/reject actions.
- State history.
- Transition validation.
- Workflow events.
- Keep the initial workflow model linear; do not add branching/canvas complexity without explicit scope change.

### Phase 10 — Actions & Events — 77% — COMPLETE (100%)
- Create/update/delete record actions.
- Change-status actions.
- Notification actions.
- Webhook actions.
- Formula actions.
- Document generation hooks where applicable.
- Record created/updated/deleted events.
- Workflow transition events.

### Phase 11 — Automation Engine — 81% — COMPLETE (100%)
- Event triggers.
- Record triggers.
- Workflow triggers.
- Scheduled triggers.
- Conditions.
- Action chains.
- Enable/disable.
- Retry and failure handling.
- Execution logs.

### Phase 12 — Notification & Webhook Runtime — 84% — COMPLETE (100%)
- In-app notifications.
- Email templates and variables.
- User/role targeting.
- Notification history.
- HTTP webhooks.
- Payload templates.
- Timeout/retry policy.
- Signing/secrets where required.
- Execution logs and isolation.

### Phase 13 — Generic Report Engine — 87% — COMPLETE (100%)
- Entity and relation data sources.
- Fields, filters, grouping and sorting.
- Aggregation.
- Calculated columns.
- Date-range filtering.
- Saved reports.
- Export.
- Reports must be metadata-driven, not domain-hardcoded.

### Phase 14 — Dashboard Builder — 89% — COMPLETE (100%)
- Dashboard definition.
- KPI widgets.
- Table/list widgets.
- Bar, line, pie and area charts.
- Grid layout.
- Filters.
- Module/user dashboard permissions.
- Saved dashboards.

### Phase 15 � Module Versioning & Upgrade � 91% � COMPLETE (100%)
- Semantic module versions.
- Immutable published versions.
- Metadata diff.
- Upgrade migration.
- Compatibility checks.
- Change history.
- Safe rollout and rollback strategy.
- Never mutate production metadata unsafely.

### Phase 16 — Module Package Import/Export — 93% — COMPLETE (100%)
- Export module metadata.
- Manifest and version.
- Dependencies.
- Import validation.
- Conflict detection.
- Migration planning.
- Preview before install.
- Install/enable/disable/uninstall lifecycle.

### Phase 17 - Tenant & Security Hardening - COMPLETE (100%)
- Tenant isolation in every runtime query.
- Module isolation.
- Permission isolation.
- File isolation.
- Webhook isolation.
- Audit isolation.
- Backup/restore safety.
- SQL injection protection.
- Expression sandboxing.
- SSRF protection.
- Secure file upload handling.

### Phase 18 — Audit & Observability — COMPLETE (100%)
- Audit log: who/what/when/target/before/after.
- Login and security events.
- Permission denial logs.
- Workflow history.
- Automation execution logs.
- Webhook execution logs.
- Runtime error logs.
- Request ID and correlation ID.
- Operational metrics sufficient to diagnose runtime failures.

Verified on `v0.0.32` (Phase D): every bullet maps to a storage table +
endpoint + UI surface. `_observability_log` records `security` events
(`register_success/failed`, `login_success/failed`, `logout`,
`permission_denied`), per-request entries with request/correlation IDs and
`runtime_error` on 5xx, plus `workflow` (`transition`), `automation`
(`automation_succeeded/failed`), `webhook` (`webhook_delivered/failed`), and
`report` (`report_run`) events. `/v1/admin/observability/logs` (filtered,
paginated) and `/v1/admin/observability/metrics` serve the admin
Observability page (`admin/observability.vue`); the audit log
(`/v1/audit`, `admin/audit.vue`), workflow history
(`/v1/documents/{id}/workflow-history`), automation executions
(`/v1/meta/automations/{id}/executions`), and notification deliveries
(`/v1/admin/notification-deliveries`) cover the remaining bullets.
`packages/core/tests/observability.rs` (7 tests) proves each event source;
`phase20_acceptance.rs` + `phase20_http.rs` prove the end-to-end flow.

### Phase 19 - Testing & Production Readiness - COMPLETE (100%)
- Unit tests for metadata, entity, field and relation engines.
- Formula and validation tests.
- Permission tests.
- Workflow and automation tests.
- API integration tests.
- Dynamic UI tests.
- Security tests.
- Migration/backup/restore tests.
- End-to-end user-defined module tests.
- CI quality gates: format, clippy, Rust tests, app tests/build.

### Phase 20 — 100% Acceptance Gate — 100%
Build and operate `Vehicle Management` entirely through the user-facing Module Builder.

Required entities:
- Vehicle
- Driver
- Maintenance
- Rental

Required capabilities:
- Custom fields and relations.
- Generated forms, lists and detail views.
- Role-based permissions.
- Workflow: Available -> Rented -> Returned.
- Automation: returned rental -> maintenance check.
- Reports: utilization, maintenance cost, rental revenue.
- Dashboard: total, available, rented, maintenance, revenue.
- Publish and enable module.
- Full CRUD through runtime.
- No Rust source changes for the module.

Then prove the same runtime can define an Accounting-like module without adding Accounting tables/routes/types to Core.

## ERP Hardcode Removal — VERIFIED (v0.0.26)

Legacy hardcoded ERP domains are out of Core:

- `migrations/0026_drop_legacy_erp.sql` drops all `_company/_currency/_fx/_tax`
  `_ledger/_invoice/_payment/_stock/_trade/_hr/_mfg/_pos` tables (forward-only).
- No Rust references to `gl_account/journal/invoice/stock_ledger/employee/
  payroll/bom/pos_order/trade_doc` remain in `packages/core/src/**`,
  `packages/app/**`, or `packages/cli/**`.
- All `/v1` routes are generic (entity IDs, module IDs, metadata IDs).
  `dashboard_pm`/`pm_summary` operate on any entity's status field.
- `seed()`/`seed_demo()` remain CLI-only demo data (work orders, PM
  schedules, inventory sample); never auto-run by the server.

Candidates originally listed (`GlAccount`, `JournalEntry`, `JournalLine`,
`Invoice`, `InvoiceLine`, `Payment`, `StockLedger`, `Employee`, `PayrollRun`,
BOM/manufacturing, POS, Customer, Sales) are now user-defined-module
territory, proven by `packages/core/tests/phase20_http.rs` (accounting-like
module with no Core changes).

## Canonical Baseline — v0.0.28 (Phase A — COMPLETE)

`v0.0.27` at `7f3b44d` is the canonical superset baseline. Verified by ancestry
check on 2026-09-13: `origin/main` (`024c2a2` CSV import preview/confirm),
`origin/dev` (`ec5d340` multi-sheet Excel import/export), and
`origin/audit/uxui` (`710e8e0` field position/status + UX) are all ancestors of
`HEAD` (`git merge-base --is-ancestor` = CONTAINED for all three, zero commits
behind). Feature presence confirmed in-tree: `preview_import`,
`preview_workbook_xlsx`/`confirm_workbook_xlsx`/`export_workbook_xlsx` in
`packages/core/src/http.rs` + `repository.rs`; `position`/`is_status` ordering
in `repository.rs` backed by `migrations/0004_field_position.sql` and
`migrations/0005_status_field.sql`.

CI inventory (`.github/workflows/ci.yml`): `rust` job runs
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
-D warnings`, `cargo test --workspace -- --test-threads=1`,
`cargo build --workspace --release`; `app` job runs `pnpm install
--frozen-lockfile` then `packages/app` `test`/`check`/`build`; `e2e` job needs
`[rust, app]` and runs Playwright Chromium `run e2e`.

Release discipline from `v0.0.28` onward: one sequential version branch per
phase (`v0.0.28` -> `v0.0.29` -> ...), branched from the previously completed
version branch only after commit + push succeed. Never reuse a completed
branch, never skip numbers, never force-push published phase history. `v0.0.28`
is docs-only (this section) plus a type-only repair: `pnpm app check` failed on
the `v0.0.27` baseline with 19 errors in
`packages/app/app/pages/admin/modules/[id].vue` (untyped `cloneDefinition()`
returning `any` poisoned 17 lambdas, plus 2 template narrowing errors); fixed
with type annotations only, no behavior change. All gates green on `v0.0.28`:
`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`,
`pnpm app test` (20/20), `pnpm app check`, `pnpm app build`.

## Production Auth and Security — v0.0.29 (Phase B — COMPLETE)

Branch `v0.0.29`, from `v0.0.28` tip `2a8d265`. HTTP + CLI boundary only;
no SQL or domain logic outside Rust Core.

- Session lifecycle: 7-day expiry enforced in `user_for_token`, logout
  deletes the token, password reset (admin HTTP `reset-password` or CLI
  `reset-password <username> <password>`) invalidates all user sessions
  and audit-logs `password_reset`.
- Brute-force protection: per-IP sliding-window limiter on
  login/register (`CORE_AUTH_RATE_LIMIT_MAX`/`CORE_AUTH_RATE_LIMIT_WINDOW_SECS`,
  `X-Forwarded-For` aware); failures only, success clears; over-budget
  returns generic `429 too_many_requests` (`AppError::TooManyRequests`).
- CORS lockdown: `cors_layer` moved into `http.rs` so every router
  (binary + tests) enforces it; `CORE_ALLOWED_ORIGINS` allowlist, empty =
  same-origin only, `"*"` = permissive dev-only with warning.
- Secrets/SSRF: inbound HMAC-SHA256 `verify_webhook` (constant-time),
  outbound SSRF block + 1MB/32-header caps kept, filename traversal block
  + 5MB attachment cap + MIME allowlist kept, observability `record`
  redacts passwords/tokens/secrets/hashes/signatures/bearer strings to
  `[redacted]` before persistence.
- Tests (`security.rs`, 20 total): rate-limiter unit, window expiry,
  client-IP keying, HMAC round-trip/tamper, HTTP 401 bypass, HTTP 429
  rate-limit, reset-invalidates-sessions, logout-invalidates, expiry
  rejection, CORS allowlist/deny, redaction unit + persistence, caps,
  tenant isolation. Escalation (403) covered in `health.rs`.

All gates green on `v0.0.29`: `cargo fmt --check`,
`cargo clippy --workspace --all-targets -D warnings`,
`cargo test --workspace -- --test-threads=1`, `pnpm app test` (20/20),
`pnpm app check`, `pnpm app build`.

## Final Quality Gate

100% requires all of the following:

- A user can create a new business domain unknown to Core.
- The module persists as metadata.
- Entities, fields and relations are runtime-defined.
- CRUD/API is generic.
- UI is generated from metadata.
- Permissions are generic.
- Workflow is generic.
- Formulas/validation are generic.
- Automation/events are generic.
- Reports/dashboards are generic.
- Modules can be published and versioned safely.
- Modules can be packaged/imported/exported.
- Tenant/security boundaries are enforced.
- Audit and observability are available.
- Tests prove the complete flow.
- No normal no-code module creation requires a Rust Core change.

## Ready-Made ERP — SHIPPED (100%)

Ready-made ERP solutions ship as **module packages** (metadata JSON), never
as hardcoded Rust — the Working Rule above still holds. Each package installs
through the same Module Runtime, lifecycle, and permission contracts as
user-created modules.

Shipped packages (`packages/erp/`, schema v2 — v1 still accepted):

- `accounting` — chart of accounts, journal entries + lines, post/void
  workflow, trial-totals report, Accounting Overview dashboard.
- `inventory` — products, warehouses, stock moves, confirm/cancel workflow,
  moves-by-type report, Inventory Overview dashboard.
- `sales` — customers, orders + lines (formula line totals), quote-to-cash
  workflow, revenue report, Sales Overview dashboard.
- `hr` — employees, leave requests (approve/reject), payroll runs
  (process/pay), headcount report, HR Overview dashboard.
- `pos` — stores, terminals, sale tickets + lines, pay/void workflow,
  sales-by-tender report, POS Overview dashboard.
- `manufacturing` — BOMs + lines, production runs (start/complete),
  output report, Manufacturing Overview dashboard.
- `dormitory` — buildings, rooms, residents, bookings (check-in/out),
  bookings report, Dormitory Overview dashboard.
- `vehicle` — fleet, drivers, rentals, maintenance (Phase 20 reference
  domain, also installable as a package).

Install and operate:

```bash
cargo run -p logholizon-cli -- install-module packages/erp/<name>.module.json
cargo run -p logholizon-cli -- list-modules --admin
```

or browse, preview (migration plan, conflicts, dependencies), and install
from the admin Solution Library (`/admin/solutions`, backed by
`GET /api/solutions` + `GET /api/solutions/:name`, JSON bundled at build
time). Package format and authoring conventions: `packages/erp/README.md`.

Platform capabilities added to support the suite (all generic, no domain
hardcode):

- Package schema v2: optional `relations`/`actions`/`automations`/
  `dashboards` sections with definition-level names, validated and
  materialized post-publish (`module_package.rs`).
- `transition`-trigger automations fire from `_workflow_event` rows
  (`automation::enqueue_events`).
- Entity short-name uniqueness scoped per module (`UNIQUE(module_id,
  name)`, migration `0043`) so independent modules share common names.
- Document writes accept every canonical field type; only server-computed
  `computed`/`formula` reject writes (`validate_field_value`).
- Admin Entity Manager gained Actions and Automations tabs; Observability
  records workflow/automation/webhook/report events.

Verification (v0.0.37, Phase I): fresh DB → migrate → seed → check →
install all 8 via CLI (all `enabled`) → live HTTP drill per module (CRUD +
workflow transition) → report aggregation → all 8 dashboards listed →
observability metrics/logs recording. Covered by
`packages/core/tests/erp_packages.rs` (10 tests: install + operate per
package, validation negative, name-scoping coexistence),
`packages/core/tests/fields.rs` (canonical write acceptance), and
`packages/app/tests/solutions.test.ts` (catalog shape).

## Working Rule

Do not increase the percentage by adding more built-in ERP features. Increase the percentage by making the runtime capable of representing those features as user-defined metadata.
