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

#[tokio::test]
async fn status_field_name_is_metadata_driven() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "ticket", "ticket", "Ticket")
        .await
        .unwrap();
    repository::create_field(&pool, "ticket", "title", "text", true, false)
        .await
        .unwrap();
    let state = repository::create_field(&pool, "ticket", "state", "select", true, true)
        .await
        .unwrap();
    assert!(state.is_status);
    for (value, label) in [("new", "New"), ("open", "Open"), ("closed", "Closed")] {
        repository::create_field_option(&pool, &state.id, value, label)
            .await
            .unwrap();
    }
    // workflow rows are seeded directly (no workflow CRUD yet)
    sqlx::query("INSERT INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, 'ticket', ?, ?, ?)")
        .bind("ticket_new")
        .bind("new")
        .bind("New")
        .bind(0)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, 'ticket', ?, ?, ?)")
        .bind("ticket_open_transition")
        .bind("new")
        .bind("open")
        .bind("open")
        .execute(&pool)
        .await
        .unwrap();

    repository::create_document(
        &pool,
        "t-1",
        "ticket",
        &json!({"title": "Login broken", "state": "new"}),
    )
    .await
    .unwrap();

    // transition uses the metadata-driven status field name
    let opened = repository::transition_document(&pool, "t-1", "open")
        .await
        .unwrap();
    assert_eq!(opened.payload["state"], "open");

    // count groups by the metadata-driven status field
    let counts = repository::count_documents_by_status(&pool, "ticket")
        .await
        .unwrap();
    assert_eq!(counts.len(), 1);
    assert_eq!(counts[0].status, "open");
    assert_eq!(counts[0].count, 1);

    // list filter uses the metadata-driven status field
    let filter = repository::ListDocumentsFilter {
        status: Some("open".to_string()),
        ..Default::default()
    };
    let list = repository::list_documents(&pool, "ticket", 50, 0, &filter)
        .await
        .unwrap();
    assert_eq!(list.total, 1);
    let filter = repository::ListDocumentsFilter {
        status: Some("new".to_string()),
        ..Default::default()
    };
    let list = repository::list_documents(&pool, "ticket", 50, 0, &filter)
        .await
        .unwrap();
    assert_eq!(list.total, 0);
}

#[tokio::test]
async fn entity_without_status_field_rejects_transition() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "note", "note", "Note")
        .await
        .unwrap();
    repository::create_field(&pool, "note", "body", "text", false, false)
        .await
        .unwrap();
    repository::create_document(&pool, "n-1", "note", &json!({"body": "hello"}))
        .await
        .unwrap();
    let err = repository::transition_document(&pool, "n-1", "submit")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no status field"));
    // counts are empty, not an error
    let counts = repository::count_documents_by_status(&pool, "note")
        .await
        .unwrap();
    assert!(counts.is_empty());
}
