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
    let title = repository::create_field(&pool, "work_order", "title", "text", true)
        .await
        .unwrap();
    assert_eq!(title.name, "title");
    assert!(title.required);
    assert_eq!(title.position, 0);

    let status = repository::create_field(&pool, "work_order", "status", "select", true)
        .await
        .unwrap();
    assert_eq!(status.position, 1);

    let fields = repository::list_fields(&pool, "work_order").await.unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "title");
    assert_eq!(fields[1].name, "status");

    // update field
    let updated = repository::update_field(&pool, &title.id, "title", "text", false)
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
async fn field_validation_rejects_bad_input() {
    let pool = setup().await;
    // invalid type
    assert!(
        repository::create_field(&pool, "work_order", "bad", "json", false)
            .await
            .is_err()
    );
    // invalid name (uppercase / spaces)
    assert!(
        repository::create_field(&pool, "work_order", "Bad Name", "text", false)
            .await
            .is_err()
    );
    // duplicate name
    repository::create_field(&pool, "work_order", "title", "text", false)
        .await
        .unwrap();
    assert!(
        repository::create_field(&pool, "work_order", "title", "text", false)
            .await
            .is_err()
    );
    // field on missing entity
    assert!(
        repository::create_field(&pool, "missing", "title", "text", false)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn option_crud_and_last_option_guard() {
    let pool = setup().await;
    let status = repository::create_field(&pool, "work_order", "status", "select", true)
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
    repository::create_field(&pool, "work_order", "title", "text", false)
        .await
        .unwrap();
    repository::create_document(
        &pool,
        "doc-1",
        "work_order",
        &serde_json::json!({ "title": "Fix pump" }),
    )
    .await
    .unwrap();
    assert!(repository::delete_entity(&pool, "work_order")
        .await
        .is_err());
}
