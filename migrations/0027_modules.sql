-- User-defined module runtime: registry plus versioning plus tenant owner.
-- Forward-only. Generic runtime tables (_meta_*, _doc, _workflow_*, etc.)
-- are untouched. Legacy ERP tables were dropped in 0026.
CREATE TABLE IF NOT EXISTS _module (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  icon TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL DEFAULT '',
  owner TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'archived')),
  version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
  definition TEXT NOT NULL DEFAULT '{}',
  created_by TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (owner, name)
);

CREATE INDEX IF NOT EXISTS idx_module_owner ON _module(owner, status);
CREATE INDEX IF NOT EXISTS idx_module_status ON _module(status);

CREATE TABLE IF NOT EXISTS _module_version (
  id TEXT PRIMARY KEY NOT NULL,
  module_id TEXT NOT NULL REFERENCES _module(id) ON DELETE CASCADE,
  version INTEGER NOT NULL CHECK (version >= 1),
  definition TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (module_id, version)
);

CREATE INDEX IF NOT EXISTS idx_module_version_module ON _module_version(module_id, version DESC);

-- Link entities to their owning module (display denormalization;
-- source of truth is _module.definition).
ALTER TABLE _meta_entity ADD COLUMN module_id TEXT REFERENCES _module(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_meta_entity_module_id ON _meta_entity(module_id);

-- Generic automations: trigger on create/update/delete/transition,
-- then webhook/notify. Does not alter _notification_rule.
CREATE TABLE IF NOT EXISTS _automation (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  trigger TEXT NOT NULL CHECK (trigger IN ('create', 'update', 'delete', 'transition')),
  action TEXT NOT NULL DEFAULT 'webhook' CHECK (action IN ('webhook', 'notify')),
  target_url TEXT NOT NULL DEFAULT '',
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_automation_entity ON _automation(entity_id, trigger, active);
