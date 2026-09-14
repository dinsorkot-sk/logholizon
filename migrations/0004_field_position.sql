-- Field ordering: add position column and backfill by name order.
ALTER TABLE _meta_field ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

-- Backfill: assign sequential positions per entity ordered by name.
-- _meta_field.name is UNIQUE per entity, so a simple count of smaller names works.
UPDATE _meta_field
SET position = (
  SELECT COUNT(*) FROM _meta_field AS f2
  WHERE f2.entity_id = _meta_field.entity_id AND f2.name < _meta_field.name
);