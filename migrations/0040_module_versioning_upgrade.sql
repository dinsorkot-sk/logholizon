-- Semantic module versions, immutable release metadata, and upgrade history.
ALTER TABLE _module ADD COLUMN semantic_version TEXT NOT NULL DEFAULT '1.0.0';
ALTER TABLE _module_version ADD COLUMN semantic_version TEXT NOT NULL DEFAULT '1.0.0';
ALTER TABLE _module_version ADD COLUMN published_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;

UPDATE _module SET semantic_version = printf('%d.0.0', version)
WHERE semantic_version = '1.0.0' AND version > 1;
UPDATE _module_version SET semantic_version = printf('%d.0.0', version)
WHERE semantic_version = '1.0.0' AND version > 1;

CREATE TABLE IF NOT EXISTS _module_change (
  id TEXT PRIMARY KEY NOT NULL,
  module_id TEXT NOT NULL REFERENCES _module(id) ON DELETE CASCADE,
  from_version INTEGER,
  to_version INTEGER NOT NULL,
  from_semantic_version TEXT,
  to_semantic_version TEXT NOT NULL,
  change_type TEXT NOT NULL CHECK (change_type IN ('publish','rollback')),
  diff TEXT NOT NULL DEFAULT '[]',
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_module_change_module ON _module_change(module_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_module_version_semantic ON _module_version(module_id, semantic_version);
