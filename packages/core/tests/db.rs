use logholizon_core::db;

#[tokio::test]
async fn migrate_and_check_in_memory() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    assert!(db::integrity_check(&pool).await.unwrap());
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(tables.contains(&"_meta_entity".to_string()));
    assert!(tables.contains(&"_meta_field".to_string()));
    assert!(tables.contains(&"_meta_field_option".to_string()));
    // Module runtime tables exist.
    assert!(tables.contains(&"_module".to_string()));
    assert!(tables.contains(&"_module_version".to_string()));
    assert!(tables.contains(&"_automation".to_string()));
    // Legacy hardcoded ERP tables (0018-0025) must be gone after 0026.
    for legacy in [
        "_company",
        "_currency",
        "_fx_rate",
        "_tax_rule",
        "_gl_account",
        "_journal_entry",
        "_journal_line",
        "_period_lock",
        "_invoice",
        "_invoice_line",
        "_payment",
        "_payment_allocation",
        "_uom",
        "_stock_balance",
        "_stock_ledger_entry",
        "_trade_doc",
        "_trade_line",
        "_employee",
        "_leave_request",
        "_payroll_run",
        "_payslip",
        "_bom",
        "_bom_line",
        "_mfg_order",
        "_pos_session",
        "_pos_order",
        "_pos_line",
    ] {
        assert!(
            !tables.contains(&legacy.to_string()),
            "legacy table should be dropped: {legacy}"
        );
    }
    // _meta_entity carries the module link column.
    let columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('_meta_entity') ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(columns.contains(&"module_id".to_string()));
    // _meta_field carries business rule columns.
    let field_columns: Vec<String> =
        sqlx::query_scalar("SELECT name FROM pragma_table_info('_meta_field') ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    for column in [
        "is_unique",
        "min_value",
        "max_value",
        "pattern",
        "min_length",
        "max_length",
        "default_value",
        "auto_number_prefix",
        "auto_number_width",
    ] {
        assert!(
            field_columns.contains(&column.to_string()),
            "missing _meta_field column: {column}"
        );
    }
}
