use logholizon_core::{db, observability};
use serde_json::json;

#[tokio::test]
async fn observability_records_and_filters_correlation() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    observability::record(
        &pool,
        "warn",
        "security",
        "permission_denied",
        Some("alice"),
        Some("req-test"),
        Some("corr-test"),
        Some("route"),
        Some("/v1/admin"),
        Some(403),
        Some(12),
        "denied",
        &json!({"method":"GET"}),
    )
    .await
    .unwrap();
    let filter = observability::ObservabilityFilter {
        correlation_id: Some("corr-test".into()),
        ..Default::default()
    };
    let (total, items) = observability::list(&pool, &filter, 20, 0).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].action, "permission_denied");
    assert_eq!(items[0].request_id.as_deref(), Some("req-test"));
    assert_eq!(items[0].status_code, Some(403));
}

#[tokio::test]
async fn observability_records_auth_security_events() {
    use axum::{body::Body, http::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "admin", "password123")
        .await
        .unwrap();
    let config = logholizon_core::Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: "sqlite::memory:".into(),
        backup_interval_hours: 0,
        backup_keep: 7,
        notify_interval_secs: 0,
        notify_timeout_secs: 10,
        notify_max_attempts: 3,
        allowed_origins: Vec::new(),
        auth_rate_limit_max_attempts: 100,
        auth_rate_limit_window_secs: 60,
    };
    let app = logholizon_core::http::router(&config, pool.clone());

    // Successful login records login_success.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"password123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let session: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = session["token"].as_str().unwrap().to_string();

    // Failed login records login_failed.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"username":"admin","password":"wrongpass"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Logout records logout with the actor resolved from the token.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/logout")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify the security events were recorded with the right actions.
    let (total, items) = observability::list(
        &pool,
        &observability::ObservabilityFilter {
            category: Some("security".into()),
            ..Default::default()
        },
        100,
        0,
    )
    .await
    .unwrap();
    assert!(total >= 3);
    let actions: Vec<&str> = items.iter().map(|i| i.action.as_str()).collect();
    assert!(actions.contains(&"login_success"));
    assert!(actions.contains(&"login_failed"));
    assert!(actions.contains(&"logout"));
    let logout = items.iter().find(|i| i.action == "logout").unwrap();
    assert_eq!(logout.actor.as_deref(), Some("admin"));
}

#[tokio::test]
async fn observability_metrics_and_logs_endpoints() {
    use axum::{body::Body, http::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "admin", "password123")
        .await
        .unwrap();
    let config = logholizon_core::Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: "sqlite::memory:".into(),
        backup_interval_hours: 0,
        backup_keep: 7,
        notify_interval_secs: 0,
        notify_timeout_secs: 10,
        notify_max_attempts: 3,
        allowed_origins: Vec::new(),
        auth_rate_limit_max_attempts: 100,
        auth_rate_limit_window_secs: 60,
    };
    let app = logholizon_core::http::router(&config, pool.clone());

    // Login to obtain an admin token.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"password123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let session: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = session["token"].as_str().unwrap().to_string();

    // Metrics endpoint returns the expected shape.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/admin/observability/metrics")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let metrics: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(metrics["observability_events"].as_i64().unwrap() >= 1);
    assert!(metrics["logins"].as_i64().unwrap() >= 1);
    assert!(metrics["security_denials"].as_i64().unwrap() >= 0);

    // Logs endpoint returns the recorded login_success event.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/admin/observability/logs?category=security&limit=10")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let logs: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(logs["total"].as_i64().unwrap() >= 1);
    let actions: Vec<&str> = logs["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|i| i["action"].as_str())
        .collect();
    assert!(actions.contains(&"login_success"));
}
