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

### Phase 18 — Audit & Observability — 97%
- Audit log: who/what/when/target/before/after.
- Login and security events.
- Permission denial logs.
- Workflow history.
- Automation execution logs.
- Webhook execution logs.
- Runtime error logs.
- Request ID and correlation ID.
- Operational metrics sufficient to diagnose runtime failures.

### Phase 19 — Testing & Production Readiness — 99%
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

## ERP Hardcode Removal

After the universal runtime passes acceptance, migrate legacy hardcoded domains out of Core. Candidates include `GlAccount`, `JournalEntry`, `JournalLine`, `Invoice`, `InvoiceLine`, `Payment`, `StockLedger`, `Employee`, `PayrollRun`, BOM/manufacturing, POS, Customer and Sales domain routes/types.

The target is:

`hardcoded ERP domain -> optional engine or user-defined module`

Never replace one hardcoded ERP folder with another.

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

## Working Rule

Do not increase the percentage by adding more built-in ERP features. Increase the percentage by making the runtime capable of representing those features as user-defined metadata.
