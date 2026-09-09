-- Phase 11: generic automation engine extensions.
ALTER TABLE _automation ADD COLUMN condition TEXT NOT NULL DEFAULT '';
ALTER TABLE _automation ADD COLUMN schedule TEXT NOT NULL DEFAULT '';
ALTER TABLE _automation ADD COLUMN actions TEXT NOT NULL DEFAULT '[]';
ALTER TABLE _automation ADD COLUMN max_attempts INTEGER NOT NULL DEFAULT 3;
ALTER TABLE _automation ADD COLUMN updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;

CREATE TABLE IF NOT EXISTS _automation_execution (
  id TEXT PRIMARY KEY NOT NULL,
  automation_id TEXT NOT NULL REFERENCES _automation(id) ON DELETE CASCADE,
  event_id TEXT,
  document_id TEXT,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','running','succeeded','failed')),
  attempt INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  result TEXT NOT NULL DEFAULT '{}',
  scheduled_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  started_at TEXT,
  finished_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_automation_execution_pending ON _automation_execution(status, scheduled_at);
CREATE INDEX IF NOT EXISTS idx_automation_execution_automation ON _automation_execution(automation_id, created_at);
