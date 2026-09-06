use logholizon_core::{db, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "work_order", "work_order", "Work Order")
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn field_crud_orders_by_position() {
    let pool = setup().await;
    let title = repository::create_field(&pool, "work_order", "title", "text", true, false)
        .await
        .unwrap();
    assert_eq!(title.name, "title");
    assert!(title.required);
    assert!(!title.is_status);
    assert_eq!(title.position, 0);

    let status = repository::create_field(&pool, "work_order", "status", "select", true, true)
        .await
        .unwrap();
    assert_eq!(status.position, 1);
    assert!(status.is_status);

    let fields = repository::list_fields(&pool, "work_order").await.unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "title");
    assert_eq!(fields[1].name, "status");

    // update field
    let updated = repository::update_field(&pool, &title.id, "title", "text", false, false)
        .await
        .unwrap();
    assert!(!updated.required);

    // delete field cascades options
    let option = repository::create_field_option(&pool, &status.id, "open", "Open")
        .await
        .unwrap();
    repository::delete_field(&pool, &status.id).await.unwrap();
    let fields = repository::list_fields(&pool, "work_order").await.unwrap();
    assert_eq!(fields.len(), 1);
    assert!(repository::get_field_option(&pool, &option.id)
        .await
        .is_err());
}

#[tokio::test]
async fn permissions_crud_and_check() {
    let pool = setup().await;

    // Default: both roles allowed.
    let perms = repository::get_entity_permissions(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(perms.len(), 2);
    repository::check_permission(&pool, "work_order", "user", false)
        .await
        .unwrap();
    repository::check_permission(&pool, "work_order", "user", true)
        .await
        .unwrap();

    // Revoke user edit.
    repository::update_entity_permissions(
        &pool,
        "work_order",
        &[
            ("admin".to_string(), true, true),
            ("user".to_string(), true, false),
        ],
    )
    .await
    .unwrap();
    repository::check_permission(&pool, "work_order", "user", false)
        .await
        .unwrap();
    let err = repository::check_permission(&pool, "work_order", "user", true)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no edit access"));

    // Revoke user view entirely.
    repository::update_entity_permissions(
        &pool,
        "work_order",
        &[
            ("admin".to_string(), true, true),
            ("user".to_string(), false, false),
        ],
    )
    .await
    .unwrap();
    let err = repository::check_permission(&pool, "work_order", "user", false)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no view access"));

    // Invalid role rejected.
    assert!(repository::update_entity_permissions(
        &pool,
        "work_order",
        &[("superuser".to_string(), true, true)],
    )
    .await
    .is_err());

    // Missing entity rejected.
    assert!(repository::get_entity_permissions(&pool, "missing")
        .await
        .is_err());
}

#[tokio::test]
async fn views_crud() {
    let pool = setup().await;

    let view = repository::create_entity_view(
        &pool,
        "work_order",
        "Open only",
        &serde_json::json!({"status": "open"}),
    )
    .await
    .unwrap();
    assert_eq!(view.name, "Open only");
    assert_eq!(view.config["status"], "open");

    // Duplicate name rejected.
    assert!(repository::create_entity_view(
        &pool,
        "work_order",
        "Open only",
        &serde_json::json!({}),
    )
    .await
    .is_err());

    let views = repository::list_entity_views(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(views.len(), 1);

    repository::delete_entity_view(&pool, &view.id)
        .await
        .unwrap();
    assert!(repository::delete_entity_view(&pool, &view.id)
        .await
        .is_err());
    let views = repository::list_entity_views(&pool, "work_order")
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn field_validation_rejects_bad_input() {
    let pool = setup().await;
    // invalid type
    assert!(
        repository::create_field(&pool, "work_order", "bad", "json", false, false)
            .await
            .is_err()
    );
    // invalid name (uppercase / spaces)
    assert!(
        repository::create_field(&pool, "work_order", "Bad Name", "text", false, false)
            .await
            .is_err()
    );
    // duplicate name
    repository::create_field(&pool, "work_order", "title", "text", false, false)
        .await
        .unwrap();
    assert!(
        repository::create_field(&pool, "work_order", "title", "text", false, false)
            .await
            .is_err()
    );
    // field on missing entity
    assert!(
        repository::create_field(&pool, "missing", "title", "text", false, false)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn status_field_validation() {
    let pool = setup().await;
    // is_status requires type select
    let err = repository::create_field(&pool, "work_order", "state", "text", false, true)
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("status field must be of type select"));

    // first status field is allowed
    let status = repository::create_field(&pool, "work_order", "status", "select", true, true)
        .await
        .unwrap();
    assert!(status.is_status);

    // second status field in the same entity is rejected
    let err = repository::create_field(&pool, "work_order", "state", "select", false, true)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("already has a status field"));

    // updating the existing status field to is_status=false is allowed
    let updated = repository::update_field(&pool, &status.id, "status", "select", true, false)
        .await
        .unwrap();
    assert!(!updated.is_status);

    // now a different field can become the status field
    let state = repository::create_field(&pool, "work_order", "state", "select", false, true)
        .await
        .unwrap();
    assert!(state.is_status);
}

#[tokio::test]
async fn option_crud_and_last_option_guard() {
    let pool = setup().await;
    let status = repository::create_field(&pool, "work_order", "status", "select", true, true)
        .await
        .unwrap();
    let open = repository::create_field_option(&pool, &status.id, "open", "Open")
        .await
        .unwrap();
    let done = repository::create_field_option(&pool, &status.id, "done", "Done")
        .await
        .unwrap();

    // update option
    let updated = repository::update_field_option(&pool, &open.id, "open", "Opened")
        .await
        .unwrap();
    assert_eq!(updated.label, "Opened");

    // cannot delete last option of a select field
    repository::delete_field_option(&pool, &open.id)
        .await
        .unwrap();
    assert!(repository::delete_field_option(&pool, &done.id)
        .await
        .is_err());

    // option on missing field
    assert!(repository::create_field_option(&pool, "missing", "x", "X")
        .await
        .is_err());
}

#[tokio::test]
async fn entity_update_and_delete_guards() {
    let pool = setup().await;
    let updated = repository::update_entity(&pool, "work_order", "work_order", "Work Orders")
        .await
        .unwrap();
    assert_eq!(updated.label, "Work Orders");

    // delete entity with no documents works
    repository::delete_entity(&pool, "work_order")
        .await
        .unwrap();
    assert!(repository::get_entity_detail(&pool, "work_order")
        .await
        .is_err());

    // delete entity with documents is rejected
    let pool = setup().await;
    repository::create_field(&pool, "work_order", "title", "text", false, false)
        .await
        .unwrap();
    repository::create_document(
        &pool,
        "doc-1",
        "work_order",
        &serde_json::json!({ "title": "Fix pump" }),
        None,
    )
    .await
    .unwrap();
    assert!(repository::delete_entity(&pool, "work_order")
        .await
        .is_err());
}
