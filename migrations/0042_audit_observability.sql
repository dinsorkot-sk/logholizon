-- Phase 18: audit and observability.
CREATE TABLE IF NOT EXISTS _observability_log (
  id TEXT PRIMARY KEY NOT NULL,
  occurred_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  level TEXT NOT NULL CHECK (level IN ('info','warn','error')),
  category TEXT NOT NULL,
  action TEXT NOT NULL,
  actor TEXT,
  request_id TEXT,
  correlation_id TEXT,
  target_type TEXT,
  target_id TEXT,
  status_code INTEGER,
  duration_ms INTEGER,
  message TEXT NOT NULL DEFAULT '',
  metadata TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_observability_time ON _observability_log(occurred_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_observability_category ON _observability_log(category, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_observability_actor ON _observability_log(actor, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_observability_request ON _observability_log(request_id);
CREATE INDEX IF NOT EXISTS idx_observability_correlation ON _observability_log(correlation_id);
