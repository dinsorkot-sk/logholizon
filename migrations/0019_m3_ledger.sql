-- M3 core ledger: chart of accounts plus balanced journal entries plus period locks.
-- Money in integer minor units; company-scoped; void reverses without deleting history.
CREATE TABLE IF NOT EXISTS _gl_account (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  code TEXT NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL CHECK (type IN ('asset', 'liability', 'equity', 'income', 'expense')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (company_id, code)
);

CREATE INDEX IF NOT EXISTS idx_gl_account_company ON _gl_account(company_id, code);

CREATE TABLE IF NOT EXISTS _journal_entry (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  memo TEXT NOT NULL DEFAULT '',
  entry_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'posted' CHECK (status IN ('posted', 'void')),
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_journal_entry_company ON _journal_entry(company_id, entry_date DESC);

CREATE TABLE IF NOT EXISTS _journal_line (
  id TEXT PRIMARY KEY NOT NULL,
  entry_id TEXT NOT NULL REFERENCES _journal_entry(id) ON DELETE CASCADE,
  account_id TEXT NOT NULL REFERENCES _gl_account(id) ON DELETE RESTRICT,
  debit INTEGER NOT NULL DEFAULT 0,
  credit INTEGER NOT NULL DEFAULT 0,
  memo TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_journal_line_entry ON _journal_line(entry_id);
CREATE INDEX IF NOT EXISTS idx_journal_line_account ON _journal_line(account_id);

CREATE TABLE IF NOT EXISTS _period_lock (
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  period TEXT NOT NULL,
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (company_id, period)
);
