-- Phase 2: universal entity/field metadata persistence.
-- Forward-only. Existing metadata remains valid via constant defaults.
ALTER TABLE _meta_entity ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE _meta_entity ADD COLUMN settings TEXT NOT NULL DEFAULT '{}';
ALTER TABLE _meta_field ADD COLUMN label TEXT NOT NULL DEFAULT '';
ALTER TABLE _meta_field ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE _meta_field ADD COLUMN readonly INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN searchable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN sortable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN filterable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN indexed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE _meta_field ADD COLUMN precision INTEGER;
ALTER TABLE _meta_field ADD COLUMN help_text TEXT NOT NULL DEFAULT '';
