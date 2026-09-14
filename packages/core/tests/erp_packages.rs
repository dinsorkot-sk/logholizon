use logholizon_core::{automation, dashboard, db, module_package, relation, repository};
use serde_json::json;

fn package_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../packages/erp/{name}.module.json"))
}

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

async fn install_package(
    pool: &sqlx::SqlitePool,
    name: &str,
) -> logholizon_core::repository::Module {
    let raw = std::fs::read_to_string(package_path(name)).unwrap();
    let package: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let preview = module_package::preview_package(pool, &package, "admin")
        .await
        .unwrap();
    assert!(
        preview.valid,
        "package {name} preview invalid: {:?}",
        preview.conflict
    );
    assert_eq!(preview.action, "install");
    let installed = module_package::install_package(pool, &package, "admin", Some("cli"))
        .await
        .unwrap();
    assert_eq!(installed.status, "enabled");
    assert_eq!(installed.name, name);
    installed
}

#[tokio::test]
async fn erp_sample_package_installs_and_operates() {
    let pool = setup().await;
    let raw = std::fs::read_to_string(package_path("vehicle")).unwrap();
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
    let raw = std::fs::read_to_string(package_path("vehicle")).unwrap();
    let mut package: serde_json::Value = serde_json::from_str(&raw).unwrap();
    package["module"]["automations"][0]["entity"] = json!("ghost");
    let err = module_package::install_package(&pool, &package, "admin", Some("cli"))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown entity"));
}

#[tokio::test]
async fn erp_accounting_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "accounting").await;
    let account = format!("{}_gl_account", installed.id);
    let entry = format!("{}_journal_entry", installed.id);
    let line = format!("{}_journal_line", installed.id);

    let cash = repository::create_document(
        &pool,
        "acc-cash",
        &account,
        &json!({"code": "1000", "name": "Cash", "account_type": "asset", "is_active": true}),
        Some("cli"),
    )
    .await
    .unwrap();
    let revenue = repository::create_document(
        &pool,
        "acc-rev",
        &account,
        &json!({"code": "4000", "name": "Sales Revenue", "account_type": "revenue", "is_active": true}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "je-1",
        &entry,
        &json!({"number": "JE-0001", "entry_date": "2026-09-14", "memo": "Sale", "status": "draft"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "jl-1",
        &line,
        &json!({"entry": "je-1", "account": cash.id, "debit": 1000.0, "credit": 0.0}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "jl-2",
        &line,
        &json!({"entry": "je-1", "account": revenue.id, "debit": 0.0, "credit": 1000.0}),
        Some("cli"),
    )
    .await
    .unwrap();
    let posted = repository::transition_document(&pool, "je-1", "post", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(posted.payload["status"], "posted");

    // Packaged automation (posted → webhook) fires.
    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    // Trial totals report balances.
    let reports = repository::list_reports(&pool, &line).await.unwrap();
    let trial = reports.iter().find(|r| r.name == "Trial totals").unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &line,
        &serde_json::from_value(trial.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    // Dashboard renders all widgets.
    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "Accounting Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn erp_inventory_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "inventory").await;
    let product = format!("{}_product", installed.id);
    let warehouse = format!("{}_warehouse", installed.id);
    let move_entity = format!("{}_stock_move", installed.id);

    let widget = repository::create_document(
        &pool,
        "inv-widget",
        &product,
        &json!({"sku": "W-001", "name": "Widget", "unit": "pcs", "unit_cost": 10.0, "reorder_level": 5}),
        Some("cli"),
    )
    .await
    .unwrap();
    let main = repository::create_document(
        &pool,
        "inv-main",
        &warehouse,
        &json!({"code": "WH-1", "location": "Bangkok"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "mv-1",
        &move_entity,
        &json!({"product": widget.id, "warehouse": main.id, "move_type": "in", "quantity": 100, "moved_at": "2026-09-14", "status": "draft"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let confirmed = repository::transition_document(&pool, "mv-1", "confirm", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(confirmed.payload["status"], "confirmed");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    let reports = repository::list_reports(&pool, &move_entity).await.unwrap();
    let moves = reports.iter().find(|r| r.name == "Moves by type").unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &move_entity,
        &serde_json::from_value(moves.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "Inventory Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_sales_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "sales").await;
    let customer = format!("{}_customer", installed.id);
    let order = format!("{}_sales_order", installed.id);
    let order_line = format!("{}_order_line", installed.id);

    let acme = repository::create_document(
        &pool,
        "so-acme",
        &customer,
        &json!({"code": "C-001", "name": "Acme", "email": "buy@acme.test", "phone": "+661234567"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "ord-1",
        &order,
        &json!({"number": "SO-0001", "customer": acme.id, "order_date": "2026-09-14", "total": 500.0, "status": "quote"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let line = repository::create_document(
        &pool,
        "ol-1",
        &order_line,
        &json!({"order": "ord-1", "item": "Widget", "quantity": 5, "unit_price": 100.0}),
        Some("cli"),
    )
    .await
    .unwrap();
    // Formula line_total = quantity * unit_price.
    assert_eq!(line.payload["line_total"], json!(500.0));
    let confirmed = repository::transition_document(&pool, "ord-1", "confirm", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(confirmed.payload["status"], "confirmed");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    let reports = repository::list_reports(&pool, &order).await.unwrap();
    let revenue = reports
        .iter()
        .find(|r| r.name == "Revenue by status")
        .unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &order,
        &serde_json::from_value(revenue.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "Sales Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_hr_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "hr").await;
    let employee = format!("{}_employee", installed.id);
    let leave = format!("{}_leave_request", installed.id);
    let payroll = format!("{}_payroll_run", installed.id);

    let ada = repository::create_document(
        &pool,
        "hr-ada",
        &employee,
        &json!({"code": "E-001", "name": "Ada", "email": "ada@example.test", "department": "Eng", "hire_date": "2024-01-15"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "lv-1",
        &leave,
        &json!({"employee": ada.id, "leave_type": "annual", "start_date": "2026-10-01", "end_date": "2026-10-03", "status": "draft"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let approved = repository::transition_document(&pool, "lv-1", "approve", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(approved.payload["status"], "approved");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    repository::create_document(
        &pool,
        "pr-1",
        &payroll,
        &json!({"period": "2026-09", "run_date": "2026-09-30", "total_amount": 250000.0, "status": "draft"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let processed = repository::transition_document(&pool, "pr-1", "process", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(processed.payload["status"], "processed");

    let reports = repository::list_reports(&pool, &employee).await.unwrap();
    let headcount = reports
        .iter()
        .find(|r| r.name == "Headcount by department")
        .unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &employee,
        &serde_json::from_value(headcount.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards.iter().find(|d| d.name == "HR Overview").unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_pos_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "pos").await;
    let store = format!("{}_store", installed.id);
    let terminal = format!("{}_terminal", installed.id);
    let ticket = format!("{}_sale_ticket", installed.id);
    let ticket_line = format!("{}_ticket_line", installed.id);

    let main = repository::create_document(
        &pool,
        "pos-store",
        &store,
        &json!({"code": "S-01", "name": "Siam", "location": "Bangkok"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let till = repository::create_document(
        &pool,
        "pos-till",
        &terminal,
        &json!({"code": "T-01", "store": main.id, "status": "active"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "pos-t1",
        &ticket,
        &json!({"number": "T-0001", "terminal": till.id, "sold_at": "2026-09-14T10:00:00", "tender": "card", "total": 300.0, "status": "open"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let line = repository::create_document(
        &pool,
        "pos-l1",
        &ticket_line,
        &json!({"ticket": "pos-t1", "item": "Coffee", "quantity": 3, "unit_price": 100.0}),
        Some("cli"),
    )
    .await
    .unwrap();
    assert_eq!(line.payload["line_total"], json!(300.0));
    let paid = repository::transition_document(&pool, "pos-t1", "pay", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(paid.payload["status"], "paid");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    let reports = repository::list_reports(&pool, &ticket).await.unwrap();
    let sales = reports
        .iter()
        .find(|r| r.name == "Sales by tender")
        .unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &ticket,
        &serde_json::from_value(sales.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "POS Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_manufacturing_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "manufacturing").await;
    let bom = format!("{}_bom", installed.id);
    let bom_line = format!("{}_bom_line", installed.id);
    let run = format!("{}_production_run", installed.id);

    let widget_bom = repository::create_document(
        &pool,
        "mfg-bom",
        &bom,
        &json!({"code": "BOM-001", "product": "Widget", "revision": "A"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "mfg-bl",
        &bom_line,
        &json!({"bom": widget_bom.id, "component": "Screw", "quantity": 4}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "mfg-run",
        &run,
        &json!({"number": "PR-0001", "bom": widget_bom.id, "planned_quantity": 100, "produced_quantity": 0, "started_at": "2026-09-14", "status": "planned"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let running = repository::transition_document(&pool, "mfg-run", "start", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(running.payload["status"], "running");
    repository::update_document(
        &pool,
        "mfg-run",
        &json!({"produced_quantity": 100}),
        Some("cli"),
        None,
    )
    .await
    .unwrap();
    let done = repository::transition_document(&pool, "mfg-run", "complete", Some("cli"), None)
        .await
        .unwrap();
    assert_eq!(done.payload["status"], "completed");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    let reports = repository::list_reports(&pool, &run).await.unwrap();
    let output = reports
        .iter()
        .find(|r| r.name == "Output by status")
        .unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &run,
        &serde_json::from_value(output.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "Manufacturing Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_dormitory_package_installs_and_operates() {
    let pool = setup().await;
    let installed = install_package(&pool, "dormitory").await;
    let building = format!("{}_building", installed.id);
    let room = format!("{}_room", installed.id);
    let resident = format!("{}_resident", installed.id);
    let booking = format!("{}_booking", installed.id);

    let tower = repository::create_document(
        &pool,
        "drm-bld",
        &building,
        &json!({"code": "B-01", "name": "North Tower", "address": "Bangkok"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let room101 = repository::create_document(
        &pool,
        "drm-room",
        &room,
        &json!({"building": tower.id, "number": "101", "capacity": 2, "monthly_rate": 5000.0, "status": "vacant"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let bob = repository::create_document(
        &pool,
        "drm-bob",
        &resident,
        &json!({"code": "R-001", "name": "Bob", "phone": "+669876543", "email": "bob@example.test"}),
        Some("cli"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "drm-bk",
        &booking,
        &json!({"room": room101.id, "resident": bob.id, "check_in": "2026-10-01", "check_out": "2027-03-31", "status": "reserved"}),
        Some("cli"),
    )
    .await
    .unwrap();
    let checked_in =
        repository::transition_document(&pool, "drm-bk", "check_in", Some("cli"), None)
            .await
            .unwrap();
    assert_eq!(checked_in.payload["status"], "checked_in");

    assert!(automation::enqueue_events(&pool).await.unwrap() >= 1);
    assert!(automation::process_pending(&pool).await.unwrap() >= 1);

    let reports = repository::list_reports(&pool, &booking).await.unwrap();
    let by_status = reports
        .iter()
        .find(|r| r.name == "Bookings by status")
        .unwrap();
    let result = logholizon_core::report::run(
        &pool,
        &booking,
        &serde_json::from_value(by_status.config.clone()).unwrap(),
        "user",
    )
    .await
    .unwrap();
    assert_eq!(result.total, 1);

    let dashboards = dashboard::list(&pool).await.unwrap();
    let overview = dashboards
        .iter()
        .find(|d| d.name == "Dormitory Overview")
        .unwrap();
    let rendered = dashboard::run(&pool, overview, "user").await.unwrap();
    assert_eq!(rendered.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn erp_entity_names_are_scoped_per_module() {
    // Independent modules (and seed demo data) may reuse common entity
    // names; uniqueness is per (module_id, name), not global.
    let pool = setup().await;
    logholizon_core::seed::seed(&pool).await.unwrap();
    let seed_product: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE name = 'product')")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(seed_product);
    let installed = install_package(&pool, "inventory").await;
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM _meta_entity WHERE name = 'product' ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"product".to_string()));
    assert!(ids.contains(&format!("{}_product", installed.id)));
}
