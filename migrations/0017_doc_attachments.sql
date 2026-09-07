-- Record attachments: per-document files stored as DB blobs.
-- 5MB per-file cap enforced in repository; backup covers blobs via VACUUM INTO.
CREATE TABLE IF NOT EXISTS _doc_attachment (
  id TEXT PRIMARY KEY NOT NULL,
  entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  filename TEXT NOT NULL,
  content_type TEXT NOT NULL,
  size INTEGER NOT NULL,
  data BLOB NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_doc_attachment_doc_id ON _doc_attachment(doc_id, created_at DESC);
