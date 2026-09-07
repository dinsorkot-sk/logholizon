use logholizon_core::{db, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn m2_company_currency_crud() {
    let pool = setup().await;
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    repository::create_currency(&pool, "USD", "US Dollar", 2)
        .await
        .unwrap();
    let company = repository::create_company(&pool, "Acme Co.", "THB")
        .await
        .unwrap();
    assert_eq!(company.base_currency, "THB");

    // Unknown base currency rejected.
    assert!(repository::create_company(&pool, "Bad Co.", "XXX")
        .await
        .is_err());
    // Bad ISO code rejected.
    assert!(repository::create_currency(&pool, "thb", "Lower", 2)
        .await
        .is_err());
}

#[tokio::test]
async fn m2_fx_convert_rounds() {
    let pool = setup().await;
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    repository::create_currency(&pool, "USD", "US Dollar", 2)
        .await
        .unwrap();
    let company = repository::create_company(&pool, "Acme Co.", "THB")
        .await
        .unwrap();
    repository::set_fx_rate(&pool, &company.id, "USD", "THB", 35.5, "2026-09-01")
        .await
        .unwrap();

    // 1099 minor units * 35.5 = 39014.5 -> rounds to 39015.
    let converted = repository::convert_money(&pool, &company.id, 1099, "USD", "THB")
        .await
        .unwrap();
    assert_eq!(converted.amount, 39015);
    assert_eq!(converted.currency, "THB");

    // Same currency returns unchanged.
    let same = repository::convert_money(&pool, &company.id, 1099, "THB", "THB")
        .await
        .unwrap();
    assert_eq!(same.amount, 1099);

    // Missing pair rejected.
    assert!(
        repository::convert_money(&pool, &company.id, 100, "THB", "USD")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn m2_tax_calc_modes() {
    // Exclusive: tax on top.
    let exclusive = repository::calc_tax(10000, 0.07, false, false);
    assert_eq!(
        (exclusive.net, exclusive.tax, exclusive.gross),
        (10000, 700, 10700)
    );
    // Inclusive: extract from gross.
    let inclusive = repository::calc_tax(10700, 0.07, true, false);
    assert_eq!(
        (inclusive.net, inclusive.tax, inclusive.gross),
        (10000, 700, 10700)
    );
    // Withholding: subtract from amount.
    let withholding = repository::calc_tax(10000, 0.03, false, true);
    assert_eq!(
        (withholding.net, withholding.tax, withholding.gross),
        (9700, 300, 10000)
    );

    // Tax rule CRUD round-trip.
    let pool = setup().await;
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    let company = repository::create_company(&pool, "Acme Co.", "THB")
        .await
        .unwrap();
    let rule = repository::create_tax_rule(&pool, &company.id, "VAT 7%", 0.07, false, false)
        .await
        .unwrap();
    assert!(!rule.is_inclusive);
    let rules = repository::list_tax_rules(&pool, &company.id)
        .await
        .unwrap();
    assert_eq!(rules.len(), 1);
    // Invalid rate rejected.
    assert!(
        repository::create_tax_rule(&pool, &company.id, "Bad", 2.0, false, false)
            .await
            .is_err()
    );
}
