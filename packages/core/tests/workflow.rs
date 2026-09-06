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
        None,
    )
    .await
    .unwrap();
    assert_eq!(doc.payload["status"], "draft");

    let submitted = repository::transition_document(&pool, "wo-1", "submit", None, None)
        .await
        .unwrap();
    assert_eq!(submitted.payload["status"], "open");

    let done = repository::transition_document(&pool, "wo-1", "done", None, None)
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
        None,
    )
    .await
    .unwrap();
    let err = repository::transition_document(&pool, "wo-2", "done", None, None)
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
            None,
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
    // workflow rows go through the CRUD API
    repository::create_workflow_state(&pool, "ticket", "new", "New")
        .await
        .unwrap();
    repository::create_workflow_state(&pool, "ticket", "open", "Open")
        .await
        .unwrap();
    repository::create_workflow_transition(&pool, "ticket", "new", "open", "open")
        .await
        .unwrap();

    repository::create_document(
        &pool,
        "t-1",
        "ticket",
        &json!({"title": "Login broken", "state": "new"}),
        None,
    )
    .await
    .unwrap();

    // transition uses the metadata-driven status field name
    let opened = repository::transition_document(&pool, "t-1", "open", None, None)
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
async fn workflow_crud_validates_and_guards_deletes() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "ticket", "ticket", "Ticket")
        .await
        .unwrap();

    let new_state = repository::create_workflow_state(&pool, "ticket", "new", "New")
        .await
        .unwrap();
    assert_eq!(new_state.name, "new");
    assert_eq!(new_state.position, 0);
    let open_state = repository::create_workflow_state(&pool, "ticket", "open", "Open")
        .await
        .unwrap();
    assert_eq!(open_state.position, 1);

    // duplicate state name conflicts
    assert!(
        repository::create_workflow_state(&pool, "ticket", "new", "New again")
            .await
            .is_err()
    );
    // bad state name rejected
    assert!(
        repository::create_workflow_state(&pool, "ticket", "Bad Name", "Bad")
            .await
            .is_err()
    );
    // missing entity rejected
    assert!(
        repository::create_workflow_state(&pool, "missing", "new", "New")
            .await
            .is_err()
    );

    let transition = repository::create_workflow_transition(&pool, "ticket", "new", "open", "open")
        .await
        .unwrap();
    assert_eq!(transition.action, "open");

    // unknown state rejected
    assert!(
        repository::create_workflow_transition(&pool, "ticket", "new", "closed", "close")
            .await
            .is_err()
    );
    // same from/to rejected
    assert!(
        repository::create_workflow_transition(&pool, "ticket", "new", "new", "noop")
            .await
            .is_err()
    );

    // state used by a transition cannot be deleted
    let err = repository::delete_workflow_state(&pool, &new_state.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("used by transitions"));

    repository::delete_workflow_transition(&pool, &transition.id)
        .await
        .unwrap();
    repository::delete_workflow_state(&pool, &new_state.id)
        .await
        .unwrap();
    let workflow = repository::get_workflow(&pool, "ticket").await.unwrap();
    assert_eq!(workflow.states.len(), 1);
    assert!(workflow.transitions.is_empty());

    // label update works
    let updated = repository::update_workflow_state(&pool, &open_state.id, "Opened")
        .await
        .unwrap();
    assert_eq!(updated.label, "Opened");
}

#[tokio::test]
async fn seed_provides_pm_schedule_workflow() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    let workflow = repository::get_workflow(&pool, "pm_schedule")
        .await
        .unwrap();
    assert_eq!(workflow.states.len(), 3);
    assert_eq!(workflow.transitions.len(), 2);
    repository::create_document(
        &pool,
        "pm-1",
        "pm_schedule",
        &json!({"title": "Check pump", "due_date": "2026-09-06", "status": "draft"}),
        None,
    )
    .await
    .unwrap();
    let scheduled = repository::transition_document(&pool, "pm-1", "schedule", None, None)
        .await
        .unwrap();
    assert_eq!(scheduled.payload["status"], "scheduled");
    let done = repository::transition_document(&pool, "pm-1", "complete", None, None)
        .await
        .unwrap();
    assert_eq!(done.payload["status"], "done");
}

#[tokio::test]
async fn pm_summary_counts_open_overdue_done() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    // open + overdue (due yesterday)
    repository::create_document(
        &pool,
        "pm-1",
        "pm_schedule",
        &json!({"title": "Overdue", "due_date": "2020-01-01", "status": "scheduled"}),
        None,
    )
    .await
    .unwrap();
    // open + not overdue (due tomorrow)
    repository::create_document(
        &pool,
        "pm-2",
        "pm_schedule",
        &json!({"title": "Future", "due_date": "2099-01-01", "status": "draft"}),
        None,
    )
    .await
    .unwrap();
    // done
    repository::create_document(
        &pool,
        "pm-3",
        "pm_schedule",
        &json!({"title": "Done", "due_date": "2020-01-01", "status": "done"}),
        None,
    )
    .await
    .unwrap();

    let summary = repository::pm_summary(&pool, "pm_schedule").await.unwrap();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.open, 2);
    assert_eq!(summary.overdue, 1);
    assert_eq!(summary.done_this_week, 1);
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
    repository::create_document(&pool, "n-1", "note", &json!({"body": "hello"}), None)
        .await
        .unwrap();
    let err = repository::transition_document(&pool, "n-1", "submit", None, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no status field"));
    // counts are empty, not an error
    let counts = repository::count_documents_by_status(&pool, "note")
        .await
        .unwrap();
    assert!(counts.is_empty());
}
