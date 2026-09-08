-- HR MVP: employees plus leave requests plus payroll runs with payslips.
-- Money in integer minor units; company-scoped; payroll posts salary journals.
CREATE TABLE IF NOT EXISTS _employee (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  code TEXT NOT NULL,
  name TEXT NOT NULL,
  base_salary INTEGER NOT NULL CHECK (base_salary >= 0),
  currency TEXT NOT NULL REFERENCES _currency(code) ON DELETE RESTRICT,
  hire_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (company_id, code)
);

CREATE INDEX IF NOT EXISTS idx_employee_company ON _employee(company_id, status);

CREATE TABLE IF NOT EXISTS _leave_request (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  employee_id TEXT NOT NULL REFERENCES _employee(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('annual', 'sick', 'unpaid')),
  from_date TEXT NOT NULL,
  to_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'rejected')),
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_leave_employee ON _leave_request(employee_id, status);

CREATE TABLE IF NOT EXISTS _payroll_run (
  id TEXT PRIMARY KEY NOT NULL,
  company_id TEXT NOT NULL REFERENCES _company(id) ON DELETE CASCADE,
  period TEXT NOT NULL,
  entry_date TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted')),
  actor TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (company_id, period)
);

CREATE INDEX IF NOT EXISTS idx_payroll_company ON _payroll_run(company_id, period DESC);

CREATE TABLE IF NOT EXISTS _payslip (
  id TEXT PRIMARY KEY NOT NULL,
  run_id TEXT NOT NULL REFERENCES _payroll_run(id) ON DELETE CASCADE,
  employee_id TEXT NOT NULL REFERENCES _employee(id) ON DELETE RESTRICT,
  gross INTEGER NOT NULL CHECK (gross >= 0),
  deductions INTEGER NOT NULL DEFAULT 0 CHECK (deductions >= 0),
  net INTEGER NOT NULL CHECK (net >= 0),
  entry_id TEXT REFERENCES _journal_entry(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (run_id, employee_id)
);

CREATE INDEX IF NOT EXISTS idx_payslip_run ON _payslip(run_id);
