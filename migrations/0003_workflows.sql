CREATE TABLE IF NOT EXISTS _workflow_state (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  label TEXT NOT NULL,
  position INTEGER NOT NULL,
  UNIQUE(entity_id, name)
);

CREATE TABLE IF NOT EXISTS _workflow_transition (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  from_state TEXT NOT NULL,
  to_state TEXT NOT NULL,
  action TEXT NOT NULL,
  UNIQUE(entity_id, from_state, action)
);

CREATE INDEX IF NOT EXISTS idx_workflow_state_entity ON _workflow_state(entity_id, position);
CREATE INDEX IF NOT EXISTS idx_workflow_transition_entity ON _workflow_transition(entity_id, from_state);
