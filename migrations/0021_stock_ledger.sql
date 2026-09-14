-- Stock ledger: full UOM dimensions plus on-hand balances plus moving-average valuation.
-- Quantities stored in base units (REAL); money in integer minor units per base unit.
-- product_id / warehouse_id reference generic _doc ids (no FK); company-scoped.
CREATE TABLE IF NOT EXISTS _uom (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  code TEXT NOT NULL,
  name TEXT NOT NULL,
  dimension TEXT NOT NULL CHECK (dimension IN ('qty', 'weight', 'length', 'volume')),
  factor_to_base REAL NOT NULL,
  is_base INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (company_id, code)
);

CREATE INDEX IF NOT EXISTS idx_uom_company ON _uom(company_id, dimension);

CREATE TABLE IF NOT EXISTS _stock_balance (
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  product_id TEXT NOT NULL,
  warehouse_id TEXT NOT NULL,
  qty_base REAL NOT NULL DEFAULT 0,
  avg_cost INTEGER NOT NULL DEFAULT 0,
  total_value INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (company_id, product_id, warehouse_id)
);

CREATE INDEX IF NOT EXISTS idx_stock_balance_product ON _stock_balance(company_id, product_id);
CREATE INDEX IF NOT EXISTS idx_stock_balance_warehouse ON _stock_balance(company_id, warehouse_id);

CREATE TABLE IF NOT EXISTS _stock_ledger_entry (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  product_id TEXT NOT NULL,
  warehouse_id TEXT NOT NULL,
  move_doc_id TEXT,
  move_type TEXT NOT NULL CHECK (move_type IN ('in', 'out')),
  qty REAL NOT NULL,
  uom_id TEXT REFERENCES _uom(id) ON DELETE SET NULL,
  qty_base REAL NOT NULL,
  unit_cost INTEGER NOT NULL DEFAULT 0,
  total_value INTEGER NOT NULL DEFAULT 0,
  balance_qty REAL NOT NULL DEFAULT 0,
  balance_avg INTEGER NOT NULL DEFAULT 0,
  entry_date TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_stock_ledger_product ON _stock_ledger_entry(company_id, product_id, warehouse_id, entry_date DESC);
