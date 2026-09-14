-- Workflow Engine extensions: transition guards, role requirements, and durable state history/events.
-- Forward-only; generic and metadata-driven.
ALTER TABLE _workflow_transition ADD COLUMN condition TEXT NOT NULL DEFAULT '';
ALTER TABLE _workflow_transition ADD COLUMN required_role TEXT NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS _workflow_history (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  transition_id TEXT NOT NULL REFERENCES _workflow_transition(id) ON DELETE CASCADE,
  action TEXT NOT NULL,
  from_state TEXT NOT NULL,
  to_state TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_workflow_history_document ON _workflow_history(document_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_workflow_history_entity ON _workflow_history(entity_id, created_at DESC);

CREATE TABLE IF NOT EXISTS _workflow_event (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  transition_id TEXT NOT NULL REFERENCES _workflow_transition(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL DEFAULT 'transition',
  action TEXT NOT NULL,
  payload TEXT NOT NULL DEFAULT '{}',
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_workflow_event_document ON _workflow_event(document_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_workflow_event_entity ON _workflow_event(entity_id, created_at DESC);
