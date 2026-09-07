-- Document activities: per-record todos with due date and assignee.
-- Simple chatter extension; no reminders yet.
CREATE TABLE IF NOT EXISTS _doc_activity (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  due_date TEXT,
  assignee TEXT,
  done INTEGER NOT NULL DEFAULT 0,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_doc_activity_doc_id ON _doc_activity(doc_id, done, due_date);
