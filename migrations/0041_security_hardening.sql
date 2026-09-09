-- Phase 17: tenant boundary and security hardening.
CREATE TABLE IF NOT EXISTS _tenant (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO _tenant (id, name)
SELECT DISTINCT owner, owner FROM _module WHERE TRIM(owner) <> '';

ALTER TABLE _module ADD COLUMN tenant_id TEXT REFERENCES _tenant(id) ON DELETE CASCADE;
UPDATE _module SET tenant_id = owner WHERE tenant_id IS NULL AND TRIM(owner) <> '';
CREATE INDEX IF NOT EXISTS idx_module_tenant ON _module(tenant_id, name);

ALTER TABLE _meta_entity ADD COLUMN tenant_id TEXT REFERENCES _tenant(id) ON DELETE CASCADE;
UPDATE _meta_entity SET tenant_id = (
  SELECT m.tenant_id FROM _module m WHERE m.id = _meta_entity.module_id
) WHERE tenant_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_meta_entity_tenant ON _meta_entity(tenant_id, id);

ALTER TABLE _doc ADD COLUMN tenant_id TEXT REFERENCES _tenant(id) ON DELETE CASCADE;
UPDATE _doc SET tenant_id = (
  SELECT e.tenant_id FROM _meta_entity e WHERE e.id = _doc.entity_id
) WHERE tenant_id IS NULL;
CREATE INDEX IF NOT EXISTS idx_doc_tenant ON _doc(tenant_id, entity_id);
