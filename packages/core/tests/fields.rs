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
    let title = repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(title.name, "title");
    assert!(title.required);
    assert!(!title.is_status);
    assert_eq!(title.position, 0);

    let status = repository::create_field(
        &pool,
        "work_order",
        "status",
        "select",
        true,
        true,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(status.position, 1);
    assert!(status.is_status);

    let fields = repository::list_fields(&pool, "work_order").await.unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "title");
    assert_eq!(fields[1].name, "status");

    // update field
    let updated =
        repository::update_field(&pool, &title.id, "title", "text", false, false, None, None)
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
async fn field_permissions_crud_and_check() {
    let pool = setup().await;
    let title = repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    let priority = repository::create_field(
        &pool,
        "work_order",
        "priority",
        "text",
        false,
        false,
        None,
        None,
    )
    .await
    .unwrap();

    // Default: both roles can view and edit every field.
    let perms = repository::get_field_permissions(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(perms.len(), 4);
    repository::check_field_permission(&pool, "work_order", "title", "user", false)
        .await
        .unwrap();
    repository::check_field_permission(&pool, "work_order", "title", "user", true)
        .await
        .unwrap();

    // Hide priority from users.
    repository::update_field_permissions(
        &pool,
        "work_order",
        &[(priority.id.clone(), "user".to_string(), false, false)],
    )
    .await
    .unwrap();
    let err = repository::check_field_permission(&pool, "work_order", "priority", "user", false)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no view access"));

    // Make title view-only for users.
    repository::update_field_permissions(
        &pool,
        "work_order",
        &[(title.id.clone(), "user".to_string(), true, false)],
    )
    .await
    .unwrap();
    repository::check_field_permission(&pool, "work_order", "title", "user", false)
        .await
        .unwrap();
    let err = repository::check_field_permission(&pool, "work_order", "title", "user", true)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no edit access"));

    // Admin bypasses the matrix.
    repository::check_field_permission(&pool, "work_order", "priority", "admin", true)
        .await
        .unwrap();

    // Invalid role rejected.
    assert!(repository::update_field_permissions(
        &pool,
        "work_order",
        &[(title.id.clone(), "superuser".to_string(), true, true)],
    )
    .await
    .is_err());

    // Field from another entity rejected.
    repository::create_entity(&pool, "other", "other", "Other")
        .await
        .unwrap();
    let other_field =
        repository::create_field(&pool, "other", "note", "text", false, false, None, None)
            .await
            .unwrap();
    assert!(repository::update_field_permissions(
        &pool,
        "work_order",
        &[(other_field.id.clone(), "user".to_string(), false, false)],
    )
    .await
    .is_err());

    // Missing entity rejected.
    assert!(repository::get_field_permissions(&pool, "missing")
        .await
        .is_err());
}

#[tokio::test]
async fn field_permissions_enforce_on_documents() {
    use serde_json::json;
    let pool = setup().await;
    let title = repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    let secret = repository::create_field(
        &pool,
        "work_order",
        "secret",
        "text",
        false,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    let note = repository::create_field(
        &pool,
        "work_order",
        "note",
        "text",
        false,
        false,
        None,
        None,
    )
    .await
    .unwrap();

    // secret hidden, note view-only for users.
    repository::update_field_permissions(
        &pool,
        "work_order",
        &[
            (secret.id.clone(), "user".to_string(), false, false),
            (note.id.clone(), "user".to_string(), true, false),
        ],
    )
    .await
    .unwrap();

    // Create as user: hidden field dropped, view-only field kept.
    let doc = repository::create_document_as_role(
        &pool,
        "d1",
        "work_order",
        &json!({"title": "Fix pump", "secret": "s3cr3t", "note": "hello"}),
        Some("alice"),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(doc.payload["title"], "Fix pump");
    assert!(doc.payload.get("secret").is_none());
    assert_eq!(doc.payload["note"], "hello");

    // List as user: hidden field stripped.
    let list =
        repository::list_documents_as_role(&pool, "work_order", 10, 0, &Default::default(), "user")
            .await
            .unwrap();
    assert_eq!(list.total, 1);
    assert!(list.items[0].payload.get("secret").is_none());
    assert_eq!(list.items[0].payload["note"], "hello");

    // Update as user: non-editable fields preserve existing values.
    let updated = repository::update_document_as_role(
        &pool,
        "d1",
        &json!({"title": "Fixed", "secret": "hacked", "note": "changed"}),
        Some("alice"),
        None,
        "user",
    )
    .await
    .unwrap();
    assert_eq!(updated.payload["title"], "Fixed");
    assert!(updated.payload.get("secret").is_none());
    assert_eq!(updated.payload["note"], "hello");

    // Export as user excludes hidden fields.
    let csv = repository::export_documents_csv_as_role(&pool, "work_order", "user")
        .await
        .unwrap();
    let header = csv.lines().next().unwrap();
    assert!(header.contains("title"), "{header}");
    assert!(!header.contains("secret"), "{header}");

    // Audit as user redacts hidden fields.
    let audit = repository::list_document_audit_as_role(&pool, "d1", 10, 0, "user")
        .await
        .unwrap();
    assert!(audit
        .items
        .iter()
        .all(|e| e.payload.get("secret").is_none()));

    // Admin still sees everything.
    let admin_list = repository::list_documents_as_role(
        &pool,
        "work_order",
        10,
        0,
        &Default::default(),
        "admin",
    )
    .await
    .unwrap();
    assert_eq!(admin_list.items[0].payload["note"], "hello");

    // Hidden required field is not required for that role.
    repository::update_field_permissions(
        &pool,
        "work_order",
        &[(title.id.clone(), "user".to_string(), false, false)],
    )
    .await
    .unwrap();
    repository::create_document_as_role(
        &pool,
        "d2",
        "work_order",
        &json!({"note": "no title"}),
        Some("alice"),
        "user",
    )
    .await
    .unwrap();
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
async fn form_layout_crud_and_validation() {
    let pool = setup().await;
    let title = repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    let note = repository::create_field(
        &pool,
        "work_order",
        "note",
        "text",
        false,
        false,
        None,
        None,
    )
    .await
    .unwrap();

    // Default: empty config when no layout saved yet.
    let layout = repository::get_entity_form_layout(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(layout.entity_id, "work_order");
    assert_eq!(layout.config, serde_json::json!({}));

    // Valid layout round-trips.
    let config = serde_json::json!({"sections": [
        {"id": "main", "label": "Main", "fields": [title.id]},
        {"id": "extra", "label": "Extra", "fields": [note.id]},
    ]});
    let saved = repository::update_entity_form_layout(&pool, "work_order", &config)
        .await
        .unwrap();
    assert_eq!(saved.config, config);
    let fetched = repository::get_entity_form_layout(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(fetched.config, config);

    // Unknown field rejected.
    let err = repository::update_entity_form_layout(
        &pool,
        "work_order",
        &serde_json::json!({"sections": [{"id": "main", "label": "Main", "fields": ["nope"]}]}),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("unknown field"));

    // Duplicate field across sections rejected.
    assert!(repository::update_entity_form_layout(
        &pool,
        "work_order",
        &serde_json::json!({"sections": [
            {"id": "a", "label": "A", "fields": [title.id]},
            {"id": "b", "label": "B", "fields": [title.id]},
        ]}),
    )
    .await
    .is_err());

    // Missing sections array rejected.
    assert!(
        repository::update_entity_form_layout(&pool, "work_order", &serde_json::json!({}))
            .await
            .is_err()
    );

    // Missing entity rejected.
    assert!(repository::get_entity_form_layout(&pool, "missing")
        .await
        .is_err());
    assert!(
        repository::update_entity_form_layout(&pool, "missing", &config)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn field_validation_rejects_bad_input() {
    let pool = setup().await;
    // invalid type
    assert!(
        repository::create_field(&pool, "work_order", "bad", "json", false, false, None, None)
            .await
            .is_err()
    );
    // invalid name (uppercase / spaces)
    assert!(repository::create_field(
        &pool,
        "work_order",
        "Bad Name",
        "text",
        false,
        false,
        None,
        None
    )
    .await
    .is_err());
    // duplicate name
    repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        false,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    assert!(repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        false,
        false,
        None,
        None
    )
    .await
    .is_err());
    // field on missing entity
    assert!(
        repository::create_field(&pool, "missing", "title", "text", false, false, None, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn status_field_validation() {
    let pool = setup().await;
    // is_status requires type select
    let err = repository::create_field(
        &pool,
        "work_order",
        "state",
        "text",
        false,
        true,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("status field must be of type select"));

    // first status field is allowed
    let status = repository::create_field(
        &pool,
        "work_order",
        "status",
        "select",
        true,
        true,
        None,
        None,
    )
    .await
    .unwrap();
    assert!(status.is_status);

    // second status field in the same entity is rejected
    let err = repository::create_field(
        &pool,
        "work_order",
        "state",
        "select",
        false,
        true,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("already has a status field"));

    // updating the existing status field to is_status=false is allowed
    let updated = repository::update_field(
        &pool, &status.id, "status", "select", true, false, None, None,
    )
    .await
    .unwrap();
    assert!(!updated.is_status);

    // now a different field can become the status field
    let state = repository::create_field(
        &pool,
        "work_order",
        "state",
        "select",
        false,
        true,
        None,
        None,
    )
    .await
    .unwrap();
    assert!(state.is_status);
}

#[tokio::test]
async fn option_crud_and_last_option_guard() {
    let pool = setup().await;
    let status = repository::create_field(
        &pool,
        "work_order",
        "status",
        "select",
        true,
        true,
        None,
        None,
    )
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
async fn reference_field_validation() {
    use serde_json::json;
    let pool = setup().await;
    repository::create_entity(&pool, "product", "product", "Product")
        .await
        .unwrap();
    repository::create_field(&pool, "product", "title", "text", true, false, None, None)
        .await
        .unwrap();
    repository::create_document(&pool, "p1", "product", &json!({"title": "Pump"}), None)
        .await
        .unwrap();

    // Reference without target rejected.
    assert!(repository::create_field(
        &pool,
        "work_order",
        "part",
        "reference",
        false,
        false,
        None,
        None
    )
    .await
    .is_err());
    // Reference to missing entity rejected.
    assert!(repository::create_field(
        &pool,
        "work_order",
        "part",
        "reference",
        false,
        false,
        Some("nope"),
        None
    )
    .await
    .is_err());
    // Reference to self rejected.
    assert!(repository::create_field(
        &pool,
        "work_order",
        "part",
        "reference",
        false,
        false,
        Some("work_order"),
        None
    )
    .await
    .is_err());
    // ref_entity on non-reference type rejected.
    assert!(repository::create_field(
        &pool,
        "work_order",
        "note",
        "text",
        false,
        false,
        Some("product"),
        None
    )
    .await
    .is_err());

    let part = repository::create_field(
        &pool,
        "work_order",
        "part",
        "reference",
        false,
        false,
        Some("product"),
        None,
    )
    .await
    .unwrap();
    assert_eq!(part.ref_entity.as_deref(), Some("product"));

    // Payload referencing a missing doc rejected.
    assert!(repository::create_document(
        &pool,
        "w1",
        "work_order",
        &json!({"part": "missing"}),
        None
    )
    .await
    .is_err());
    // Payload referencing an existing doc accepted.
    repository::create_document(&pool, "w1", "work_order", &json!({"part": "p1"}), None)
        .await
        .unwrap();

    // Options endpoint resolves labels.
    let options = repository::list_entity_options(&pool, "product", "admin")
        .await
        .unwrap();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].id, "p1");
    assert_eq!(options[0].label, "Pump");
}

#[tokio::test]
async fn new_field_types_validate() {
    use serde_json::json;
    let pool = setup().await;
    for (name, field_type) in [
        ("done", "checkbox"),
        ("notes", "textarea"),
        ("price", "currency"),
    ] {
        let field = repository::create_field(
            &pool,
            "work_order",
            name,
            field_type,
            false,
            false,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(field.r#type, field_type);
    }
    repository::create_document(
        &pool,
        "w1",
        "work_order",
        &json!({"done": true, "notes": "hello", "price": 9.5}),
        None,
    )
    .await
    .unwrap();
    // Wrong JSON types rejected.
    assert!(
        repository::create_document(&pool, "w2", "work_order", &json!({"done": "yes"}), None)
            .await
            .is_err()
    );
    assert!(repository::create_document(
        &pool,
        "w3",
        "work_order",
        &json!({"price": "free"}),
        None
    )
    .await
    .is_err());
}

#[tokio::test]
async fn entity_module_roundtrip() {
    let pool = setup().await;
    let entities = repository::list_entities(&pool).await.unwrap();
    assert!(entities.iter().all(|e| e.module.is_none()));
    let updated = repository::update_entity(
        &pool,
        "work_order",
        "work_order",
        "Work Order",
        Some("Stock"),
    )
    .await
    .unwrap();
    assert_eq!(updated.module.as_deref(), Some("Stock"));
    let entities = repository::list_entities(&pool).await.unwrap();
    let work_order = entities.iter().find(|e| e.id == "work_order").unwrap();
    assert_eq!(work_order.module.as_deref(), Some("Stock"));
    let detail = repository::get_entity_detail(&pool, "work_order")
        .await
        .unwrap();
    assert_eq!(detail.module.as_deref(), Some("Stock"));
}

#[tokio::test]
async fn entity_update_and_delete_guards() {
    let pool = setup().await;
    let updated = repository::update_entity(&pool, "work_order", "work_order", "Work Orders", None)
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
    repository::create_field(
        &pool,
        "work_order",
        "title",
        "text",
        false,
        false,
        None,
        None,
    )
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
