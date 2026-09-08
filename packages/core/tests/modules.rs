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
                "views": [{"name": "All vehicles", "config": {}}]
            }
        ]
    })
}

#[tokio::test]
async fn module_registry_publish_rollback() {
    let pool = setup().await;
    let definition = vehicle_definition();

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

    let published = repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    assert_eq!(published.status, "published");
    assert_eq!(published.version, 2);

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
