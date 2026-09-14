-- Multi-user collaboration polish: audit actor attribution.
ALTER TABLE _audit_log ADD COLUMN actor TEXT;
CREATE INDEX IF NOT EXISTS idx_audit_actor ON _audit_log(actor);
