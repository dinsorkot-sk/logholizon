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

## v0.1.0 — Stabilization Release — COMPLETE (100%)

Base: `origin/dev` at `d2128f5` (merge of `v0.0.37` Phase I + cross-platform
e2e runner fix). Cargo workspace already at `0.1.0`, matching the branch.

Scope (stabilization only, no new runtime features):

- Cross-platform `packages/app/tests/e2e/run.cjs`: detect `win32`, use
  `cargo` from `PATH` and spawn the `nuxt` binary directly on POSIX.
  Fixes CI `E2E (playwright)` timing out on `http://127.0.0.1:8788/health`.
- Release readiness: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace -- --test-threads=1`, `pnpm app test` (23/23),
  `pnpm app check`, `pnpm app build` all green on this branch.

## v0.1.5 — RBAC Roles Expansion & CSRF Hardening — COMPLETE (100%)

Base: `v0.1.4`.

Scope:

- RBAC roles expansion with manager, operator, and viewer system roles,
  including permission backfills for existing entities and fields.
- CSRF middleware and request-ID propagation in the Axum core error contract.
- Verification: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace -- --test-threads=1`,
  `pnpm test`, `pnpm check`, and `pnpm build` in `packages/app` all pass.

## v0.1.1 — Tauri Desktop (offline) — COMPLETE

Completed in v0.1.1. Offline desktop scope implemented with Tauri shell,
in-process core boot, static SPA, and SQLite under OS app-data. All gates
passed. Deferred to v0.1.2+: auto-updater + signing, tray, multi-window,
file associations, `.icns`/`.ico` packaging.

## v0.1.6 — Tauri Desktop (offline) — COMPLETE

Base: `v0.1.5`.

Scope:

- Test regression fix: corrected expected field permission count from 4 to 10
  in `packages/core/tests/fields.rs` to match migration `0044_rbac_roles_expansion.sql`
  (5 system roles × 2 fields = 10 rows).
- Verified: `cargo test --workspace -- --test-threads=1` all green.
- Verified: `pnpm desktop:test`, `pnpm desktop:check`, `pnpm desktop:build` all pass.
- No new Core domain logic; only test expectation correction and roadmap update.

Release readiness: `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace -- --test-threads=1`,
`pnpm test`, `pnpm check`, and `pnpm build` in `packages/app` all pass.

## v0.1.7 — Desktop App-Data Lifecycle & Startup Hardening — COMPLETE

Base: `v0.1.6`.

Scope (no Core domain changes, no SQL outside core, no new `/v1` routes):

- `packages/desktop/src-tauri/src/lib.rs`: new `ensure_app_data_dir()` function
  to validate and prepare the app-data directory before boot, with writable-check
  and comprehensive error reporting on initialization failure.
- `packages/desktop/src-tauri/src/main.rs`: call `ensure_app_data_dir()` early
  in the boot sequence with explicit error logging so startup failures are clear.
- `packages/desktop/src-tauri/tests/app_data.rs`: focused integration test covering
  directory creation, idempotency, writability validation, and error rejection
  on file-path collision (3 tests).
- Verified: `cargo test --workspace -- --test-threads=1` all green (incl. new
  app_data tests + existing sidecar boot test).
- Verified: `pnpm desktop:test`, `pnpm desktop:check`, `pnpm desktop:build`
  all pass.
- No new features; hardening only. Desktop scope boundary unchanged.

Release readiness: `cargo fmt --all -- --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace -- --test-threads=1`,
and `pnpm desktop:build` all pass.

## v0.1.8 — Core Audit & Optional AI Boundary — COMPLETE

Base: `v0.1.7`.

Scope:

- Keep the Logholizon Core vendor-neutral and independent of AI providers.
- TypeSafe/Jev is not a Core dependency and is not required for the ERP runtime.
- AI integrations, if added later, must live behind an optional extension/provider boundary.
- Remove the TypeSafe/Jev skill and lock entry when it is not part of the product scope.
- Verify the Phase 1–20 acceptance contract against the current source, migrations,
  Core tests, application tests, and production build gates before declaring the
  baseline complete.

Acceptance evidence for branch `v0.1.8` is complete:

- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace --all-targets -- --test-threads=1` passed.
- `pnpm desktop:build` passed.
- `git diff --check` passed.
- Hardcoded ERP-domain implementation audit found no built-in ERP module implementation in Core/App/CLI.
- TypeSafe/Jev is removed from Core scope and remains an optional future integration boundary.

## v0.1.9 — Metadata Runtime Hardening & End-to-End Acceptance — COMPLETE

Base: `v0.1.8`.

Scope:

- Prove that user-defined module metadata drives entities, fields, relations, validation, and CRUD without ERP-specific runtime code.
- Add a focused metadata runtime acceptance test covering required, unique, range, pattern, and reference rules.
- Keep the existing Phase 20 HTTP acceptance as the public API proof; v0.1.9 adds deeper repository/runtime coverage.
- No built-in Accounting, Inventory, HR, CRM, Sales, or other ERP module implementation is introduced.

Acceptance evidence for branch `v0.1.9` is complete:

- Added `packages/core/tests/metadata_runtime.rs` covering module publication, entity/field metadata, relation creation, required/unique/range/pattern/reference validation, CRUD update, and filtered query.
- Targeted metadata runtime test passed.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace --all-targets -- --test-threads=1` passed, including the new metadata runtime test.
- `git diff --check` passed.
- No built-in ERP module implementation was added; the acceptance scenario remains metadata-driven.

## v0.2.0 — Dynamic UI Runtime — COMPLETE

Base: `v0.1.9`.

Scope:

- Verify that the generic Nuxt UI renders entity lists and record forms directly from entity/field metadata.
- Cover metadata-driven labels, columns, field visibility, required fields, status options, and record creation/editing through the generic entity route.
- Keep the UI domain-neutral: no Accounting, Inventory, HR, CRM, Sales, or other ERP-specific page implementation.
- Prefer reusable dynamic components/utilities over per-entity UI definitions.

Acceptance evidence for branch `v0.2.0` is complete:

- Added `packages/app/tests/e2e/dynamic-ui.spec.ts` as the user-facing acceptance contract.
- Chromium E2E passed against the production Nuxt server with seeded demo metadata: `1 passed` in 10.9s.
- The E2E verifies metadata-driven field/table rendering, required/status form behavior, record creation, persistence through the document API, and record editing through the generic row action.
- Hardened `packages/app/tests/e2e/run.cjs` to support production-server E2E mode, use built Core/App binaries when available, forward `NUXT_CORE_URL`, and use a consistent IPv4 loopback origin.
- Fixed CSRF origin detection to derive the protocol from the actual request socket or trusted `x-forwarded-proto`, so local production E2E over HTTP does not incorrectly require HTTPS.
- `pnpm --dir packages/app check` passed.
- `pnpm --dir packages/app build` passed.
- `git diff --check` passed.

The Dynamic UI Runtime acceptance contract is complete. The next work should extend generic UI capabilities, not introduce ERP-specific screens.

## v0.2.1 — Dynamic Form Layout Runtime — COMPLETE

Base: `v0.2.0`.

Scope:

- Prove that form section grouping and field order are controlled by entity metadata rather than entity-specific UI code.
- Keep layout configuration generic and reusable across arbitrary user-defined entities.
- Ensure the generic record form consumes the persisted `_entity_form_layout` configuration.

Acceptance evidence for branch `v0.2.1` is complete:

- Demo metadata now seeds a representative `work_order` form layout with a `Primary details` section and explicit field order.
- Extended `packages/app/tests/e2e/dynamic-ui.spec.ts` to verify the metadata-defined section and field ordering in the generic record form.
- Chromium E2E passed against the production Nuxt server: `1 passed` in 11.5s.
- `cargo fmt --all -- --check` passed.
- `cargo clippy -p logholizon-core --all-targets -- -D warnings` passed.
- `cargo test -p logholizon-core --lib -- --test-threads=1` passed: 11/11.
- `pnpm --dir packages/app check` passed.
- `git diff --check` passed.

The Dynamic Form Layout Runtime acceptance contract is complete. The next work should add generic view/table capabilities while preserving the metadata-driven boundary.

## v0.2.2 — Dynamic View/Table Runtime — COMPLETE

Base: `v0.2.1`.

Scope:

- Persist table projection settings as generic entity-view metadata.
- Restore saved column visibility and existing sort settings through the generic entity runtime.
- Keep view configuration reusable for arbitrary entities; no ERP-specific table definitions.

Acceptance evidence for branch `v0.2.2` is complete:

- Extended the generic entity view config with a `columns` projection alongside search/status/sort settings.
- Saved views now restore their column projection after navigation, constrained to fields the current user can view.
- Fixed initial saved-view loading so a view supplied directly in the route query is fetched on first render.
- Extended `packages/app/tests/e2e/dynamic-ui.spec.ts` to create a metadata view containing only `title`, navigate to that view, and verify `title` remains visible while `priority` is hidden.
- Chromium E2E passed against the production Nuxt server: `1 passed` in 12.2s.
- Production Nuxt build completed successfully after the runtime change.
- `git diff --check` passed.

The Dynamic View/Table Runtime acceptance contract is complete. The next work should extend relation-aware generic UI behavior without introducing ERP-specific screens.

## v0.2.3 — Relation-aware UI Runtime — COMPLETE

Base: `v0.2.2`.

Scope:

- Expose entity relation metadata through the API and connect the generic entity page to relation-aware loading and editing.
- Navigate to related-entity rows from the entity list; load and edit related documents inline.
- Ensure the relation endpoints are generic and reusable for any user-defined entity configuration.

Acceptance evidence for branch `v0.2.3` is complete:

- Added Rust core `GET /v1/entities/:id/relations` and `GET /v1/entities/:id/relations/:relation_id/related` endpoints to return relation metadata and related document records.
- Added Nuxt Nitro gateway forwarding at `packages/app/server/api/entities/[id]/relations.get.ts` and `packages/app/server/api/entities/[id]/relations/[relation_id].get.ts`.
- Extended `packages/app/server/core/client.ts` with `listRelations(id)` and `relatedDocuments(id, relationId)` methods.
- Updated `packages/app/app/pages/app/[entity].vue` to load relation metadata, display relation badges, navigate to related rows, and edit related records inline.
- `pnpm --dir packages/app build` passed — production build clean, zero errors.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `git diff --check` passed.

The Relation-aware UI Runtime acceptance contract is complete. The next work should add module versioning and upgrade metadata without introducing ERP-specific screens.

## Working Rule

Do not increase the percentage by adding more built-in ERP features. Increase the percentage by making the runtime capable of representing those features as user-defined metadata.
