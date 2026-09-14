-- Saved reports (group-by count on select/status fields).
-- Config shape: `{ "group_by": "<field_name>", "chart_type": "bar|pie" }`.
-- Admin-managed; users read reports for entities they can view.
CREATE TABLE IF NOT EXISTS _report (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  config TEXT NOT NULL DEFAULT '{}',
  created_by TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(entity_id, name)
);

CREATE INDEX IF NOT EXISTS idx_report_entity ON _report(entity_id);
