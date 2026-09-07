-- M2 foundation: multi-company plus currencies plus FX rates plus tax rules.
-- No document scoping yet; money uses integer minor units plus ISO code.
CREATE TABLE IF NOT EXISTS _company (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  base_currency TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS _currency (
  code TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  decimals INTEGER NOT NULL DEFAULT 2,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS _fx_rate (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  from_currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  to_currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  rate REAL NOT NULL,
  rate_date TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_fx_rate_pair ON _fx_rate(company_id, from_currency, to_currency, rate_date DESC);

CREATE TABLE IF NOT EXISTS _tax_rule (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  rate REAL NOT NULL,
  is_inclusive INTEGER NOT NULL DEFAULT 0,
  is_withholding INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_tax_rule_company ON _tax_rule(company_id);
