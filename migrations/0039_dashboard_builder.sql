CREATE TABLE IF NOT EXISTS _dashboard (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    layout TEXT NOT NULL DEFAULT '[]',
    filters TEXT NOT NULL DEFAULT '{}',
    roles TEXT NOT NULL DEFAULT '[]',
    users TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0,1)),
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_dashboard_active ON _dashboard(active, name);
CREATE INDEX IF NOT EXISTS idx_dashboard_created_by ON _dashboard(created_by);
