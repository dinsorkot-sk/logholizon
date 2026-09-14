# ERP Module Packages — Authoring Conventions

Ready-made ERP solutions ship as **module packages** (metadata JSON), never as
hardcoded Rust. Each package installs through the same Module Runtime,
lifecycle, and permission contracts as user-created modules
(`Module -> Entities -> Fields -> Relations -> Forms -> Views -> Permissions ->
Workflow -> Actions -> Formula/Validation -> Automation -> Reports ->
Dashboard -> Publish -> Runtime`).

## Install

```bash
cargo run -p logholizon-cli -- install-module packages/erp/<name>.module.json
cargo run -p logholizon-cli -- list-modules
cargo run -p logholizon-cli -- list-modules --admin   # every owner
```

Install runs the normal lifecycle: draft → review → published → enabled, then
materializes the v2 sections (relations, actions, automations, dashboards).
Use `--owner <tenant>` (default `admin`) to scope the install.

## Package file layout (`<name>.module.json`)

```json
{
  "schema_version": 2,
  "kind": "logholizon.module",
  "manifest": { "name": "vehicle", "version": "1.0.0", "dependencies": [] },
  "module": {
    "name": "vehicle",
    "label": "Vehicle Management",
    "description": "...",
    "icon": "i-lucide-truck",
    "color": "blue",
    "definition": { "entities": [ ... ] },
    "relations": [ ... ],
    "actions": [ ... ],
    "automations": [ ... ],
    "dashboards": [ ... ]
  }
}
```

Schema 1 packages (definition only) still install; schema 2 adds the optional
self-contained sections. `manifest.name` must equal `module.name`.

## Conventions

- **Names are definition-level.** Entities, fields, and widget `entity_id`
  values use short names (`customer`, `status`); the installer resolves them
  to materialized IDs (`<owner>_<module>_<entity>`,
  `<entity>_<field>`) after publish. Never hardcode materialized IDs.
- **Entities** follow `validate_module_definition`: unique snake_case names,
  typed fields, at most one `is_status` select field per entity, reference
  fields point at sibling entities, computed/formula expressions validate
  against sibling field names.
- **Relations** (`source`, `target` = entity names; `source_field` /
  `target_field` = field names): `relation_type` is `one_to_one`,
  `one_to_many`, `many_to_one`, or `many_to_many`; `on_delete` is `restrict`
  (default), `cascade`, or `set_null`. Non-`many_to_many` relations require
  `source_field`.
- **Actions** (`entity`, `name`, `label`, `kind`, `config`): `kind` is
  `create`, `update`, `delete`, `change_status`, `notify`, `webhook`,
  `formula`, or `generate`. Names are snake_case.
- **Automations** (`entity`, `trigger`, `action`, `target_url`, `active`,
  `condition`, `schedule`, `max_attempts`): `trigger` is `create`, `update`,
  `delete`, or `transition`; `action` is `webhook` or `notify`. Target URLs
  must pass outbound SSRF validation (http/https, no localhost/private).
- **Dashboards** (`name`, `description`, `layout`, `filters`, `roles`,
  `users`): widget `kind` is `kpi`, `table`, `list`, `bar`, `line`, `pie`,
  or `area`; widget `entity_id` is the definition entity name; widget
  `config` is a report config (`fields`, `aggregates`, `group_by`, ...).
- **Reports** live inside `definition.entities[].reports` (`name` + `config`);
  **views** in `entities[].views`; **form layouts** in
  `entities[].form_layout`; **workflows** in `entities[].workflow`;
  **notification rules** in `entities[].notifications`.
- **No business logic in Core.** If a package needs a deterministic domain
  invariant that metadata cannot express, propose a native engine through the
  extension contract — do not hardcode the domain into Core or the package
  format.

## Authoring checklist

1. `cargo run -p logholizon-cli -- install-module packages/erp/<name>.module.json`
2. Exercise CRUD, workflow transitions, actions, reports, and dashboards
   through the runtime UI or API.
3. `cargo test -p logholizon-core --test erp_packages` covers install +
   operate for every shipped package; extend it when adding a package.
