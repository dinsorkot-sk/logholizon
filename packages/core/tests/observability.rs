use logholizon_core::{db, observability};
use serde_json::json;

#[tokio::test]
async fn observability_records_and_filters_correlation() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    observability::record(
        &pool,
        "warn",
        "security",
        "permission_denied",
        Some("alice"),
        Some("req-test"),
        Some("corr-test"),
        Some("route"),
        Some("/v1/admin"),
        Some(403),
        Some(12),
        "denied",
        &json!({"method":"GET"}),
    )
    .await
    .unwrap();
    let filter = observability::ObservabilityFilter {
        correlation_id: Some("corr-test".into()),
        ..Default::default()
    };
    let (total, items) = observability::list(&pool, &filter, 20, 0).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].action, "permission_denied");
    assert_eq!(items[0].request_id.as_deref(), Some("req-test"));
    assert_eq!(items[0].status_code, Some(403));
}
