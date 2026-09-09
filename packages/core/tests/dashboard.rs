use logholizon_core::{dashboard, db};
use serde_json::json;

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn dashboard_crud_and_access_rules() {
    let pool = setup().await;
    let roles = vec!["manager".to_string()];
    let users = vec!["alice".to_string()];
    let d = dashboard::create(
        &pool,
        "Operations",
        "Overview",
        &json!([]),
        &json!({"status":"active"}),
        &roles,
        &users,
        Some("alice"),
    )
    .await
    .unwrap();
    assert!(dashboard::can_view(&d, Some("alice"), "user"));
    assert!(dashboard::can_view(&d, Some("bob"), "manager"));
    assert!(!dashboard::can_view(&d, Some("bob"), "user"));
    let updated = dashboard::update(
        &pool,
        &d.id,
        "Operations 2",
        "Updated",
        &json!([]),
        &json!({}),
        &roles,
        &users,
        true,
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "Operations 2");
    dashboard::delete(&pool, &d.id).await.unwrap();
    assert!(dashboard::get(&pool, &d.id).await.is_err());
}
