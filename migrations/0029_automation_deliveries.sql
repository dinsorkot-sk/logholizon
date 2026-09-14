-- Generic automation deliveries: allow _notification_delivery.rule_id to
-- reference _automation rows as well as _notification_rule rows.
-- Forward-only. Existing deliveries are preserved; the FK is dropped and
-- the column becomes an opaque reference (rule or automation id).
-- SQLite cannot drop a single FK, so rebuild the table preserving rows.
PRAGMA foreign_keys = OFF;

CREATE TABLE IF NOT EXISTS _notification_delivery_new (
  id TEXT PRIMARY KEY NOT NULL,
  rule_id TEXT NOT NULL,
  document_id TEXT NOT NULL,
  action TEXT NOT NULL,
  payload TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO _notification_delivery_new
  (id, rule_id, document_id, action, payload, status, attempts, last_error, created_at)
SELECT id, rule_id, document_id, action, payload, status, attempts, last_error, created_at
FROM _notification_delivery;

DROP TABLE _notification_delivery;

ALTER TABLE _notification_delivery_new RENAME TO _notification_delivery;

CREATE INDEX IF NOT EXISTS idx_notification_delivery_status ON _notification_delivery(status, created_at);

PRAGMA foreign_keys = ON;
