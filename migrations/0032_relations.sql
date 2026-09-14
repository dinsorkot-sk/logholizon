-- Universal relation metadata and many-to-many link storage.
CREATE TABLE IF NOT EXISTS _meta_relation (
  id TEXT PRIMARY KEY NOT NULL,
  source_entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE CASCADE,
  source_field_id TEXT REFERENCES _meta_field(id) ON DELETE CASCADE,
  target_entity_id TEXT NOT NULL REFERENCES _meta_entity(id) ON DELETE RESTRICT,
  target_field_id TEXT REFERENCES _meta_field(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  relation_type TEXT NOT NULL CHECK (relation_type IN ('one_to_one','one_to_many','many_to_one','many_to_many')),
  on_delete TEXT NOT NULL DEFAULT 'restrict' CHECK (on_delete IN ('restrict','set_null','cascade')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(source_entity_id, name)
);
CREATE INDEX IF NOT EXISTS idx_meta_relation_source ON _meta_relation(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_meta_relation_target ON _meta_relation(target_entity_id);
CREATE TABLE IF NOT EXISTS _meta_relation_link (
  relation_id TEXT NOT NULL REFERENCES _meta_relation(id) ON DELETE CASCADE,
  source_doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  target_doc_id TEXT NOT NULL REFERENCES _doc(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (relation_id, source_doc_id, target_doc_id),
  CHECK (source_doc_id != target_doc_id)
);
CREATE INDEX IF NOT EXISTS idx_relation_link_source ON _meta_relation_link(relation_id, source_doc_id);
CREATE INDEX IF NOT EXISTS idx_relation_link_target ON _meta_relation_link(relation_id, target_doc_id);
