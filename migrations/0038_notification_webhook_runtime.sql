-- Phase 12: generic notification and webhook runtime.
CREATE TABLE IF NOT EXISTS _notification_template (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  channel TEXT NOT NULL CHECK (channel IN ('in_app','email')),
  subject TEXT NOT NULL DEFAULT '',
  body TEXT NOT NULL,
  variables TEXT NOT NULL DEFAULT '[]',
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS _notification_target (
  id TEXT PRIMARY KEY NOT NULL,
  template_id TEXT NOT NULL REFERENCES _notification_template(id) ON DELETE CASCADE,
  user_id TEXT REFERENCES _user(id) ON DELETE CASCADE,
  role_name TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (user_id IS NOT NULL OR role_name IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS idx_notification_target_template ON _notification_target(template_id);
CREATE INDEX IF NOT EXISTS idx_notification_target_user ON _notification_target(user_id);
CREATE INDEX IF NOT EXISTS idx_notification_target_role ON _notification_target(role_name);
CREATE TABLE IF NOT EXISTS _notification (
  id TEXT PRIMARY KEY NOT NULL,
  template_id TEXT REFERENCES _notification_template(id) ON DELETE SET NULL,
  user_id TEXT NOT NULL REFERENCES _user(id) ON DELETE CASCADE,
  channel TEXT NOT NULL CHECK (channel IN ('in_app','email')),
  subject TEXT NOT NULL DEFAULT '',
  body TEXT NOT NULL,
  data TEXT NOT NULL DEFAULT '{}',
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','read','delivered','failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  read_at TEXT,
  delivered_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_notification_user_status ON _notification(user_id,status,created_at);
CREATE INDEX IF NOT EXISTS idx_notification_pending ON _notification(channel,status,created_at);
CREATE TABLE IF NOT EXISTS _webhook_endpoint (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL UNIQUE,
  url TEXT NOT NULL,
  secret TEXT NOT NULL DEFAULT '',
  headers TEXT NOT NULL DEFAULT '{}',
  active INTEGER NOT NULL DEFAULT 1,
  timeout_secs INTEGER NOT NULL DEFAULT 10,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS _webhook_delivery (
  id TEXT PRIMARY KEY NOT NULL,
  endpoint_id TEXT NOT NULL REFERENCES _webhook_endpoint(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  document_id TEXT,
  payload TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','delivered','failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  response_status INTEGER,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  delivered_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_pending ON _webhook_delivery(status,created_at);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_endpoint ON _webhook_delivery(endpoint_id,created_at);
