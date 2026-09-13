use logholizon_core::{db, repository};
use serde_json::json;

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

fn vehicle_definition() -> serde_json::Value {
    json!({
        "entities": [
            {
                "name": "driver",
                "label": "Driver",
                "fields": [
                    {"name": "code", "type": "text", "required": true},
                    {"name": "name", "type": "text", "required": true},
                    {"name": "status", "type": "select", "required": true, "is_status": true, "options": [
                        {"value": "active", "label": "Active"},
                        {"value": "retired", "label": "Retired"}
                    ]}
                ],
                "workflow": {
                    "states": [
                        {"name": "active", "label": "Active"},
                        {"name": "retired", "label": "Retired"}
                    ],
                    "transitions": [
                        {"from_state": "active", "to_state": "retired", "action": "retire"}
                    ]
                }
            },
            {
                "name": "vehicle",
                "label": "Vehicle",
                "fields": [
                    {"name": "code", "type": "text", "required": true},
                    {"name": "plate_number", "type": "text", "required": true},
                    {"name": "vehicle_type", "type": "select", "options": [
                        {"value": "truck", "label": "Truck"},
                        {"value": "van", "label": "Van"}
                    ]},
                    {"name": "status", "type": "select", "required": true, "is_status": true, "options": [
                        {"value": "active", "label": "Active"},
                        {"value": "maintenance", "label": "Maintenance"},
                        {"value": "retired", "label": "Retired"}
                    ]},
                    {"name": "driver", "type": "reference", "ref_entity": "driver"},
                    {"name": "summary", "type": "computed", "computed_expr": "{code} [{plate_number}]"}
                ],
                "workflow": {
                    "states": [
                        {"name": "active", "label": "Active"},
                        {"name": "maintenance", "label": "Maintenance"},
                        {"name": "retired", "label": "Retired"}
                    ],
                    "transitions": [
                        {"from_state": "active", "to_state": "maintenance", "action": "send_to_maintenance"},
                        {"from_state": "maintenance", "to_state": "active", "action": "return_to_service"},
                        {"from_state": "maintenance", "to_state": "retired", "action": "retire"}
                    ]
                },
                "views": [{"name": "All vehicles", "config": {}}],
                "form_layout": {"config": {"sections": [
                    {"id": "main", "label": "Main", "fields": ["code", "plate_number"]}
                ]}}
            }
        ]
    })
}

#[tokio::test]
async fn module_form_layout_names_translate_to_ids() {
    let pool = setup().await;
    // Unknown layout refs fail fast at draft time.
    let bad = json!({"entities": [{"name": "a", "label": "A",
        "fields": [{"name": "x", "type": "text"}],
        "form_layout": {"config": {"sections": [{"id": "s", "label": "S", "fields": ["missing"]}]}}} ]});
    assert!(
        repository::create_module(&pool, "badlayout", "BadLayout", None, None, None, "alice", &bad, None)
            .await
            .is_err()
    );

    let module = repository::create_module(
        &pool, "layout", "Layout", None, None, None, "alice",
        &vehicle_definition(), Some("alice"),
    )
    .await
    .unwrap();
    repository::submit_module_for_review(&pool, &module.id, "alice", "user").await.unwrap();
    repository::publish_module(&pool, &module.id, "alice", "user", Some("alice")).await.unwrap();
    let vehicle_entity = format!("{}_vehicle", module.id);
    let layout = repository::get_entity_form_layout(&pool, &vehicle_entity).await.unwrap();
    let sections = layout.config.get("sections").and_then(|s| s.as_array()).unwrap();
    assert_eq!(sections.len(), 1);
    let refs: Vec<&str> = sections[0].get("fields").and_then(|f| f.as_array()).unwrap()
        .iter().filter_map(|f| f.as_str()).collect();
    // Builder-authored names translate to materialized field IDs.
    assert_eq!(refs, vec![
        format!("{vehicle_entity}_code").as_str(),
        format!("{vehicle_entity}_plate_number").as_str(),
    ]);
}

#[tokio::test]
async fn module_registry_publish_rollback() {
    let pool = setup().await;
    let definition = vehicle_definition();

    // Drafts may start empty: the Module Builder creates the module shell
    // first and adds entities afterwards. Review still requires completeness.
    let empty = repository::create_module(
        &pool,
        "empty_shell",
        "Empty Shell",
        None,
        None,
        None,
        "alice",
        &json!({"entities": []}),
        Some("alice"),
    )
    .await
    .unwrap();
    assert_eq!(empty.status, "draft");
    assert!(
        repository::submit_module_for_review(&pool, &empty.id, "alice", "user")
            .await
            .is_err()
    );
    repository::update_module_draft(
        &pool,
        &empty.id,
        None,
        None,
        None,
        None,
        Some(&vehicle_definition()),
        "alice",
        "user",
    )
    .await
    .unwrap();
    repository::submit_module_for_review(&pool, &empty.id, "alice", "user")
        .await
        .unwrap();

    // Invalid: reference to unknown entity.
    let bad = json!({"entities": [{"name": "a", "label": "A", "fields": [{"name": "x", "type": "reference", "ref_entity": "missing"}]}]});
    assert!(
        repository::create_module(&pool, "bad", "Bad", None, None, None, "alice", &bad, None)
            .await
            .is_err()
    );

    // Invalid: self reference.
    let bad_self = json!({"entities": [{"name": "a", "label": "A", "fields": [{"name": "x", "type": "reference", "ref_entity": "a"}]}]});
    assert!(repository::create_module(
        &pool, "badself", "BadSelf", None, None, None, "alice", &bad_self, None
    )
    .await
    .is_err());

    let module = repository::create_module(
        &pool,
        "vehicle",
        "Vehicle Management",
        Some("Fleet"),
        Some("i-lucide-truck"),
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    assert_eq!(module.status, "draft");
    assert_eq!(module.owner, "alice");

    // Tenant isolation: bob cannot read alice's module.
    assert!(repository::get_module(&pool, &module.id, "bob", "user")
        .await
        .is_err());
    // Admin bypasses owner check.
    repository::get_module(&pool, &module.id, "bob", "admin")
        .await
        .unwrap();

    // Draft edits do not touch live entities until publish.
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
        .bind(format!("{}_vehicle", module.id))
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists);

    repository::submit_module_for_review(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    let published = repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    assert_eq!(published.status, "published");
    assert_eq!(published.version, 2);
    assert_eq!(published.semantic_version, "1.0.1");
    let changes = repository::list_module_changes(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0].get("change_type").and_then(|v| v.as_str()),
        Some("publish")
    );

    // Materialized entities are usable through the generic document runtime.
    let vehicle_entity = format!("{}_vehicle", module.id);
    let driver_entity = format!("{}_driver", module.id);
    repository::create_document(
        &pool,
        "drv-1",
        &driver_entity,
        &json!({"code": "D1", "name": "Ann", "status": "active"}),
        Some("alice"),
    )
    .await
    .unwrap();
    repository::create_document(&pool, "veh-1", &vehicle_entity, &json!({"code": "V1", "plate_number": "AB-123", "vehicle_type": "truck", "status": "active", "driver": "drv-1"}), Some("alice")).await.map_err(|e| panic!("veh-1 create failed: {e:#}")).unwrap();
    // Bad reference rejected.
    assert!(repository::create_document(&pool, "veh-bad", &vehicle_entity, &json!({"code": "V9", "plate_number": "XX", "vehicle_type": "truck", "status": "active", "driver": "missing"}), None).await.is_err());
    // Computed field interpolates on read.
    let doc = repository::get_document(&pool, "veh-1").await.unwrap();
    assert_eq!(
        doc.payload
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "V1 [AB-123]"
    );
    // Workflow transition follows the published definition.
    let transitioned = repository::transition_document_as_role(
        &pool,
        "veh-1",
        "send_to_maintenance",
        Some("alice"),
        None,
        "admin",
    )
    .await
    .unwrap();
    assert_eq!(
        transitioned
            .payload
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "maintenance"
    );

    // Compatibility: removing a field while records exist is blocked on next publish.
    let mut next_definition = definition.clone();
    next_definition["entities"][1]["fields"] = json!([
        {"name": "code", "type": "text", "required": true},
        {"name": "status", "type": "select", "required": true, "is_status": true, "options": [
            {"value": "active", "label": "Active"},
            {"value": "maintenance", "label": "Maintenance"},
            {"value": "retired", "label": "Retired"}
        ]}
    ]);
    // The layout references the removed field too; drop it so the draft
    // stays internally consistent (stale refs fail fast by design).
    next_definition["entities"][1]["form_layout"] = json!({"config": {"sections": [
        {"id": "main", "label": "Main", "fields": ["code"]}
    ]}});
    // Publish requires draft status: archive then restore to draft first.
    repository::archive_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    repository::restore_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    repository::update_module_draft(
        &pool,
        &module.id,
        None,
        None,
        None,
        None,
        Some(&next_definition),
        "alice",
        "user",
    )
    .await
    .unwrap();
    assert!(
        repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
            .await
            .is_err()
    );

    // Rollback to version 2 republishes the full definition.
    let rolled_back =
        repository::rollback_module(&pool, &module.id, 2, "alice", "user", Some("alice"))
            .await
            .unwrap();
    assert_eq!(rolled_back.status, "published");
    let versions = repository::list_module_versions(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert!(versions.len() >= 2);
    assert!(versions.iter().any(|v| v.semantic_version == "1.0.2"));
    let changes = repository::list_module_changes(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert!(changes
        .iter()
        .any(|v| v.get("change_type").and_then(|x| x.as_str()) == Some("rollback")));

    // Automations CRUD.
    let automation = repository::create_automation(
        &pool,
        &vehicle_entity,
        "transition",
        "webhook",
        "https://example.test/hook",
        true,
    )
    .await
    .unwrap();
    assert_eq!(automation.trigger, "transition");
    let listed = repository::list_automations(&pool, &vehicle_entity)
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    repository::delete_automation(&pool, &automation.id)
        .await
        .unwrap();
    assert!(repository::list_automations(&pool, &vehicle_entity)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn business_rules_validate() {
    let pool = setup().await;
    repository::create_entity(&pool, "rules_widget", "rules_widget", "Rules Widget")
        .await
        .unwrap();
    repository::create_field(
        &pool,
        "rules_widget",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    repository::create_field_with_rules(
        &pool,
        "rules_widget",
        "sku",
        "text",
        true,
        false,
        None,
        None,
        &repository::FieldRules {
            is_unique: true,
            pattern: Some("^SKU-.*".to_string()),
            min_length: Some(5),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    repository::create_field_with_rules(
        &pool,
        "rules_widget",
        "price",
        "number",
        false,
        false,
        None,
        None,
        &repository::FieldRules {
            min_value: Some(0.0),
            max_value: Some(100.0),
            default_value: Some("9".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    repository::create_field_with_rules(
        &pool,
        "rules_widget",
        "ticket",
        "text",
        false,
        false,
        None,
        None,
        &repository::FieldRules {
            auto_number_prefix: Some("T-".to_string()),
            auto_number_width: Some(4),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Defaults + auto-number apply on create.
    let created = repository::create_document(
        &pool,
        "rules-1",
        "rules_widget",
        &json!({"title": "Widget", "sku": "SKU-001"}),
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        created.payload.get("price").and_then(|v| v.as_f64()),
        Some(9.0)
    );
    assert_eq!(
        created.payload.get("ticket").and_then(|v| v.as_str()),
        Some("T-0001")
    );
    let created_two = repository::create_document(
        &pool,
        "rules-2",
        "rules_widget",
        &json!({"title": "Widget 2", "sku": "SKU-002"}),
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        created_two.payload.get("ticket").and_then(|v| v.as_str()),
        Some("T-0002")
    );

    // Unique + pattern + range reject bad payloads.
    assert!(repository::create_document(
        &pool,
        "rules-dup",
        "rules_widget",
        &json!({"title": "Dup", "sku": "SKU-001"}),
        None,
    )
    .await
    .is_err());
    assert!(repository::create_document(
        &pool,
        "rules-bad-pattern",
        "rules_widget",
        &json!({"title": "Bad", "sku": "BAD-1"}),
        None,
    )
    .await
    .is_err());
    assert!(repository::create_document(
        &pool,
        "rules-bad-range",
        "rules_widget",
        &json!({"title": "Bad", "sku": "SKU-009", "price": 101.0}),
        None,
    )
    .await
    .is_err());

    // Unique excludes self on update.
    repository::update_document(&pool, "rules-1", &json!({"price": 10.0}), None, None)
        .await
        .unwrap();
    assert!(
        repository::update_document(&pool, "rules-1", &json!({"sku": "SKU-002"}), None, None)
            .await
            .is_err()
    );

    // Invalid rule shapes are rejected at field creation.
    assert!(repository::create_field_with_rules(
        &pool,
        "rules_widget",
        "bad_range",
        "number",
        false,
        false,
        None,
        None,
        &repository::FieldRules {
            min_value: Some(10.0),
            max_value: Some(1.0),
            ..Default::default()
        },
    )
    .await
    .is_err());
}

#[tokio::test]
async fn action_transactional_audit_event() {
    let pool = setup().await;
    let definition = vehicle_definition();
    let module = repository::create_module(
        &pool,
        "vehicle",
        "Vehicle Management",
        None,
        None,
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    repository::submit_module_for_review(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    let vehicle_entity = format!("{}_vehicle", module.id);
    let driver_entity = format!("{}_driver", module.id);
    repository::create_document(
        &pool,
        "drv-action",
        &driver_entity,
        &json!({"code": "D9", "name": "Bo", "status": "active"}),
        Some("alice"),
    )
    .await
    .unwrap();

    let create_action = repository::create_module_action(
        &pool,
        &vehicle_entity,
        "create",
        "Create",
        "create",
        &json!({}),
    )
    .await
    .unwrap();

    // Generic create action writes _doc + audit.
    let created = repository::execute_module_action(
        &pool,
        &vehicle_entity,
        &create_action.id,
        Some("veh-action-1"),
        Some(&json!({"code": "V9", "plate_number": "ZZ-9", "vehicle_type": "van", "status": "active", "driver": "drv-action"})),
        Some("alice"),
        None,
        "admin",
    )
    .await
    .unwrap();
    assert_eq!(created.document.as_ref().unwrap().id, "veh-action-1");

    let close_action = repository::create_module_action(
        &pool,
        &vehicle_entity,
        "close",
        "Close",
        "update",
        &json!({}),
    )
    .await
    .unwrap();

    // Custom close action stamps audit without ERP tables.
    let closed = repository::execute_module_action(
        &pool,
        &vehicle_entity,
        &close_action.id,
        Some("veh-action-1"),
        Some(&json!({"code": "V9"})),
        Some("alice"),
        None,
        "admin",
    )
    .await
    .unwrap();
    assert!(closed
        .document
        .as_ref()
        .unwrap()
        .payload
        .get("code")
        .is_some());

    let events: Vec<String> =
        sqlx::query_scalar("SELECT event_type FROM _event WHERE entity_id = ? ORDER BY rowid")
            .bind(&vehicle_entity)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(events.iter().any(|e| e == "record.created"), "{events:?}");
    assert!(events.iter().any(|e| e == "action.executed"), "{events:?}");

    // Unknown actions are rejected.
    assert!(repository::execute_module_action(
        &pool,
        &vehicle_entity,
        "nope",
        Some("veh-action-1"),
        None,
        Some("alice"),
        None,
        "admin",
    )
    .await
    .is_err());

    // Audit covers the action trail (query raw rows to avoid redaction parse).
    let actions: Vec<String> =
        sqlx::query_scalar("SELECT action FROM _audit_log WHERE doc_id = ? ORDER BY rowid")
            .bind("veh-action-1")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(actions.contains(&"create".to_string()), "{actions:?}");
    assert!(actions.contains(&"close".to_string()), "{actions:?}");
}

#[tokio::test]
async fn automation_triggers() {
    let pool = setup().await;
    repository::create_entity(&pool, "auto_widget", "auto_widget", "Auto Widget")
        .await
        .unwrap();
    repository::create_field(
        &pool,
        "auto_widget",
        "title",
        "text",
        true,
        false,
        None,
        None,
    )
    .await
    .unwrap();
    for trigger in ["create", "update", "delete"] {
        repository::create_automation(
            &pool,
            "auto_widget",
            trigger,
            "webhook",
            "https://example.test/hook",
            true,
        )
        .await
        .unwrap();
    }
    repository::create_document(
        &pool,
        "auto-1",
        "auto_widget",
        &json!({"title": "Hi"}),
        None,
    )
    .await
    .unwrap();
    repository::update_document(&pool, "auto-1", &json!({"title": "Yo"}), None, None)
        .await
        .unwrap();
    repository::delete_document(&pool, "auto-1", None)
        .await
        .unwrap();
    let deliveries = repository::list_notification_deliveries(&pool, 10, 0)
        .await
        .unwrap();
    let actions: Vec<String> = deliveries.items.iter().map(|d| d.action.clone()).collect();
    assert!(actions.contains(&"create".to_string()));
    assert!(actions.contains(&"update".to_string()));
    assert!(actions.contains(&"delete".to_string()));
}

#[tokio::test]
async fn module_lifecycle_requires_ordered_transitions() {
    let pool = setup().await;
    let module = repository::create_module(
        &pool,
        "lifecycle",
        "Lifecycle",
        None,
        None,
        None,
        "alice",
        &vehicle_definition(),
        Some("alice"),
    )
    .await
    .unwrap();
    assert_eq!(module.status, "draft");

    repository::submit_module_for_review(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    let module = repository::get_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(module.status, "review");
    assert!(
        repository::enable_module(&pool, &module.id, "alice", "user")
            .await
            .is_err()
    );

    let module = repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    assert_eq!(module.status, "published");
    let module = repository::enable_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(module.status, "enabled");
    let module = repository::disable_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(module.status, "disabled");
    let module = repository::archive_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(module.status, "archived");
    let module = repository::restore_module(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    assert_eq!(module.status, "draft");
}

#[tokio::test]
async fn module_package_export_preview_install_uninstall() {
    let pool = setup().await;
    let definition = vehicle_definition();
    let module = repository::create_module(
        &pool,
        "vehicle",
        "Vehicle",
        None,
        None,
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    let package = logholizon_core::module_package::export_package(&module).unwrap();
    assert_eq!(package["kind"], "logholizon.module");
    assert_eq!(package["manifest"]["version"], "1.0.0");

    let preview = logholizon_core::module_package::preview_package(&pool, &package, "bob")
        .await
        .unwrap();
    assert!(preview.valid);
    assert_eq!(preview.action, "install");
    assert!(!preview.migration.creates.is_empty());

    let installed =
        logholizon_core::module_package::install_package(&pool, &package, "bob", Some("bob"))
            .await
            .unwrap();
    assert_eq!(installed.status, "enabled");
    assert_eq!(installed.name, "vehicle");

    let entity_id = format!("{}_vehicle", installed.id);
    let entity_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
            .bind(&entity_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(entity_exists);

    let disabled = repository::disable_module(&pool, &installed.id, "bob", "user")
        .await
        .unwrap();
    assert_eq!(disabled.status, "disabled");
    let uninstalled =
        logholizon_core::module_package::uninstall_package(&pool, &installed.id, "bob", "user")
            .await
            .unwrap();
    assert_eq!(uninstalled.status, "archived");
}
