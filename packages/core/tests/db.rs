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

// --- Phase C: SQLite single-host concurrency policy ---

#[tokio::test]
async fn connect_applies_wal_concurrency_policy() {
    // WAL requires a file-backed database; in-memory SQLite stays in
    // `memory` journal mode, so use a temp file to assert the policy.
    let dir = std::env::temp_dir().join(format!(
        "logholizon-test-wal-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let path = dir.join("core.db");
    let url = format!("sqlite://{}", path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&url).await.unwrap();
    let snapshot = db::pragma_snapshot(&pool).await.unwrap();
    assert_eq!(snapshot.journal_mode, "WAL");
    assert_eq!(snapshot.busy_timeout_ms, db::BUSY_TIMEOUT_MS);
    assert_eq!(snapshot.synchronous, "NORMAL");
    assert!(snapshot.foreign_keys);
    pool.close().await;
    tokio::fs::remove_dir_all(&dir).await.ok();
}

#[tokio::test]
async fn concurrent_writes_serialize_without_busy_errors() {
    // File-backed DB: 10 parallel writers must all succeed (single-writer
    // serialization + busy_timeout), proving the pool absorbs bursts.
    let dir = std::env::temp_dir().join(format!(
        "logholizon-test-concurrency-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let path = dir.join("core.db");
    let url = format!("sqlite://{}", path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&url).await.unwrap();
    db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    let mut handles = Vec::new();
    for i in 0..10 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            logholizon_core::repository::create_document(
                &pool,
                &format!("concurrent-{i}"),
                "work_order",
                &serde_json::json!({"title": format!("job {i}"), "status": "draft"}),
                None,
            )
            .await
        }));
    }
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc WHERE id LIKE 'concurrent-%'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 10);
    pool.close().await;
    tokio::fs::remove_dir_all(&dir).await.ok();
}
