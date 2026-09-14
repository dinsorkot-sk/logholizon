-- Field business rules: unique, ranges, patterns, defaults, auto-number.
-- Forward-only. Existing _meta_field columns are untouched.
ALTER TABLE _meta_field ADD COLUMN is_unique INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN min_value REAL;
ALTER TABLE _meta_field ADD COLUMN max_value REAL;
ALTER TABLE _meta_field ADD COLUMN pattern TEXT;
ALTER TABLE _meta_field ADD COLUMN min_length INTEGER;
ALTER TABLE _meta_field ADD COLUMN max_length INTEGER;
ALTER TABLE _meta_field ADD COLUMN default_value TEXT;
ALTER TABLE _meta_field ADD COLUMN auto_number_prefix TEXT;
ALTER TABLE _meta_field ADD COLUMN auto_number_width INTEGER;

-- Per-entity auto-number counters. One row per (entity, field).
CREATE TABLE IF NOT EXISTS _number_counter (
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  field_name TEXT NOT NULL,
  next_value INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (entity_id, field_name)
);
