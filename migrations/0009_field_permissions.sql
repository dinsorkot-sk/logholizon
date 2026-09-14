-- Field-level permissions (per-field view/edit by role).
-- Admin role bypasses the matrix in code; rows exist for both roles for uniformity.
CREATE TABLE IF NOT EXISTS _field_permission (
  field_id TEXT NOT NULL REFERENCES _meta_field(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK (role IN ('admin', 'user')),
  can_view INTEGER NOT NULL DEFAULT 1,
  can_edit INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (field_id, role)
);

-- Default: both roles can view and edit every field.
INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit)
SELECT id, 'admin', 1, 1 FROM _meta_field;
INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit)
SELECT id, 'user', 1, 1 FROM _meta_field;

CREATE INDEX IF NOT EXISTS idx_field_permission_field ON _field_permission(field_id);

-- Backfill default permissions for fields created after this migration.
CREATE TRIGGER IF NOT EXISTS trg_field_permission_backfill
AFTER INSERT ON _meta_field
BEGIN
  INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit)
  VALUES (NEW.id, 'admin', 1, 1), (NEW.id, 'user', 1, 1);
END;
