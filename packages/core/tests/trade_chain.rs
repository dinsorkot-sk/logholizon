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
        ("1100", "Receivables", "asset"),
        ("2100", "Payables", "liability"),
        ("4000", "Revenue", "income"),
        ("5000", "Purchases", "expense"),
    ] {
        repository::create_gl_account(pool, &company_id, code, name, account_type)
            .await
            .unwrap();
    }
    company_id
}

fn line(description: &str, qty: f64, unit_price: i64) -> repository::TradeLineInput {
    repository::TradeLineInput {
        product_id: None,
        description: description.to_string(),
        qty,
        uom_id: None,
        unit_price,
        tax_rule_id: None,
    }
}

#[tokio::test]
async fn trade_chain_lead_to_posted_invoice() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    let lead = repository::create_trade_doc(
        &pool,
        &company_id,
        "sale",
        "lead",
        None,
        "Customer A",
        "THB",
        "2026-09-01",
        None,
        &[line("Widget", 2.0, 5000)],
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(lead.status, "new");
    assert_eq!(lead.subtotal, 10000);

    let quote = repository::convert_lead_to_quotation(&pool, &lead.id, None, Some("admin"))
        .await
        .unwrap();
    assert_eq!(quote.doc_type, "quotation");
    assert_eq!(quote.status, "draft");
    assert_eq!(quote.source_id.as_deref(), Some(lead.id.as_str()));
    let refreshed_lead = repository::list_trade_docs(&pool, &company_id, Some("lead"))
        .await
        .unwrap();
    assert_eq!(refreshed_lead[0].status, "qualified");

    // Double-convert rejected.
    let err = repository::convert_lead_to_quotation(&pool, &lead.id, None, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already converted"));

    let order = repository::confirm_quotation_to_order(&pool, &quote.id, None, Some("admin"))
        .await
        .unwrap();
    assert_eq!(order.doc_type, "order");
    assert_eq!(order.status, "confirmed");

    // Double-confirm rejected.
    let err = repository::confirm_quotation_to_order(&pool, &quote.id, None, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already converted"));

    let invoiced = repository::invoice_from_order(&pool, &order.id, Some("admin"))
        .await
        .unwrap();
    assert_eq!(invoiced.status, "done");
    let invoice_id = invoiced.invoice_id.clone().unwrap();
    let invoices = repository::list_invoices(&pool, &company_id).await.unwrap();
    let invoice = invoices.into_iter().find(|i| i.id == invoice_id).unwrap();
    assert_eq!(invoice.status, "posted");
    assert_eq!(invoice.base_total, 10000);

    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, trial.total_credit);
    assert_eq!(trial.total_debit, 10000);

    // Already-invoiced rejected.
    let err = repository::invoice_from_order(&pool, &order.id, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already invoiced"));
}

#[tokio::test]
async fn trade_fractional_qty_rejected_at_invoice() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    let order = repository::create_trade_doc(
        &pool,
        &company_id,
        "sale",
        "order",
        Some("confirmed"),
        "Customer B",
        "THB",
        "2026-09-01",
        None,
        &[line("Bulk goods", 1.5, 1000)],
        None,
    )
    .await
    .unwrap();
    let err = repository::invoice_from_order(&pool, &order.id, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("fractional qty"));
}

#[tokio::test]
async fn trade_invalid_status_rejected() {
    let pool = setup().await;
    let company_id = company_with_accounts(&pool).await;
    let err = repository::create_trade_doc(
        &pool,
        &company_id,
        "sale",
        "lead",
        Some("draft"),
        "Customer C",
        "THB",
        "2026-09-01",
        None,
        &[],
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("invalid status"));
}
