use logholizon_core::{db, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    pool
}

async fn company(pool: &sqlx::SqlitePool) -> String {
    repository::create_company(pool, "Acme Co.", "THB")
        .await
        .unwrap()
        .id
}

async fn accounts(pool: &sqlx::SqlitePool, company_id: &str) -> (String, String, String, String) {
    for (code, name, account_type) in [
        ("1000", "Cash", "asset"),
        ("1100", "Receivables", "asset"),
        ("2100", "Payables", "liability"),
        ("4000", "Revenue", "income"),
        ("5000", "Purchases", "expense"),
    ] {
        repository::create_gl_account(pool, company_id, code, name, account_type)
            .await
            .unwrap();
    }
    let list = repository::list_gl_accounts(pool, company_id)
        .await
        .unwrap();
    let id = |code: &str| list.iter().find(|a| a.code == code).unwrap().id.clone();
    (id("1000"), id("1100"), id("2100"), id("4000"))
}

fn line(description: &str, quantity: i64, unit_price: i64) -> repository::InvoiceLineInput {
    repository::InvoiceLineInput {
        description: description.to_string(),
        quantity,
        unit_price,
        tax_rule_id: None,
    }
}

#[tokio::test]
async fn invoice_post_totals_and_gl() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    accounts(&pool, &company_id).await;
    let invoice = repository::create_invoice(
        &pool,
        &company_id,
        "sale",
        "Customer A",
        "THB",
        "2026-09-01",
        &[line("Widget", 2, 5000)],
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(invoice.status, "draft");
    let posted = repository::post_invoice(&pool, &invoice.id).await.unwrap();
    assert_eq!(posted.status, "posted");
    assert_eq!(posted.base_total, 10000);
    assert!(posted.entry_id.is_some());

    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, trial.total_credit);
    assert_eq!(trial.total_debit, 10000);
}

#[tokio::test]
async fn payment_allocate_partial_and_paid() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    accounts(&pool, &company_id).await;
    let invoice = repository::create_invoice(
        &pool,
        &company_id,
        "sale",
        "Customer A",
        "THB",
        "2026-09-01",
        &[line("Widget", 1, 10000)],
        None,
    )
    .await
    .unwrap();
    repository::post_invoice(&pool, &invoice.id).await.unwrap();
    let payment = repository::create_payment(
        &pool,
        &company_id,
        "receive",
        "Customer A",
        "THB",
        10000,
        "2026-09-02",
        None,
    )
    .await
    .unwrap();
    let allocated = repository::allocate_payment(&pool, &payment.id, &invoice.id, 4000)
        .await
        .unwrap();
    assert_eq!(allocated.remaining, 6000);
    let allocated = repository::allocate_payment(&pool, &payment.id, &invoice.id, 6000)
        .await
        .unwrap();
    assert_eq!(allocated.remaining, 0);
    let refreshed = repository::list_invoices(&pool, &company_id).await.unwrap();
    let invoice = refreshed.into_iter().find(|i| i.id == invoice.id).unwrap();
    assert_eq!(invoice.status, "paid");
    assert_eq!(invoice.remaining, 0);
}

#[tokio::test]
async fn payment_overpay_rejected() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    accounts(&pool, &company_id).await;
    let invoice = repository::create_invoice(
        &pool,
        &company_id,
        "sale",
        "Customer A",
        "THB",
        "2026-09-01",
        &[line("Widget", 1, 5000)],
        None,
    )
    .await
    .unwrap();
    repository::post_invoice(&pool, &invoice.id).await.unwrap();
    let payment = repository::create_payment(
        &pool,
        &company_id,
        "receive",
        "Customer A",
        "THB",
        5000,
        "2026-09-02",
        None,
    )
    .await
    .unwrap();
    let err = repository::allocate_payment(&pool, &payment.id, &invoice.id, 6000)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("exceeds"));
}

#[tokio::test]
async fn void_locked_invoice_blocked() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    accounts(&pool, &company_id).await;
    let invoice = repository::create_invoice(
        &pool,
        &company_id,
        "sale",
        "Customer A",
        "THB",
        "2026-09-01",
        &[line("Widget", 1, 5000)],
        None,
    )
    .await
    .unwrap();
    repository::post_invoice(&pool, &invoice.id).await.unwrap();
    repository::lock_period(&pool, &company_id, "2026-09", Some("admin"))
        .await
        .unwrap();
    let err = repository::void_invoice(&pool, &invoice.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("period locked"));
}
