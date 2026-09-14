-- Entity permissions and saved views.
CREATE TABLE IF NOT EXISTS _entity_permission (
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK (role IN ('admin', 'user')),
  can_view INTEGER NOT NULL DEFAULT 1,
  can_edit INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (entity_id, role)
);

-- Default: both roles can view and edit every entity.
INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit)
SELECT id, 'admin', 1, 1 FROM _meta_entity;
INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit)
SELECT id, 'user', 1, 1 FROM _meta_entity;

CREATE TABLE IF NOT EXISTS _entity_view (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  config TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(entity_id, name)
);

CREATE INDEX IF NOT EXISTS idx_entity_view_entity ON _entity_view(entity_id);