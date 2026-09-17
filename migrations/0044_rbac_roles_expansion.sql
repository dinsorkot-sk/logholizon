-- Phase: RBAC Roles Expansion (v0.1.5+)
-- Add manager, operator, viewer roles alongside existing admin/user roles
-- for more granular role-based access control.
--
-- This migration is forward-only and never edits previously applied migrations.
-- New roles are integrated into existing permission tables following the pattern
-- established in 0033_rbac_permissions.sql.

INSERT OR IGNORE INTO _role (id, name, label, description, system)
VALUES
  ('role_manager', 'manager', 'Manager', 'Can create, edit, delete documents and configure workflows', 1),
  ('role_operator', 'operator', 'Operator', 'Can create and edit documents, transition workflows', 1),
  ('role_viewer', 'viewer', 'Viewer', 'Read-only access to all documents and reports', 1);

-- Backfill new roles into entity permissions for all existing entities
-- (policy: manager/operator = view+edit, viewer = view only)
INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit, can_export, can_import, can_execute, can_approve)
SELECT e.id, 'manager', 1, 1, 1, 0, 1, 0 FROM _meta_entity e
UNION
SELECT e.id, 'operator', 1, 1, 0, 0, 1, 0 FROM _meta_entity e
UNION
SELECT e.id, 'viewer', 1, 0, 1, 0, 0, 0 FROM _meta_entity e;

-- Backfill new roles into field permissions for all existing fields
-- (policy: manager/operator = view+edit, viewer = view only)
INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit)
SELECT f.id, 'manager', 1, 1 FROM _meta_field f
UNION
SELECT f.id, 'operator', 1, 1 FROM _meta_field f
UNION
SELECT f.id, 'viewer', 1, 0 FROM _meta_field f;

-- Backfill new roles into record permissions
-- (policy: manager/operator = all records, viewer = all records view-only)
INSERT OR IGNORE INTO _record_permission (entity_id, role, scope)
SELECT e.id, 'manager', 'all' FROM _meta_entity e
UNION
SELECT e.id, 'operator', 'all' FROM _meta_entity e
UNION
SELECT e.id, 'viewer', 'all' FROM _meta_entity e;
