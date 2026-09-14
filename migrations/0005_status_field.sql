ALTER TABLE _meta_field ADD COLUMN is_status INTEGER NOT NULL DEFAULT 0;

-- Backfill: mark the seeded work_order status field as the status field.
-- Covers databases that were migrated before this column existed.
UPDATE _meta_field SET is_status = 1 WHERE entity_id = 'work_order' AND name = 'status';