-- Manufacturing MVP: BOMs plus manufacturing orders with consume/produce.
-- Quantities REAL stock-aligned with qty_base plus UOM; product ids opaque
-- _doc ids (no FK); company-scoped; costing via stock moving average.
CREATE TABLE IF NOT EXISTS _bom (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  product_id TEXT NOT NULL,
  name TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'archived')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_bom_company ON _bom(company_id, product_id, status);

CREATE TABLE IF NOT EXISTS _bom_line (
  id TEXT PRIMARY KEY NOT NULL,
  bom_id TEXT NOT NULL REFERENCES _bom(id) ON DELETE CASCADE,
  component_id TEXT NOT NULL,
  qty REAL NOT NULL CHECK (qty > 0),
  uom_id TEXT REFERENCES _uom(id) ON DELETE SET NULL,
  qty_base REAL NOT NULL CHECK (qty_base > 0)
);

CREATE INDEX IF NOT EXISTS idx_bom_line_bom ON _bom_line(bom_id);

CREATE TABLE IF NOT EXISTS _mfg_order (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  bom_id TEXT NOT NULL REFERENCES _bom(id) ON DELETE RESTRICT,
  product_id TEXT NOT NULL,
  qty REAL NOT NULL CHECK (qty > 0),
  qty_base REAL NOT NULL CHECK (qty_base > 0),
  warehouse_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'confirmed', 'done')),
  entry_date TEXT NOT NULL,
  entry_id TEXT REFERENCES _journal_entry(id) ON DELETE SET NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_mfg_order_company ON _mfg_order(company_id, status, entry_date DESC);
