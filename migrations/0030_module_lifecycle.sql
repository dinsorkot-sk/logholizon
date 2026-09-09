-- Complete module lifecycle for Phase 1.
PRAGMA foreign_keys = OFF;
CREATE TABLE _module_new (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  icon TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL DEFAULT '',
  owner TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'review', 'published', 'enabled', 'disabled', 'archived')),
  version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
  definition TEXT NOT NULL DEFAULT '{}',
  created_by TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (owner, name)
);
INSERT INTO _module_new (id,name,label,description,icon,color,owner,status,version,definition,created_by,created_at,updated_at)
SELECT id,name,label,description,icon,color,owner,status,version,definition,created_by,created_at,updated_at FROM _module;
DROP TABLE _module;
ALTER TABLE _module_new RENAME TO _module;
CREATE INDEX idx_module_owner ON _module(owner, status);
CREATE INDEX idx_module_status ON _module(status);
PRAGMA foreign_keys = ON;
