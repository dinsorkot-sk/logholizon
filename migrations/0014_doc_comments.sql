-- Record comments (chatter write): per-document threaded notes.
-- Separate from _audit_log (system-generated); comments are user-authored.
CREATE TABLE IF NOT EXISTS _doc_comment (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  body TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_doc_comment_doc_id ON _doc_comment(doc_id, created_at DESC);
