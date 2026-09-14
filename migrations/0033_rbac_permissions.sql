-- Phase 7: generic roles and permission capabilities.
-- Keep legacy _user.role for compatibility; _user_role is the effective assignment.
CREATE TABLE IF NOT EXISTS _role (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  label TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  system INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO _role (id, name, label, description, system)
VALUES ('role_admin', 'admin', 'Administrator', 'Full platform administration', 1),
       ('role_user', 'user', 'User', 'Standard application user', 1);

CREATE TABLE IF NOT EXISTS _user_role (
  user_id TEXT PRIMARY KEY NOT NULL REFERENCES _user(id) ON DELETE CASCADE,
  role_id TEXT NOT NULL REFERENCES _role(id) ON DELETE RESTRICT
);

INSERT OR IGNORE INTO _user_role (user_id, role_id)
SELECT id, CASE role WHEN 'admin' THEN 'role_admin' ELSE 'role_user' END FROM _user;

-- Remove the old trigger before replacing the fixed-role permission table.
DROP TRIGGER IF EXISTS trg_field_permission_backfill;

-- Replace fixed admin/user role constraints with generic role names.
CREATE TABLE _entity_permission_v2 (
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  can_view INTEGER NOT NULL DEFAULT 1,
  can_edit INTEGER NOT NULL DEFAULT 1,
  can_export INTEGER NOT NULL DEFAULT 1,
  can_import INTEGER NOT NULL DEFAULT 1,
  can_execute INTEGER NOT NULL DEFAULT 1,
  can_approve INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (entity_id, role)
);
INSERT INTO _entity_permission_v2 (entity_id, role, can_view, can_edit)
SELECT entity_id, role, can_view, can_edit FROM _entity_permission;
DROP TABLE _entity_permission;
ALTER TABLE _entity_permission_v2 RENAME TO _entity_permission;

CREATE TABLE _field_permission_v2 (
  field_id TEXT NOT NULL REFERENCES _meta_field(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  can_view INTEGER NOT NULL DEFAULT 1,
  can_edit INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (field_id, role)
);
INSERT INTO _field_permission_v2 (field_id, role, can_view, can_edit)
SELECT field_id, role, can_view, can_edit FROM _field_permission;
DROP TABLE _field_permission;
ALTER TABLE _field_permission_v2 RENAME TO _field_permission;
CREATE INDEX IF NOT EXISTS idx_field_permission_field ON _field_permission(field_id);

-- Generic record scope: all records or records owned by the current user.
CREATE TABLE IF NOT EXISTS _record_permission (
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'all' CHECK (scope IN ('all', 'own')),
  owner_field TEXT,
  PRIMARY KEY (entity_id, role)
);
CREATE INDEX IF NOT EXISTS idx_record_permission_entity ON _record_permission(entity_id);

CREATE TRIGGER IF NOT EXISTS trg_field_permission_backfill_v2
AFTER INSERT ON _meta_field
BEGIN
  INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit)
  SELECT NEW.id, name, 1, 1 FROM _role;
END;

INSERT OR IGNORE INTO _record_permission (entity_id, role, scope)
SELECT e.id, r.name, 'all' FROM _meta_entity e CROSS JOIN _role r;
