-- Invoice/Payment AR/AP: dedicated tables with balanced auto-post GL.
-- Money in integer minor units; FX rate stored at post; period locks enforced.
CREATE TABLE IF NOT EXISTS _invoice (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('sale', 'purchase')),
  partner TEXT NOT NULL,
  currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  fx_rate REAL NOT NULL DEFAULT 1.0,
  base_total INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted', 'paid', 'void')),
  entry_id TEXT REFERENCES _journal_entry(id) ON DELETE SET NULL,
  entry_date TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_invoice_company ON _invoice(company_id, status, entry_date DESC);

CREATE TABLE IF NOT EXISTS _invoice_line (
  id TEXT PRIMARY KEY NOT NULL,
  invoice_id TEXT NOT NULL REFERENCES _invoice(id) ON DELETE CASCADE,
  description TEXT NOT NULL,
  quantity INTEGER NOT NULL,
  unit_price INTEGER NOT NULL,
  tax_rule_id TEXT REFERENCES _tax_rule(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_invoice_line_invoice ON _invoice_line(invoice_id);

CREATE TABLE IF NOT EXISTS _payment (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('receive', 'pay')),
  partner TEXT NOT NULL,
  currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  amount INTEGER NOT NULL,
  entry_id TEXT REFERENCES _journal_entry(id) ON DELETE SET NULL,
  entry_date TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_payment_company ON _payment(company_id, entry_date DESC);

CREATE TABLE IF NOT EXISTS _payment_allocation (
  id TEXT PRIMARY KEY NOT NULL,
  payment_id TEXT NOT NULL REFERENCES _payment(id) ON DELETE CASCADE,
  invoice_id TEXT NOT NULL REFERENCES _invoice(id) ON DELETE CASCADE,
  amount INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (payment_id, invoice_id)
);

CREATE INDEX IF NOT EXISTS idx_allocation_invoice ON _payment_allocation(invoice_id);
