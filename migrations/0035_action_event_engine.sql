-- Generic Action & Event Engine.
-- Actions are metadata, never hardcoded business modules.
CREATE TABLE IF NOT EXISTS _action (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('create','update','delete','change_status','notify','webhook','formula','generate')),
  config TEXT NOT NULL DEFAULT '{}',
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0,1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(entity_id, name)
);

CREATE INDEX IF NOT EXISTS idx_action_entity_active ON _action(entity_id, active, name);

CREATE TABLE IF NOT EXISTS _event (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  document_id TEXT,
  event_type TEXT NOT NULL,
  action_id TEXT REFERENCES _action(id) ON DELETE SET NULL,
  payload TEXT NOT NULL DEFAULT '{}',
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_event_entity_created ON _event(entity_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_event_document_created ON _event(document_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_event_type_created ON _event(entity_id, event_type, created_at DESC);
