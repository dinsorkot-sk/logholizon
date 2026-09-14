-- Scope entity short-name uniqueness per owning module (Phase F).
-- Previously `_meta_entity.name` was globally UNIQUE, so two independent
-- modules (or a module and seed demo data) could not reuse common names
-- like `product` or `customer`. Names are now unique per `module_id`;
-- global entities (module_id IS NULL) keep one row per name because NULL
-- values never conflict in a SQLite UNIQUE index.
-- Runs inside the migrator transaction: defer FK checks to COMMIT so the
-- rebuild (drop + re-create + re-insert) validates against the final state.
PRAGMA defer_foreign_keys=ON;

CREATE TABLE _meta_entity_new (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  module TEXT,
  module_id TEXT REFERENCES _module(id) ON DELETE SET NULL,
  description TEXT NOT NULL DEFAULT '',
  settings TEXT NOT NULL DEFAULT '{}',
  tenant_id TEXT REFERENCES _tenant(id) ON DELETE CASCADE,
  UNIQUE (module_id, name)
);

INSERT INTO _meta_entity_new
  (id, name, label, created_at, module, module_id, description, settings, tenant_id)
SELECT id, name, label, created_at, module, module_id, description, settings, tenant_id
FROM _meta_entity;

DROP TABLE _meta_entity;

ALTER TABLE _meta_entity_new RENAME TO _meta_entity;

CREATE INDEX IF NOT EXISTS idx_meta_entity_module_id ON _meta_entity(module_id);
CREATE INDEX IF NOT EXISTS idx_meta_entity_tenant ON _meta_entity(tenant_id, id);
