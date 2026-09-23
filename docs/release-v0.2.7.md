# v0.2.7 — RBAC & Permissions COMPLETE (100%)

Phase 7: Full Role-Based Access Control and Permission System, completing the 100% Master Plan.

## Highlights

- **6-Column Entity Permissions**: `can_view`, `can_edit`, `can_export`, `can_import`, `can_execute`, `can_approve` — enforced at every HTTP and repository boundary
- **Record-Level Scope**: `all` or `own` scope per role/entity, with configurable `owner_field` for custom ownership mapping
- **Field-Level Permissions**: per-role `can_view`/`can_edit` with automatic payload redaction on reads and write protection on updates
- **Admin UI**: Full permission configuration tab with toggles for all permission types, field permission matrix, and record scope selector

## What Changed

### Rust Core (`packages/core`)

**Entity Permissions** — `EntityPermission` struct now carries 6 boolean capability flags (view/edit/export/import/execute/approve). `update_entity_permissions` accepts the full struct. `get_entity_permission_for_role` reads all columns from `_entity_permission`.

**Record Permissions** — New `RecordPermission` struct (`role`, `scope`, `owner_field`) with `get_record_permissions`/`update_record_permissions` CRUD. Scope validated to `all` or `own`, role validated against `_role` table.

**Scope Enforcement** — `check_record_access` enforces own-scope on single-document reads/updates/deletes. `record_scope_filter` returns a SQL WHERE fragment for list queries. `list_documents_as_role_actor` integrates scope filtering.

**Capability Checks** — `check_entity_capability` enforces export/import/execute/approve at the HTTP handler level. Applied to `preview_import_for_user`, `export_documents_for_user`, `execute_module_action`, and `transition_document`.

### HTTP Layer (`packages/core/src/http.rs`)

- `list_documents` handler now passes actor for record-scope filtering
- `get_document` handler applies `check_record_access` before returning
- `preview_import_for_user` enforces `can_import` capability
- `UpdatePermissionsRequest` uses `EntityPermission` directly

### Tests

- `rbac_permissions.rs`: Updated struct shapes (removed `entity_id` from `RecordPermission`)
- `fields.rs`, `health.rs`, `phase20_acceptance.rs`, `workbook.rs`: Use `EntityPermission::simple()` constructor
- All 11 lib tests pass

## Verification

| Gate | Result |
|------|--------|
| `cargo fmt --all` | ✅ clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo test -p logholizon-core --lib` | ✅ 11/11 passed |
| `vue-tsc --noEmit` | ✅ clean |
| `pnpm run build` (Nuxt 4.5.2) | ✅ built in 34s |

## v0.2.x Series Summary

| Version | Phase | Scope |
|---------|-------|-------|
| v0.2.0 | Phase 5 | Dynamic UI Runtime — metadata-driven entity routes, forms, tables |
| v0.2.1 | Phase 5+ | Dynamic Form Layout Runtime — section grouping, field ordering |
| v0.2.2 | Phase 5+ | Dynamic View/Table Runtime — saved views, column projection |
| v0.2.3 | Phase 5+ | Relation-aware UI Runtime — inline relation editing |
| v0.2.4 | Phase 6 | Module Builder completion verification |
| v0.2.5 | Phase 6+ | Module Builder hardening |
| v0.2.6 | Phase 7 (63%) | RBAC initial — role management, entity/field permissions, admin UI |
| **v0.2.7** | **Phase 7 (100%)** | **RBAC complete — record scope, capability enforcement, all gates** |

## Master Plan Status

All 20 phases are **COMPLETE (100%)**. The platform can represent any business domain as user-defined metadata without Rust Core changes.
