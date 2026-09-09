use logholizon_core::{automation, dashboard, db, relation, repository};
use serde_json::json;

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

fn vehicle_definition() -> serde_json::Value {
    json!({
        "entities": [
            {"name":"vehicle","label":"Vehicle","fields":[
                {"name":"code","type":"text","required":true},
                {"name":"plate_number","type":"text","required":true},
                {"name":"status","type":"select","required":true,"is_status":true,
                 "options":[{"value":"available","label":"Available"},{"value":"rented","label":"Rented"},{"value":"maintenance","label":"Maintenance"}]}
            ]},
            {"name":"driver","label":"Driver","fields":[
                {"name":"code","type":"text","required":true},
                {"name":"name","type":"text","required":true}
            ]},
            {"name":"maintenance","label":"Maintenance","fields":[
                {"name":"vehicle","type":"reference","ref_entity":"vehicle","required":true},
                {"name":"cost","type":"number","required":true},
                {"name":"completed_at","type":"date"}
            ]},
            {"name":"rental","label":"Rental","fields":[
                {"name":"vehicle","type":"reference","ref_entity":"vehicle","required":true},
                {"name":"driver","type":"reference","ref_entity":"driver","required":true},
                {"name":"status","type":"select","required":true,"is_status":true,
                 "options":[{"value":"available","label":"Available"},{"value":"rented","label":"Rented"},{"value":"returned","label":"Returned"}]},
                {"name":"revenue","type":"number","required":true},
                {"name":"rented_at","type":"date"}
            ],"workflow":{"states":[
                {"name":"available","label":"Available"},{"name":"rented","label":"Rented"},{"name":"returned","label":"Returned"}
            ],"transitions":[
                {"from_state":"available","to_state":"rented","action":"rent"},
                {"from_state":"rented","to_state":"returned","action":"return"}
            ]}}
        ]
    })
}

#[tokio::test]
async fn phase20_vehicle_and_accounting_like_acceptance() {
    let pool = setup().await;
    let definition = vehicle_definition();
    let module = repository::create_module(
        &pool,
        "vehicle_management",
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
    assert_eq!(definition["entities"].as_array().unwrap().len(), 4);
    repository::submit_module_for_review(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    let published = repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    assert_eq!(published.status, "published");

    let vehicle = format!("{}_vehicle", module.id);
    let driver = format!("{}_driver", module.id);
    let maintenance = format!("{}_maintenance", module.id);
    let rental = format!("{}_rental", module.id);
    for entity in [&vehicle, &driver, &maintenance, &rental] {
        assert!(sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id=?)"
        )
        .bind(entity)
        .fetch_one(&pool)
        .await
        .unwrap());
    }

    let vehicle_fields = repository::list_fields(&pool, &vehicle).await.unwrap();
    let rental_fields = repository::list_fields(&pool, &rental).await.unwrap();
    let vehicle_id = vehicle_fields
        .iter()
        .find(|f| f.name == "code")
        .unwrap()
        .id
        .clone();
    let rental_vehicle = rental_fields
        .iter()
        .find(|f| f.name == "vehicle")
        .unwrap()
        .id
        .clone();
    assert!(!vehicle_id.is_empty());
    let relation = relation::create_relation(
        &pool,
        &rental,
        Some(&rental_vehicle),
        &vehicle,
        None,
        "rental_vehicle",
        "many_to_one",
        "restrict",
    )
    .await
    .unwrap();
    assert_eq!(relation.target_entity_id, vehicle);

    repository::update_entity_permissions(
        &pool,
        &rental,
        &[("user".into(), true, true), ("admin".into(), true, true)],
    )
    .await
    .unwrap();
    let permissions = repository::get_entity_permissions(&pool, &rental)
        .await
        .unwrap();
    assert!(permissions.iter().any(|p| p.role == "user" && p.can_edit));

    let vehicle_doc = repository::create_document(
        &pool,
        "veh-20",
        &vehicle,
        &json!({"code":"V20","plate_number":"TH-2020","status":"available"}),
        Some("alice"),
    )
    .await
    .unwrap();
    let driver_doc = repository::create_document(
        &pool,
        "drv-20",
        &driver,
        &json!({"code":"D20","name":"Acceptance Driver"}),
        Some("alice"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool, "rent-20", &rental,
        &json!({"vehicle":vehicle_doc.id,"driver":driver_doc.id,"status":"available","revenue":1500,"rented_at":"2026-09-09"}), Some("alice")
    ).await.unwrap();

    let rented = repository::transition_document_as_role(
        &pool,
        "rent-20",
        "rent",
        Some("alice"),
        None,
        "user",
    )
    .await
    .unwrap();
    assert_eq!(rented.payload["status"], "rented");
    let returned = repository::transition_document_as_role(
        &pool,
        "rent-20",
        "return",
        Some("alice"),
        None,
        "user",
    )
    .await
    .unwrap();
    assert_eq!(returned.payload["status"], "returned");

    let automation = repository::create_automation(
        &pool,
        &rental,
        "transition",
        "webhook",
        "https://example.test/maintenance-check",
        true,
    )
    .await
    .unwrap();
    automation::update(
        &pool,
        &automation.id,
        Some("{status} == \"returned\""),
        None,
        None,
        Some(3),
        Some(true),
    )
    .await
    .unwrap();
    let automations = repository::list_automations(&pool, &rental).await.unwrap();
    assert!(automations
        .iter()
        .any(|a| a.id == automation.id && a.active));

    let revenue_report = repository::create_report(
        &pool, &rental, "Rental Revenue",
        &json!({"fields":["status"],"aggregates":[{"field":"revenue","op":"sum","alias":"revenue_total"}]}), Some("alice")
    ).await.unwrap();
    let utilization_report = repository::create_report(
        &pool, &rental, "Utilization",
        &json!({"fields":["status"],"group_by":["status"],"aggregates":[{"field":"revenue","op":"count","alias":"rental_count"}]}), Some("alice")
    ).await.unwrap();
    let maintenance_report = repository::create_report(
        &pool, &maintenance, "Maintenance Cost",
        &json!({"fields":["cost"],"aggregates":[{"field":"cost","op":"sum","alias":"maintenance_cost"}]}), Some("alice")
    ).await.unwrap();
    assert_eq!(
        repository::list_reports(&pool, &rental)
            .await
            .unwrap()
            .len(),
        2
    );

    let dashboard = dashboard::create(
        &pool, "Vehicle Dashboard", "Acceptance dashboard", &json!([
            {"id":"total","kind":"kpi","title":"Total","entity_id":vehicle,"config":{"aggregates":[{"field":"code","op":"count","alias":"total"}]},"x":0,"y":0,"w":3,"h":2},
            {"id":"revenue","kind":"kpi","title":"Revenue","entity_id":rental,"config":{"aggregates":[{"field":"revenue","op":"sum","alias":"revenue"}]},"x":3,"y":0,"w":3,"h":2}
        ]), &json!({}), &[], &[], Some("alice")
    ).await.unwrap();
    let rendered = dashboard::run(&pool, &dashboard, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
    assert!(
        !revenue_report.id.is_empty()
            && !utilization_report.id.is_empty()
            && !maintenance_report.id.is_empty()
    );

    repository::create_document(
        &pool,
        "mnt-20",
        &maintenance,
        &json!({"vehicle":"veh-20","cost":250,"completed_at":"2026-09-09"}),
        Some("alice"),
    )
    .await
    .unwrap();
    let report_result = logholizon_core::report::run(
        &pool,
        &rental,
        &serde_json::from_value(revenue_report.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(report_result.total, 1);

    // A second, previously unknown business domain uses exactly the same runtime.
    let accounting = repository::create_module(
        &pool,
        "accounting_like",
        "Accounting Like",
        None,
        None,
        None,
        "alice",
        &json!({"entities":[
            {"name":"gl_account","label":"GL Account","fields":[
                {"name":"code","type":"text","required":true},
                {"name":"name","type":"text","required":true}
            ]},
            {"name":"journal_entry","label":"Journal Entry","fields":[
                {"name":"number","type":"text","required":true},
                {"name":"date","type":"date"}
            ]},
            {"name":"journal_line","label":"Journal Line","fields":[
                {"name":"entry","type":"reference","ref_entity":"journal_entry"},
                {"name":"account","type":"reference","ref_entity":"gl_account"},
                {"name":"debit","type":"number"},
                {"name":"credit","type":"number"}
            ]}
        ]}),
        Some("alice"),
    )
    .await
    .unwrap();
    repository::submit_module_for_review(&pool, &accounting.id, "alice", "user")
        .await
        .unwrap();
    repository::publish_module(&pool, &accounting.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    for name in ["gl_account", "journal_entry", "journal_line"] {
        let entity = format!("{}_{}", accounting.id, name);
        assert!(
            repository::get_entity_detail(&pool, &entity)
                .await
                .unwrap()
                .name
                == name
        );
    }

    // Core contains no accounting-owned physical tables; accounting is metadata only.
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(!tables
        .iter()
        .any(|name| name == "gl_account" || name == "journal_entry" || name == "journal_line"));
}
