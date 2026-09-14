use logholizon_core::{automation, dashboard, db, module_package, relation, repository};
use serde_json::json;

fn package_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/erp/vehicle.module.json")
}

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn erp_sample_package_installs_and_operates() {
    let pool = setup().await;
    let raw = std::fs::read_to_string(package_path()).unwrap();
    let package: serde_json::Value = serde_json::from_str(&raw).unwrap();

    let preview = module_package::preview_package(&pool, &package, "admin")
        .await
        .unwrap();
    assert!(preview.valid);
    assert_eq!(preview.action, "install");
    assert_eq!(preview.migration.creates.len(), 4);

    let installed = module_package::install_package(&pool, &package, "admin", Some("cli"))
        .await
        .unwrap();
    assert_eq!(installed.status, "enabled");
    assert_eq!(installed.name, "vehicle");

    // Entities materialized with fields, workflow, views, reports, layouts.
    let vehicle = format!("{}_vehicle", installed.id);
    let driver = format!("{}_driver", installed.id);
    let maintenance = format!("{}_maintenance", installed.id);
    let rental = format!("{}_rental", installed.id);
    for entity in [&vehicle, &driver, &maintenance, &rental] {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
                .bind(entity)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(exists, "missing entity {entity}");
    }
    assert_eq!(
        repository::list_fields(&pool, &vehicle)
            .await
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        repository::list_reports(&pool, &rental)
            .await
            .unwrap()
            .len(),
        1
    );

    // Relations materialized from definition names.
    let relations = relation::list_relations(&pool, &rental).await.unwrap();
    assert!(relations
        .iter()
        .any(|r| r.name == "rental_vehicle" && r.target_entity_id == vehicle));
    assert!(relations
        .iter()
        .any(|r| r.name == "rental_driver" && r.target_entity_id == driver));

    // Actions materialized.
    let actions = repository::list_module_actions(&pool, &rental)
        .await
        .unwrap();
    assert!(actions
        .iter()
        .any(|a| a.name == "send_receipt" && a.kind == "notify"));

    // Automations materialized with condition.
    let automations = repository::list_automations(&pool, &rental).await.unwrap();
    assert_eq!(automations.len(), 1);
    assert_eq!(automations[0].trigger, "transition");
    let stored: String = sqlx::query_scalar("SELECT condition FROM _automation WHERE id = ?")
        .bind(&automations[0].id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "{status} == \"returned\"");

    // Dashboards materialized with resolved entity IDs.
    let dashboards = dashboard::list(&pool).await.unwrap();
    let fleet = dashboards
        .iter()
        .find(|d| d.name == "Fleet Overview")
        .unwrap();
    let widgets = fleet.layout.as_array().unwrap();
    assert_eq!(widgets.len(), 2);
    assert!(widgets.iter().all(|w| {
        let entity = w["entity_id"].as_str().unwrap_or("");
        entity == vehicle || entity == rental
    }));
    let rendered = dashboard::run(&pool, fleet, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);

    // Operate: CRUD + workflow + automation execution + report.
    let vehicle_doc = repository::create_document(
        &pool,
        "veh-erp",
        &vehicle,
        &json!({"code": "V1", "plate_number": "TH-1", "status": "available"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let driver_doc = repository::create_document(
        &pool,
        "drv-erp",
        &driver,
        &json!({"code": "D1", "name": "ERP Driver"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "rent-erp",
        &rental,
        &json!({"vehicle": vehicle_doc.id, "driver": driver_doc.id, "status": "available", "revenue": 2000, "rented_at": "2026-09-14"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let rented = repository::transition_document(&pool, "rent-erp", "rent", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(rented.payload["status"], "rented");
    let returned = repository::transition_document(&pool, "rent-erp", "return", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(returned.payload["status"], "returned");

    // The packaged automation (returned → webhook) enqueues and processes.
    let enqueued = automation::enqueue_events(&pool).await.unwrap();
    assert!(enqueued >= 1);
    let processed = automation::process_pending(&pool).await.unwrap();
    assert!(processed >= 1);

    let reports = repository::list_reports(&pool, &rental).await.unwrap();
    let revenue = reports.iter().find(|r| r.name == "Rental revenue").unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &rental,
        &serde_json::from_value(revenue.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);
}

#[tokio::test]
async fn erp_package_rejects_unknown_entity_reference() {
    let pool = setup().await;
    let raw = std::fs::read_to_string(package_path()).unwrap();
    let mut package: serde_json::Value = serde_json::from_str(&raw).unwrap();
    package["module"]["automations"][0]["entity"] = json!("ghost");
    let err = module_package::install_package(&pool, &package, "admin", Some("cli"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown entity"));
}
