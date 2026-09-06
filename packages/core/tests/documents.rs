use logholizon_core::{db, repository};
use serde_json::json;

async fn seeded_pool() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO _meta_entity (id, name, label) VALUES ('ticket', 'ticket', 'Ticket')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO _meta_field (id, entity_id, name, type, required) VALUES ('f1', 'ticket', 'title', 'text', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO _meta_field (id, entity_id, name, type, required) VALUES ('f2', 'ticket', 'priority', 'number', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn computed_field_interpolates_on_read() {
    use logholizon_core::seed;
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    repository::create_field(
        &pool,
        "work_order",
        "summary",
        "computed",
        false,
        false,
        None,
        Some("{title} [{status}]"),
    )
    .await
    .unwrap();

    // Incoming computed keys are rejected.
    assert!(repository::create_document(
        &pool,
        "w1",
        "work_order",
        &json!({"title": "Pump", "status": "draft", "summary": "x"}),
        None,
    )
    .await
    .is_err());

    repository::create_document(
        &pool,
        "w1",
        "work_order",
        &json!({"title": "Pump", "status": "draft"}),
        None,
    )
    .await
    .unwrap();
    let doc = repository::get_document(&pool, "w1").await.unwrap();
    assert_eq!(doc.payload["summary"], "Pump [draft]");

    // Computed without expr rejected.
    assert!(repository::create_field(
        &pool,
        "work_order",
        "bad",
        "computed",
        false,
        false,
        None,
        None
    )
    .await
    .is_err());
}

#[tokio::test]
async fn document_crud_validates_payload() {
    let pool = seeded_pool().await;
    let doc =
        repository::create_document(&pool, "d1", "ticket", &json!({"title": "Fix pump"}), None)
            .await
            .unwrap();
    assert_eq!(doc.payload["title"], "Fix pump");
    repository::update_document(&pool, "d1", &json!({"title": "Fixed"}), None, None)
        .await
        .unwrap();
    let audit = repository::list_document_audit(&pool, "d1", 10, 0)
        .await
        .unwrap();
    assert_eq!(audit.total, 2);
    assert_eq!(audit.items[0].action, "update");
    assert_eq!(audit.items[1].action, "create");

    let err = repository::create_document(&pool, "d2", "ticket", &json!({"priority": 1}), None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("missing required field"));

    let err = repository::create_document(&pool, "d3", "ticket", &json!({"title": 1}), None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid value"));

    let err = repository::create_document(
        &pool,
        "d4",
        "ticket",
        &json!({"title": "x", "nope": 1}),
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("unknown field"));

    let updated = repository::update_document(&pool, "d1", &json!({"title": "Fixed"}), None, None)
        .await
        .unwrap();
    assert_eq!(updated.payload["title"], "Fixed");

    let list = repository::list_documents(&pool, "ticket", 10, 0, &Default::default())
        .await
        .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items.len(), 1);

    repository::delete_document(&pool, "d1", None)
        .await
        .unwrap();
    assert!(repository::get_document(&pool, "d1").await.is_err());
}

#[tokio::test]
async fn global_audit_lists_filters_and_paginates() {
    let pool = seeded_pool().await;
    repository::create_document(&pool, "d1", "ticket", &json!({"title": "Fix pump"}), None)
        .await
        .unwrap();
    repository::update_document(&pool, "d1", &json!({"title": "Fixed"}), None, None)
        .await
        .unwrap();
    repository::create_document(
        &pool,
        "d2",
        "ticket",
        &json!({"title": "Replace belt"}),
        None,
    )
    .await
    .unwrap();

    // No filter: all entries with entity label.
    let all = repository::list_global_audit(&pool, 50, 0, &Default::default())
        .await
        .unwrap();
    assert_eq!(all.total, 3);
    assert!(all.items.iter().all(|e| e.entity_label == "Ticket"));

    // Filter by action.
    let updates = repository::list_global_audit(
        &pool,
        50,
        0,
        &repository::GlobalAuditFilter {
            action: Some("update".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(updates.total, 1);
    assert_eq!(updates.items[0].doc_id, "d1");

    // Filter by entity_id.
    let by_entity = repository::list_global_audit(
        &pool,
        50,
        0,
        &repository::GlobalAuditFilter {
            entity_id: Some("ticket".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(by_entity.total, 3);
    let missing = repository::list_global_audit(
        &pool,
        50,
        0,
        &repository::GlobalAuditFilter {
            entity_id: Some("missing".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(missing.total, 0);

    // Search by doc_id.
    let searched = repository::list_global_audit(
        &pool,
        50,
        0,
        &repository::GlobalAuditFilter {
            search: Some("d1".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(searched.total, 2);

    // Pagination.
    let page = repository::list_global_audit(&pool, 2, 0, &Default::default())
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.items.len(), 2);
    let page2 = repository::list_global_audit(&pool, 2, 2, &Default::default())
        .await
        .unwrap();
    assert_eq!(page2.items.len(), 1);
}

#[tokio::test]
async fn list_documents_supports_search_filter_sort() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "d1",
        "ticket",
        &json!({"title": "Fix pump", "priority": 1}),
        None,
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "d2",
        "ticket",
        &json!({"title": "Replace belt", "priority": 2}),
        None,
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "d3",
        "ticket",
        &json!({"title": "Fix valve", "priority": 3}),
        None,
    )
    .await
    .unwrap();

    // search matches text fields and id
    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            search: Some("pump".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].id, "d1");

    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            search: Some("d2".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].id, "d2");

    // sort by priority asc
    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            sort_by: Some("priority".into()),
            sort_dir: Some("asc".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.items[0].id, "d1");
    assert_eq!(list.items[2].id, "d3");

    // sort by priority desc
    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            sort_by: Some("priority".into()),
            sort_dir: Some("desc".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.items[0].id, "d3");

    // pagination
    let list = repository::list_documents(&pool, "ticket", 2, 0, &Default::default())
        .await
        .unwrap();
    assert_eq!(list.total, 3);
    assert_eq!(list.items.len(), 2);
    let list = repository::list_documents(&pool, "ticket", 2, 2, &Default::default())
        .await
        .unwrap();
    assert_eq!(list.items.len(), 1);
}

#[tokio::test]
async fn list_documents_applies_view_config() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "d1",
        "ticket",
        &json!({"title": "Fix pump", "priority": 1}),
        None,
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "d2",
        "ticket",
        &json!({"title": "Replace belt", "priority": 2}),
        None,
    )
    .await
    .unwrap();

    let view =
        repository::create_entity_view(&pool, "ticket", "Pump only", &json!({"search": "pump"}))
            .await
            .unwrap();

    // View config filters the list.
    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            view_id: Some(view.id.clone()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].id, "d1");

    // Explicit params win over the view config.
    let list = repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            view_id: Some(view.id.clone()),
            search: Some("belt".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].id, "d2");

    // Unknown view rejected.
    assert!(repository::list_documents(
        &pool,
        "ticket",
        10,
        0,
        &repository::ListDocumentsFilter {
            view_id: Some("missing".into()),
            ..Default::default()
        },
    )
    .await
    .is_err());
}

#[tokio::test]
async fn audit_records_actor_per_user() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "d1",
        "ticket",
        &json!({"title": "Fix pump"}),
        Some("alice"),
    )
    .await
    .unwrap();
    repository::update_document(&pool, "d1", &json!({"title": "Fixed"}), Some("bob"), None)
        .await
        .unwrap();

    let audit = repository::list_document_audit(&pool, "d1", 10, 0)
        .await
        .unwrap();
    assert_eq!(audit.total, 2);
    assert_eq!(audit.items[0].action, "update");
    assert_eq!(audit.items[0].actor.as_deref(), Some("bob"));
    assert_eq!(audit.items[1].action, "create");
    assert_eq!(audit.items[1].actor.as_deref(), Some("alice"));

    let global = repository::list_global_audit(&pool, 10, 0, &Default::default())
        .await
        .unwrap();
    assert!(global.items.iter().all(|e| e.actor.is_some()));
}

#[tokio::test]
async fn report_crud_and_aggregation() {
    use logholizon_core::seed;
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    for (id, status) in [("r1", "draft"), ("r2", "draft"), ("r3", "open")] {
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

    // Aggregation groups by select field.
    let buckets = repository::report_aggregate_as_role(&pool, "work_order", "status", "admin")
        .await
        .unwrap();
    let by_status: std::collections::HashMap<&str, i64> = buckets
        .iter()
        .map(|b| (b.status.as_str(), b.count))
        .collect();
    assert_eq!(by_status.get("draft"), Some(&2));
    assert_eq!(by_status.get("open"), Some(&1));

    // Non-select field rejected.
    assert!(
        repository::report_aggregate_as_role(&pool, "work_order", "title", "admin")
            .await
            .is_err()
    );
    // Unknown field rejected.
    assert!(
        repository::report_aggregate_as_role(&pool, "work_order", "nope", "admin")
            .await
            .is_err()
    );

    // Report CRUD.
    let report = repository::create_report(
        &pool,
        "work_order",
        "By status",
        &json!({"group_by": "status", "chart_type": "bar"}),
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(report.name, "By status");
    assert_eq!(report.config["group_by"], "status");

    // Invalid config rejected.
    assert!(repository::create_report(
        &pool,
        "work_order",
        "Bad",
        &json!({"chart_type": "bar"}),
        None,
    )
    .await
    .is_err());
    assert!(repository::create_report(
        &pool,
        "work_order",
        "Bad chart",
        &json!({"group_by": "status", "chart_type": "3d"}),
        None,
    )
    .await
    .is_err());

    let reports = repository::list_reports(&pool, "work_order").await.unwrap();
    assert_eq!(reports.len(), 1);
    repository::delete_report(&pool, &report.id).await.unwrap();
    assert!(repository::delete_report(&pool, &report.id).await.is_err());
    let reports = repository::list_reports(&pool, "work_order").await.unwrap();
    assert!(reports.is_empty());
}

#[tokio::test]
async fn update_rejects_stale_expected_updated_at() {
    let pool = seeded_pool().await;
    repository::create_document(&pool, "d1", "ticket", &json!({"title": "Fix pump"}), None)
        .await
        .unwrap();
    let fresh = repository::get_document(&pool, "d1").await.unwrap();

    // Fresh precondition succeeds.
    let updated = repository::update_document(
        &pool,
        "d1",
        &json!({"title": "Fixed"}),
        Some("alice"),
        Some(&fresh.updated_at),
    )
    .await
    .unwrap();
    assert_eq!(updated.payload["title"], "Fixed");

    // Stale precondition conflicts.
    let err = repository::update_document(
        &pool,
        "d1",
        &json!({"title": "Stale"}),
        Some("bob"),
        Some(&fresh.updated_at),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("stale record"), "{err}");

    // Loser did not overwrite.
    let doc = repository::get_document(&pool, "d1").await.unwrap();
    assert_eq!(doc.payload["title"], "Fixed");
}
