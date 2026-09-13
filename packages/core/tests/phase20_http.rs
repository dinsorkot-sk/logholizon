use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use logholizon_core::{db, http, Config};
use serde_json::{json, Value};
use tower::ServiceExt;

fn test_config(url: &str) -> Config {
    Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: url.into(),
        backup_interval_hours: 0,
        backup_keep: 7,
        notify_interval_secs: 0,
        notify_timeout_secs: 10,
        notify_max_attempts: 3,
    }
}

async fn setup_app() -> (axum::Router, String) {
    let dir = std::env::temp_dir().join(format!(
        "logholizon-test-phase20-http-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let db_path = dir.join("core.db");
    let url = format!("sqlite://{}", db_path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&url).await.unwrap();
    db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "alice", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "alice", "password123")
        .await
        .unwrap();
    let app = http::router(&test_config(&url), pool);
    (app, session.token)
}

async fn call(app: &axum::Router, token: &str, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let request = builder
        .body(body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

fn vehicle_definition() -> Value {
    json!({
        "entities": [
            {"name":"vehicle","label":"Vehicle","fields":[
                {"name":"code","type":"text","required":true},
                {"name":"plate_number","type":"text","required":true},
                {"name":"status","type":"select","required":true,"is_status":true,
                 "options":[{"value":"available","label":"Available"},{"value":"rented","label":"Rented"},{"value":"returned","label":"Returned"}]}
            ]},
            {"name":"driver","label":"Driver","fields":[
                {"name":"code","type":"text","required":true},
                {"name":"name","type":"text","required":true}
            ]},
            {"name":"rental","label":"Rental","fields":[
                {"name":"vehicle","type":"reference","ref_entity":"vehicle","required":true},
                {"name":"driver","type":"reference","ref_entity":"driver","required":true},
                {"name":"status","type":"select","required":true,"is_status":true,
                 "options":[{"value":"available","label":"Available"},{"value":"rented","label":"Rented"},{"value":"returned","label":"Returned"}]},
                {"name":"revenue","type":"number","required":true}
            ],"workflow":{"states":[
                {"name":"available","label":"Available"},{"name":"rented","label":"Rented"},{"name":"returned","label":"Returned"}
            ],"transitions":[
                {"from_state":"available","to_state":"rented","action":"rent"},
                {"from_state":"rented","to_state":"returned","action":"return"}
            ]}}
        ]
    })
}

/// Phase 20 acceptance through the public HTTP API: a previously unknown
/// business domain is created, published, and operated with no Rust changes.
#[tokio::test]
async fn phase20_vehicle_lifecycle_over_http() {
    let (app, token) = setup_app().await;

    // 1. Drafts may start empty (Module Builder shell-first flow).
    let (status, empty) = call(
        &app,
        &token,
        "POST",
        "/v1/modules",
        Some(json!({"name": "vehicle_management", "label": "Vehicle Management", "definition": {"entities": []}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{empty:?}");
    let module_id = empty["id"].as_str().unwrap().to_string();

    // 2. Empty drafts cannot go to review; completeness is enforced there.
    let (status, _) = call(&app, &token, "POST", &format!("/v1/modules/{module_id}/review"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // 3. Fill in the definition through the draft update path.
    let (status, _) = call(
        &app,
        &token,
        "PUT",
        &format!("/v1/modules/{module_id}"),
        Some(json!({"definition": vehicle_definition()})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 4. Review -> publish -> enable through the public lifecycle.
    for action in ["review", "publish", "enable"] {
        let (status, body) = call(&app, &token, "POST", &format!("/v1/modules/{module_id}/{action}"), None).await;
        assert_eq!(status, StatusCode::OK, "{action}: {body:?}");
    }

    // 5. Full CRUD through the generic module-entity runtime.
    let (status, _) = call(
        &app,
        &token,
        "POST",
        &format!("/v1/modules/{module_id}/entities/vehicle"),
        Some(json!({"id": "veh-1", "payload": {"code": "V1", "plate_number": "AB-1", "status": "available"}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        &app,
        &token,
        "POST",
        &format!("/v1/modules/{module_id}/entities/driver"),
        Some(json!({"id": "drv-1", "payload": {"code": "D1", "name": "Ann"}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        &app,
        &token,
        "POST",
        &format!("/v1/modules/{module_id}/entities/rental"),
        Some(json!({"id": "rent-1", "payload": {"vehicle": "veh-1", "driver": "drv-1", "status": "available", "revenue": 1500}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // 6. Workflow transitions follow the published definition.
    let (status, rented) = call(
        &app,
        &token,
        "POST",
        "/v1/documents/rent-1/transition",
        Some(json!({"action": "rent"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rented:?}");
    assert_eq!(rented["payload"]["status"], "rented");
    let (status, returned) = call(
        &app,
        &token,
        "POST",
        "/v1/documents/rent-1/transition",
        Some(json!({"action": "return"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{returned:?}");
    assert_eq!(returned["payload"]["status"], "returned");

    // 7. A second unknown domain (accounting-like) uses the same runtime
    // with no accounting tables, routes, or types in Core.
    let (status, accounting) = call(
        &app,
        &token,
        "POST",
        "/v1/modules",
        Some(json!({"name": "accounting_like", "label": "Accounting Like", "definition": {"entities": [
            {"name": "gl_account", "label": "GL Account", "fields": [
                {"name": "code", "type": "text", "required": true},
                {"name": "name", "type": "text", "required": true}
            ]},
            {"name": "journal_line", "label": "Journal Line", "fields": [
                {"name": "account", "type": "reference", "ref_entity": "gl_account"},
                {"name": "debit", "type": "number"}
            ]}
        ]}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{accounting:?}");
    let accounting_id = accounting["id"].as_str().unwrap().to_string();
    for action in ["review", "publish", "enable"] {
        let (status, body) = call(&app, &token, "POST", &format!("/v1/modules/{accounting_id}/{action}"), None).await;
        assert_eq!(status, StatusCode::OK, "{action}: {body:?}");
    }
    let (status, _) = call(
        &app,
        &token,
        "POST",
        &format!("/v1/modules/{accounting_id}/entities/gl_account"),
        Some(json!({"id": "a1", "payload": {"code": "1000", "name": "Cash"}})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}
