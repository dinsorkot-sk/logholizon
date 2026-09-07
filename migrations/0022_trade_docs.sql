-- Trade docs: single table with doc_type lead/quotation/order plus chain links.
-- Dedicated tables like invoice for transactional integrity; money in integer
-- minor units; qty REAL stock-aligned with qty_base plus UOM; product_id is an
-- opaque _doc id (no FK); company-scoped; FX handled at invoice conversion.
CREATE TABLE IF NOT EXISTS _trade_doc (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('sale', 'purchase')),
  doc_type TEXT NOT NULL CHECK (doc_type IN ('lead', 'quotation', 'order')),
  status TEXT NOT NULL CHECK (
    (doc_type = 'lead' AND status IN ('new', 'qualified', 'lost'))
    OR (doc_type IN ('quotation', 'order') AND status IN ('draft', 'sent', 'confirmed', 'done'))
  ),
  partner TEXT NOT NULL,
  currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  entry_date TEXT NOT NULL,
  source_id TEXT REFERENCES _trade_doc(id) ON DELETE SET NULL,
  invoice_id TEXT REFERENCES _invoice(id) ON DELETE SET NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_trade_doc_company ON _trade_doc(company_id, doc_type, status, entry_date DESC);
CREATE INDEX IF NOT EXISTS idx_trade_doc_source ON _trade_doc(source_id);
CREATE INDEX IF NOT EXISTS idx_trade_doc_invoice ON _trade_doc(invoice_id);

CREATE TABLE IF NOT EXISTS _trade_line (
  id TEXT PRIMARY KEY NOT NULL,
  trade_doc_id TEXT NOT NULL REFERENCES _trade_doc(id) ON DELETE CASCADE,
  product_id TEXT,
  description TEXT NOT NULL,
  qty REAL NOT NULL CHECK (qty > 0),
  uom_id TEXT REFERENCES _uom(id) ON DELETE SET NULL,
  qty_base REAL NOT NULL CHECK (qty_base > 0),
  unit_price INTEGER NOT NULL CHECK (unit_price >= 0),
  tax_rule_id TEXT REFERENCES _tax_rule(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_trade_line_doc ON _trade_line(trade_doc_id);
