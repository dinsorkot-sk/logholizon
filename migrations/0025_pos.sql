-- POS MVP: sessions plus orders with cart lines plus tender/change.
-- Money in integer minor units; qty REAL stock-aligned; product ids opaque
-- _doc ids (no FK); company-scoped; close reconciles cash drawer.
CREATE TABLE IF NOT EXISTS _pos_session (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  warehouse_id TEXT NOT NULL,
  opening_cash INTEGER NOT NULL CHECK (opening_cash >= 0),
  closing_cash INTEGER CHECK (closing_cash IS NULL OR closing_cash >= 0),
  status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
  entry_date TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_pos_session_company ON _pos_session(company_id, status, entry_date DESC);

CREATE TABLE IF NOT EXISTS _pos_order (
  id TEXT PRIMARY KEY NOT NULL,
  session_id TEXT NOT NULL REFERENCES _pos_session(id) ON DELETE CASCADE,
  partner TEXT NOT NULL DEFAULT 'Walk-in',
  currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'paid', 'void')),
  tendered INTEGER NOT NULL DEFAULT 0 CHECK (tendered >= 0),
  change_due INTEGER NOT NULL DEFAULT 0 CHECK (change_due >= 0),
  entry_id TEXT REFERENCES _journal_entry(id) ON DELETE SET NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_pos_order_session ON _pos_order(session_id, status);

CREATE TABLE IF NOT EXISTS _pos_line (
  id TEXT PRIMARY KEY NOT NULL,
  order_id TEXT NOT NULL REFERENCES _pos_order(id) ON DELETE CASCADE,
  product_id TEXT NOT NULL,
  description TEXT NOT NULL,
  qty REAL NOT NULL CHECK (qty > 0),
  uom_id TEXT REFERENCES _uom(id) ON DELETE SET NULL,
  qty_base REAL NOT NULL CHECK (qty_base > 0),
  unit_price INTEGER NOT NULL CHECK (unit_price >= 0),
  tax_rule_id TEXT REFERENCES _tax_rule(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_pos_line_order ON _pos_line(order_id);
