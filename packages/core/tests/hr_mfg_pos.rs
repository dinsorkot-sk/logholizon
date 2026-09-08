use logholizon_core::{db, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    pool
}

async fn company_with_accounts(pool: &sqlx::SqlitePool) -> String {
    let company_id = repository::create_company(pool, "Acme Co.", "THB")
        .await
        .unwrap()
        .id;
    for (code, name, account_type) in [
        ("1000", "Cash", "asset"),
        ("1400", "Inventory", "asset"),
        ("1410", "Work in Progress", "asset"),
        ("4000", "Revenue", "income"),
        ("5290", "Cash Short", "expense"),
        ("6000", "Salaries", "expense"),
        ("2200", "Salary Payable", "liability"),
    ] {
        repository::create_gl_account(pool, &company_id, code, name, account_type)
            .await
            .unwrap();
    }
    company_id
}

#[tokio::test]
async fn hr_payroll_posts_balanced_journals() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    let alice = repository::create_employee(
        &pool,
        &company_id,
        "E001",
        "Alice",
        50000,
        "THB",
        "2026-01-01",
        Some("admin"),
    )
    .await
    .unwrap();
    let bob = repository::create_employee(
        &pool,
        &company_id,
        "E002",
        "Bob",
        40000,
        "THB",
        "2026-01-01",
        None,
    )
    .await
    .unwrap();
    let run =
        repository::create_payroll_run(&pool, &company_id, "2026-09", "2026-09-30", Some("admin"))
            .await
            .unwrap();
    assert_eq!(run.status, "draft");
    repository::add_payslip(&pool, &run.id, &alice.id, 50000, 5000)
        .await
        .unwrap();
    repository::add_payslip(&pool, &run.id, &bob.id, 40000, 0)
        .await
        .unwrap();
    let posted = repository::post_payroll_run(&pool, &run.id).await.unwrap();
    assert_eq!(posted.status, "posted");
    assert_eq!(posted.total_gross, 90000);
    assert_eq!(posted.total_net, 85000);
    assert!(posted.payslips.iter().all(|s| s.entry_id.is_some()));

    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, trial.total_credit);
    assert_eq!(trial.total_debit, 90000);

    // Repost rejected.
    let err = repository::post_payroll_run(&pool, &run.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("payroll run is posted"));
}

#[tokio::test]
async fn hr_leave_overlap_rejected() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    let alice = repository::create_employee(
        &pool,
        &company_id,
        "E001",
        "Alice",
        50000,
        "THB",
        "2026-01-01",
        None,
    )
    .await
    .unwrap();
    repository::request_leave(
        &pool,
        &company_id,
        &alice.id,
        "annual",
        "2026-09-10",
        "2026-09-12",
        None,
    )
    .await
    .unwrap();
    let err = repository::request_leave(
        &pool,
        &company_id,
        &alice.id,
        "sick",
        "2026-09-11",
        "2026-09-13",
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("overlaps"));

    let leaves = repository::list_leave_requests(&pool, &company_id)
        .await
        .unwrap();
    let approved = repository::decide_leave(&pool, &leaves[0].id, true)
        .await
        .unwrap();
    assert_eq!(approved.status, "approved");
    // Decide twice rejected.
    let err = repository::decide_leave(&pool, &leaves[0].id, false)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("leave is approved"));
}

#[tokio::test]
async fn mfg_consume_produce_and_cost_rollup() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    // Component stock: 10 units @ 100 each.
    repository::apply_stock_move(
        &pool,
        &company_id,
        "comp-1",
        "wh-1",
        "in",
        10.0,
        None,
        100,
        "2026-09-01",
        None,
        None,
    )
    .await
    .unwrap();
    let bom = repository::create_bom(
        &pool,
        &company_id,
        "finished-1",
        "Widget BOM",
        &[repository::BomLineInput {
            component_id: "comp-1".to_string(),
            qty: 2.0,
            uom_id: None,
        }],
    )
    .await
    .unwrap();
    repository::activate_bom(&pool, &bom.id).await.unwrap();
    let order = repository::create_mfg_order(
        &pool,
        &company_id,
        &bom.id,
        3.0,
        "wh-1",
        "2026-09-02",
        Some("admin"),
    )
    .await
    .unwrap();
    repository::confirm_mfg_order(&pool, &order.id)
        .await
        .unwrap();
    let done = repository::complete_mfg_order(&pool, &order.id)
        .await
        .unwrap();
    assert_eq!(done.status, "done");
    assert!(done.entry_id.is_some());

    // 6 components consumed (10 -> 4), 3 finished produced at 200 avg.
    let comp = repository::get_stock_balance(&pool, &company_id, "comp-1", "wh-1")
        .await
        .unwrap();
    assert!((comp.qty_base - 4.0).abs() < 1e-6);
    let finished = repository::get_stock_balance(&pool, &company_id, "finished-1", "wh-1")
        .await
        .unwrap();
    assert!((finished.qty_base - 3.0).abs() < 1e-6);
    assert_eq!(finished.avg_cost, 200);
    assert_eq!(finished.total_value, 600);

    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, trial.total_credit);
    assert_eq!(trial.total_debit, 600);

    // Insufficient stock rejected.
    let big = repository::create_mfg_order(
        &pool,
        &company_id,
        &bom.id,
        10.0,
        "wh-1",
        "2026-09-03",
        None,
    )
    .await
    .unwrap();
    repository::confirm_mfg_order(&pool, &big.id).await.unwrap();
    let err = repository::complete_mfg_order(&pool, &big.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("insufficient stock"));
}

#[tokio::test]
async fn pos_tender_shortfall_and_close_reconcile() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    repository::apply_stock_move(
        &pool,
        &company_id,
        "prod-1",
        "wh-1",
        "in",
        10.0,
        None,
        100,
        "2026-09-01",
        None,
        None,
    )
    .await
    .unwrap();
    let session = repository::open_pos_session(
        &pool,
        &company_id,
        "Shift 1",
        "wh-1",
        10000,
        "2026-09-02",
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(session.status, "open");

    // Tender shortfall rejected.
    let err = repository::create_pos_order(
        &pool,
        &session.id,
        Some("Walk-in"),
        "THB",
        100,
        &[repository::PosLineInput {
            product_id: "prod-1".to_string(),
            description: "Widget".to_string(),
            qty: 2.0,
            uom_id: None,
            unit_price: 5000,
            tax_rule_id: None,
        }],
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("tendered is less than total"));

    // Exact tender pays, stocks out, posts cash/income.
    let order = repository::create_pos_order(
        &pool,
        &session.id,
        Some("Walk-in"),
        "THB",
        10000,
        &[repository::PosLineInput {
            product_id: "prod-1".to_string(),
            description: "Widget".to_string(),
            qty: 2.0,
            uom_id: None,
            unit_price: 5000,
            tax_rule_id: None,
        }],
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(order.status, "paid");
    assert_eq!(order.change_due, 0);
    assert!(order.entry_id.is_some());
    let balance = repository::get_stock_balance(&pool, &company_id, "prod-1", "wh-1")
        .await
        .unwrap();
    assert!((balance.qty_base - 8.0).abs() < 1e-6);

    // Close with exact cash: no drawer journal, trial balanced.
    let closed = repository::close_pos_session(&pool, &session.id, 20000)
        .await
        .unwrap();
    assert_eq!(closed.status, "closed");
    assert_eq!(closed.sales_total, 10000);
    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, trial.total_credit);
    assert_eq!(trial.total_debit, 10000);

    // Close with shortage posts cash-short and stays balanced.
    let session2 = repository::open_pos_session(
        &pool,
        &company_id,
        "Shift 2",
        "wh-1",
        5000,
        "2026-09-03",
        None,
    )
    .await
    .unwrap();
    repository::create_pos_order(
        &pool,
        &session2.id,
        None,
        "THB",
        3000,
        &[repository::PosLineInput {
            product_id: "prod-1".to_string(),
            description: "Widget".to_string(),
            qty: 1.0,
            uom_id: None,
            unit_price: 3000,
            tax_rule_id: None,
        }],
        None,
    )
    .await
    .unwrap();
    let closed2 = repository::close_pos_session(&pool, &session2.id, 7000)
        .await
        .unwrap();
    assert_eq!(closed2.status, "closed");
    let trial2 = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial2.total_debit, trial2.total_credit);
}
