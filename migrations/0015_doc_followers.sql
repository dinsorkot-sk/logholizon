-- Document followers: per-user watch list for chatter updates.
-- Keyed by actor username (no user FK); toggle is idempotent.
CREATE TABLE IF NOT EXISTS _doc_follower (
  doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  actor TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (doc_id, actor)
);

CREATE INDEX IF NOT EXISTS idx_doc_follower_doc_id ON _doc_follower(doc_id);
