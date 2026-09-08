-- Drop legacy hardcoded ERP domain tables (0018-0025).
-- Forward-only: removes _company/_currency/_fx/_tax/_ledger/_invoice/
-- _payment/_stock/_trade/_hr/_mfg/_pos tables. Generic runtime
-- (_meta_*, _doc, _workflow_*, permissions, views, layouts,
-- notifications, reports, collab) is untouched.
-- Child tables first to satisfy FK constraints.

DROP TABLE IF EXISTS _pos_line;
DROP TABLE IF EXISTS _pos_order;
DROP TABLE IF EXISTS _pos_session;
DROP TABLE IF EXISTS _mfg_order;
DROP TABLE IF EXISTS _bom_line;
DROP TABLE IF EXISTS _bom;
DROP TABLE IF EXISTS _payslip;
DROP TABLE IF EXISTS _payroll_run;
DROP TABLE IF EXISTS _leave_request;
DROP TABLE IF EXISTS _employee;
DROP TABLE IF EXISTS _trade_line;
DROP TABLE IF EXISTS _trade_doc;
DROP TABLE IF EXISTS _stock_ledger_entry;
DROP TABLE IF EXISTS _stock_balance;
DROP TABLE IF EXISTS _payment_allocation;
DROP TABLE IF EXISTS _invoice_line;
DROP TABLE IF EXISTS _payment;
DROP TABLE IF EXISTS _invoice;
DROP TABLE IF EXISTS _journal_line;
DROP TABLE IF EXISTS _journal_entry;
DROP TABLE IF EXISTS _period_lock;
DROP TABLE IF EXISTS _gl_account;
DROP TABLE IF EXISTS _uom;
DROP TABLE IF EXISTS _fx_rate;
DROP TABLE IF EXISTS _tax_rule;
DROP TABLE IF EXISTS _company;
DROP TABLE IF EXISTS _currency;
