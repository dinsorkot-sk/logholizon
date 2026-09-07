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

async fn uoms(pool: &sqlx::SqlitePool, company_id: &str) -> (String, String) {
    let pcs = repository::create_uom(pool, company_id, "PCS", "Pieces", "qty", 1.0, true)
        .await
        .unwrap();
    let box_uom = repository::create_uom(pool, company_id, "BOX", "Box", "qty", 12.0, false)
        .await
        .unwrap();
    (pcs.id, box_uom.id)
}

#[tokio::test]
async fn stock_moving_average_and_on_hand() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let (_pcs, box_uom) = uoms(&pool, &company_id).await;
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
    repository::apply_stock_move(
        &pool,
        &company_id,
        "prod-1",
        "wh-1",
        "in",
        1.0,
        Some(&box_uom),
        120,
        "2026-09-02",
        None,
        None,
    )
    .await
    .unwrap();
    let balance = repository::get_stock_balance(&pool, &company_id, "prod-1", "wh-1")
        .await
        .unwrap();
    assert!((balance.qty_base - 22.0).abs() < 1e-6);
    assert_eq!(balance.avg_cost, 111);
    assert_eq!(balance.total_value, 2442);
    let out = repository::apply_stock_move(
        &pool,
        &company_id,
        "prod-1",
        "wh-1",
        "out",
        2.0,
        None,
        0,
        "2026-09-03",
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(out.total_value, 222);
    let balance = repository::get_stock_balance(&pool, &company_id, "prod-1", "wh-1")
        .await
        .unwrap();
    assert!((balance.qty_base - 20.0).abs() < 1e-6);
    assert_eq!(balance.avg_cost, 111);
}

#[tokio::test]
async fn stock_uom_dimension_mismatch_rejected() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let (pcs, _) = uoms(&pool, &company_id).await;
    let kg = repository::create_uom(&pool, &company_id, "KG", "Kilogram", "weight", 1.0, true)
        .await
        .unwrap();
    let err = repository::convert_uom(&pool, &company_id, 1.0, &pcs, &kg.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("dimension mismatch"));
}

#[tokio::test]
async fn stock_negative_out_rejected() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    uoms(&pool, &company_id).await;
    repository::apply_stock_move(
        &pool,
        &company_id,
        "prod-1",
        "wh-1",
        "in",
        1.0,
        None,
        100,
        "2026-09-01",
        None,
        None,
    )
    .await
    .unwrap();
    let err = repository::apply_stock_move(
        &pool,
        &company_id,
        "prod-1",
        "wh-1",
        "out",
        2.0,
        None,
        0,
        "2026-09-02",
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("insufficient stock"));
}
