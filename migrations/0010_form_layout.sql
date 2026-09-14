-- Form layout designer (Visual Builder Phase 2).
-- Singleton config per entity: `{ "sections": [{ "id": "...", "label": "...", "fields": ["<field_id>", ...] }] }`.
-- Fields missing from the config render in a trailing "Other" section;
-- unknown field ids are ignored at render time (tolerant policy).
CREATE TABLE IF NOT EXISTS _entity_form_layout (
  entity_id TEXT PRIMARY KEY NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  config TEXT NOT NULL DEFAULT '{}'
);
