-- Webhook notifications (transition-only, webhook-first).
-- Rules are admin-managed per entity; deliveries are enqueued atomically
-- with the transition audit row and sent by a background worker with retry.
CREATE TABLE IF NOT EXISTS _notification_rule (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  trigger TEXT NOT NULL DEFAULT 'transition' CHECK (trigger IN ('transition')),
  target_url TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notification_rule_entity ON _notification_rule(entity_id);

CREATE TABLE IF NOT EXISTS _notification_delivery (
  id TEXT PRIMARY KEY NOT NULL,
  rule_id TEXT NOT NULL REFERENCES _notification_rule(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL,
  action TEXT NOT NULL,
  payload TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notification_delivery_status ON _notification_delivery(status, created_at);
