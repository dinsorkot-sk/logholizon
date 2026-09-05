use logholizon_core::{db, repository, seed};
use serde_json::json;

#[tokio::test]
async fn transition_follows_workflow_and_audits() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    let doc = repository::create_document(
        &pool,
        "wo-1",
        "work_order",
        &json!({"title": "Fix pump", "status": "draft", "priority": "high"}),
    )
    .await
    .unwrap();
    assert_eq!(doc.payload["status"], "draft");

    let submitted = repository::transition_document(&pool, "wo-1", "submit")
        .await
        .unwrap();
    assert_eq!(submitted.payload["status"], "open");

    let done = repository::transition_document(&pool, "wo-1", "done")
        .await
        .unwrap();
    assert_eq!(done.payload["status"], "done");

    let audit = repository::list_document_audit(&pool, "wo-1", 10, 0)
        .await
        .unwrap();
    let actions: Vec<&str> = audit
        .items
        .iter()
        .map(|entry| entry.action.as_str())
        .collect();
    assert_eq!(actions, vec!["transition", "transition", "create"]);
}

#[tokio::test]
async fn invalid_transition_is_rejected() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    repository::create_document(
        &pool,
        "wo-2",
        "work_order",
        &json!({"title": "Skip", "status": "draft", "priority": "low"}),
    )
    .await
    .unwrap();
    let err = repository::transition_document(&pool, "wo-2", "done")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid transition"));
    let doc = repository::get_document(&pool, "wo-2").await.unwrap();
    assert_eq!(doc.payload["status"], "draft");
}

#[tokio::test]
async fn dashboard_counts_group_by_status() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    for (id, status) in [
        ("wo-a", "draft"),
        ("wo-b", "draft"),
        ("wo-c", "open"),
        ("wo-d", "done"),
    ] {
        repository::create_document(
            &pool,
            id,
            "work_order",
            &json!({"title": id, "status": status, "priority": "low"}),
        )
        .await
        .unwrap();
    }
    let counts = repository::count_documents_by_status(&pool, "work_order")
        .await
        .unwrap();
    let by_status: std::collections::HashMap<&str, i64> = counts
        .iter()
        .map(|entry| (entry.status.as_str(), entry.count))
        .collect();
    assert_eq!(by_status.get("draft"), Some(&2));
    assert_eq!(by_status.get("open"), Some(&1));
    assert_eq!(by_status.get("done"), Some(&1));
}
