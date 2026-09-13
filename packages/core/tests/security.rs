use logholizon_core::security;

#[test]
fn outbound_url_blocks_local_and_private_targets() {
    assert!(security::validate_outbound_url("https://example.com/hook").is_ok());
    assert!(security::validate_outbound_url("http://localhost:8080/hook").is_err());
    assert!(security::validate_outbound_url("http://127.0.0.1:8080/hook").is_err());
    assert!(security::validate_outbound_url("http://10.0.0.5/hook").is_err());
    assert!(security::validate_outbound_url("http://192.168.1.10/hook").is_err());
    assert!(security::validate_outbound_url("file:///etc/passwd").is_err());
}

#[test]
fn outbound_url_rejects_credentials_and_fragments() {
    assert!(security::validate_outbound_url("https://user:pass@example.com/hook").is_err());
    assert!(security::validate_outbound_url("https://example.com/hook#secret").is_err());
}

#[test]
fn webhook_headers_reject_hop_by_hop_headers() {
    assert!(security::validate_webhook_headers(&serde_json::json!({"X-Test":"ok"})).is_ok());
    assert!(security::validate_webhook_headers(&serde_json::json!({"Host":"internal"})).is_err());
    assert!(
        security::validate_webhook_headers(&serde_json::json!({"Content-Length":"1"})).is_err()
    );
}

#[test]
fn filenames_cannot_escape_attachment_namespace() {
    assert!(security::validate_filename("report.pdf").is_ok());
    assert!(security::validate_filename("../secret.txt").is_err());
    assert!(security::validate_filename("..\\secret.txt").is_err());
    assert!(security::validate_filename("evil\nname.txt").is_err());
}

#[tokio::test]
async fn tenant_access_blocks_cross_tenant_module_entity_and_document() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    let definition = serde_json::json!({
        "entities": [{
            "name": "record",
            "label": "Record",
            "fields": [{"name": "name", "type": "text"}]
        }]
    });
    let alice = logholizon_core::repository::create_module(
        &pool,
        "tenant_records",
        "Tenant Records",
        None,
        None,
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    logholizon_core::module_lifecycle::submit_module_for_review(&pool, &alice.id, "alice", "user")
        .await
        .unwrap();
    logholizon_core::repository::publish_module(&pool, &alice.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    let entity_id = format!("{}_record", alice.id);
    assert!(logholizon_core::repository::check_entity_tenant_access(
        &pool, &entity_id, "alice", "user"
    )
    .await
    .is_ok());
    assert!(logholizon_core::repository::check_entity_tenant_access(
        &pool, &entity_id, "bob", "user"
    )
    .await
    .is_err());
    let doc = logholizon_core::repository::create_document_as_role(
        &pool,
        "alice-doc",
        &entity_id,
        &serde_json::json!({"name":"Alice"}),
        Some("alice"),
        "user",
    )
    .await
    .unwrap();
    assert!(logholizon_core::repository::check_document_tenant_access(
        &pool, &doc.id, "alice", "user"
    )
    .await
    .is_ok());
    assert!(logholizon_core::repository::check_document_tenant_access(
        &pool, &doc.id, "bob", "user"
    )
    .await
    .is_err());
    let tenant: String = sqlx::query_scalar("SELECT tenant_id FROM _meta_entity WHERE id = ?")
        .bind(&entity_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tenant, "alice");
    let doc_tenant: String = sqlx::query_scalar("SELECT tenant_id FROM _doc WHERE id = ?")
        .bind(&doc.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_tenant, "alice");
}

// --- Phase B: rate limiter ---

#[test]
fn auth_rate_limiter_counts_failures_in_window() {
    use logholizon_core::security::AuthRateLimiter;
    use std::time::Duration;
    let mut limiter = AuthRateLimiter::new();
    let window = Duration::from_secs(60);
    // Below the budget: not limited.
    for _ in 0..10 {
        assert!(!limiter.record_failure("1.2.3.4", 10, window));
    }
    // 11th failure exceeds max 10: limited.
    assert!(limiter.record_failure("1.2.3.4", 10, window));
    assert!(limiter.is_limited("1.2.3.4", 10, window));
    // Other IPs are unaffected.
    assert!(!limiter.is_limited("5.6.7.8", 10, window));
    // Success clears the counter.
    limiter.clear("1.2.3.4");
    assert!(!limiter.is_limited("1.2.3.4", 10, window));
}

#[test]
fn auth_rate_limiter_window_expiry_forgives_old_failures() {
    use logholizon_core::security::AuthRateLimiter;
    use std::time::Duration;
    let mut limiter = AuthRateLimiter::new();
    // Budget of 1: first failure fills the budget (not yet limited),
    // second failure inside the window trips the limiter.
    assert!(!limiter.record_failure("9.9.9.9", 1, Duration::from_secs(60)));
    assert!(limiter.record_failure("9.9.9.9", 1, Duration::from_secs(60)));
    assert!(limiter.is_limited("9.9.9.9", 1, Duration::from_secs(60)));
    // Tiny window: old failures expire, key is forgiven.
    std::thread::sleep(Duration::from_millis(10));
    assert!(!limiter.is_limited("9.9.9.9", 1, Duration::from_millis(1)));
}

#[test]
fn client_ip_key_prefers_forwarded_for() {
    use logholizon_core::security::client_ip_key;
    assert_eq!(
        client_ip_key(Some("203.0.113.7, 70.41.3.18"), "10.0.0.1"),
        "203.0.113.7"
    );
    assert_eq!(client_ip_key(None, "10.0.0.1"), "10.0.0.1");
    assert_eq!(client_ip_key(Some("  "), "10.0.0.1"), "10.0.0.1");
}

// --- Phase B: inbound webhook signature ---

#[test]
fn webhook_signature_round_trip_verifies() {
    let secret = "test-secret";
    let payload = r#"{"event":"record.created"}"#;
    let signed = logholizon_core::notification::sign_webhook(secret, payload);
    assert!(signed.starts_with("sha256="));
    assert!(logholizon_core::security::verify_webhook_signature(
        secret,
        payload.as_bytes(),
        &signed
    ));
    assert!(logholizon_core::notification::verify_webhook(
        secret,
        payload.as_bytes(),
        &signed
    ));
}

#[test]
fn webhook_signature_rejects_wrong_secret_and_tampered_payload() {
    let signed = logholizon_core::notification::sign_webhook("correct", "{}");
    assert!(!logholizon_core::security::verify_webhook_signature(
        "wrong", b"{}", &signed
    ));
    assert!(!logholizon_core::security::verify_webhook_signature(
        "correct",
        b"{\"tampered\":true}",
        &signed
    ));
    assert!(!logholizon_core::security::verify_webhook_signature(
        "correct",
        b"{}",
        "sha256=deadbeef"
    ));
    assert!(!logholizon_core::security::verify_webhook_signature(
        "correct",
        b"{}",
        "not-a-signature"
    ));
    assert!(!logholizon_core::security::verify_webhook_signature(
        "correct",
        b"{}",
        "sha256=zzzz"
    ));
}

// --- Phase B: auth bypass / RBAC escalation over HTTP ---

async fn http_test_app() -> axum::Router {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "admin", "password123")
        .await
        .unwrap();
    logholizon_core::auth::register(&pool, "bob", "password123")
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
    // NOTE: sqlite::memory: pools are single-connection (see db::connect),
    // so the router shares the seeded DB above.
    logholizon_core::http::router(&config, pool)
}

#[tokio::test]
async fn http_rejects_missing_and_invalid_bearer_tokens() {
    use axum::{body::Body, http::Request, http::StatusCode};
    use tower::ServiceExt;
    let app = http_test_app().await;
    // No token at all.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // Garbage token.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/auth/me")
                .header("authorization", "Bearer garbage-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // Malformed scheme (no Bearer prefix).
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/auth/me")
                .header("authorization", "Token abc123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn http_login_rate_limit_returns_429() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
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
        auth_rate_limit_max_attempts: 3,
        auth_rate_limit_window_secs: 60,
    };
    let app = logholizon_core::http::router(&config, pool);
    let login = || {
        let body = Body::from(r#"{"username":"admin","password":"wrong"}"#);
        Request::builder()
            .method("POST")
            .uri("/v1/auth/login")
            .header("content-type", "application/json")
            .body(body)
            .unwrap()
    };
    // Budget of 3: first 3 failures are 401s, the 4th failure trips the
    // limiter, and the 5th attempt is rejected with 429 before auth runs.
    for _ in 0..3 {
        let response = app.clone().oneshot(login()).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    let response = app.clone().oneshot(login()).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // 5th attempt trips the limiter: 429 with a generic message.
    let response = app.clone().oneshot(login()).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["code"], "too_many_requests");
    // Generic message: must not reveal whether the username exists.
    assert!(!body["message"].as_str().unwrap_or("").contains("admin"));
}

// --- Phase B: oversized payloads and header injection ---

#[test]
fn webhook_body_and_header_caps_hold() {
    let big = "x".repeat(1_000_001);
    assert!(big.len() > logholizon_core::security::MAX_WEBHOOK_BODY_BYTES);
    // 33 headers exceeds the 32 cap.
    let mut headers = serde_json::Map::new();
    for i in 0..33 {
        headers.insert(format!("X-Header-{i}"), serde_json::json!("v"));
    }
    assert!(
        logholizon_core::security::validate_webhook_headers(&serde_json::Value::Object(headers))
            .is_err()
    );
    // Oversized single header value rejected.
    let huge = "v".repeat(4097);
    assert!(logholizon_core::security::validate_webhook_headers(
        &serde_json::json!({"X-Big": huge})
    )
    .is_err());
}

#[test]
fn attachment_caps_reject_empty_oversize_and_bad_types() {
    // Empty file rejected.
    assert!(
        logholizon_core::repository::validate_attachment("a.pdf", "application/pdf", 0).is_err()
    );
    // Over 5MB rejected.
    assert!(logholizon_core::repository::validate_attachment(
        "a.pdf",
        "application/pdf",
        6 * 1024 * 1024
    )
    .is_err());
    // Executable type rejected.
    assert!(logholizon_core::repository::validate_attachment(
        "a.exe",
        "application/x-msdownload",
        100
    )
    .is_err());
    // Allowed types pass.
    assert!(
        logholizon_core::repository::validate_attachment("photo.png", "image/png", 100).is_ok()
    );
    assert!(
        logholizon_core::repository::validate_attachment("doc.pdf", "application/pdf", 100).is_ok()
    );
}

// --- Phase B: session lifecycle + recovery ---

#[tokio::test]
async fn password_reset_invalidates_existing_sessions() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    let user = logholizon_core::auth::register(&pool, "carol", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "carol", "password123")
        .await
        .unwrap();
    // Token works before reset.
    assert!(logholizon_core::auth::user_for_token(&pool, &session.token)
        .await
        .is_ok());
    // Reset via recovery helper invalidates the old token.
    logholizon_core::auth::reset_password_by_username(&pool, "carol", "newpassword123")
        .await
        .unwrap();
    assert!(logholizon_core::auth::user_for_token(&pool, &session.token)
        .await
        .is_err());
    // New password logs in; old password does not.
    assert!(
        logholizon_core::auth::login(&pool, "carol", "newpassword123")
            .await
            .is_ok()
    );
    assert!(logholizon_core::auth::login(&pool, "carol", "password123")
        .await
        .is_err());
    let _ = user;
}

#[tokio::test]
async fn logout_invalidates_token() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "dave", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "dave", "password123")
        .await
        .unwrap();
    logholizon_core::auth::logout(&pool, &session.token)
        .await
        .unwrap();
    assert!(logholizon_core::auth::user_for_token(&pool, &session.token)
        .await
        .is_err());
}

#[tokio::test]
async fn expired_sessions_are_rejected() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    let user = logholizon_core::auth::register(&pool, "erin", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "erin", "password123")
        .await
        .unwrap();
    // Backdate the session past the 7-day expiry: user_for_token must reject.
    sqlx::query("UPDATE _session SET expires_at = datetime('now', '-8 days') WHERE token = ?")
        .bind(&session.token)
        .execute(&pool)
        .await
        .unwrap();
    let err = logholizon_core::auth::user_for_token(&pool, &session.token)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("expired"));
    let _ = user;
}

#[tokio::test]
async fn cors_allowlist_permits_listed_origin_only() {
    use axum::{body::Body, http::Request, http::StatusCode};
    use tower::ServiceExt;
    async fn app_with_origins(origins: Vec<String>) -> axum::Router {
        let pool = logholizon_core::db::connect("sqlite::memory:")
            .await
            .unwrap();
        logholizon_core::db::migrate(&pool).await.unwrap();
        logholizon_core::seed::seed(&pool).await.unwrap();
        let config = logholizon_core::Config {
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "sqlite::memory:".into(),
            backup_interval_hours: 0,
            backup_keep: 7,
            notify_interval_secs: 0,
            notify_timeout_secs: 10,
            notify_max_attempts: 3,
            allowed_origins: origins,
            auth_rate_limit_max_attempts: 100,
            auth_rate_limit_window_secs: 60,
        };
        // NOTE: sqlite::memory: pools are single-connection, so the router
        // shares the seeded DB above.
        logholizon_core::http::router(&config, pool)
    }
    // Same-origin default: no CORS headers for a cross-origin preflight.
    let app = app_with_origins(Vec::new()).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/auth/login")
                .header("origin", "https://evil.example.com")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response
        .headers()
        .get("access-control-allow-origin")
        .is_none());
    // Listed origin is echoed back.
    let app = app_with_origins(vec!["https://app.example.com".into()]).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/auth/login")
                .header("origin", "https://app.example.com")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("https://app.example.com")
    );
    // Unlisted origin is denied even when an allowlist exists.
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/v1/auth/login")
                .header("origin", "https://evil.example.com")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let _ = response.status() == StatusCode::OK;
    assert!(response
        .headers()
        .get("access-control-allow-origin")
        .is_none());
}

// --- Phase B: secret redaction in observability ---

#[test]
fn observability_redacts_secrets() {
    let scrubbed = logholizon_core::observability::redact_secrets(&serde_json::json!({
        "username": "carol",
        "password": "hunter2",
        "token": "abc123",
        "webhook_secret": "shh",
        "nested": {"authorization": "Bearer xyz", "safe": "ok"}
    }));
    assert_eq!(scrubbed["username"], "carol");
    assert_eq!(scrubbed["password"], "[redacted]");
    assert_eq!(scrubbed["token"], "[redacted]");
    assert_eq!(scrubbed["webhook_secret"], "[redacted]");
    assert_eq!(scrubbed["nested"]["authorization"], "[redacted]");
    assert_eq!(scrubbed["nested"]["safe"], "ok");
}

#[tokio::test]
async fn observability_record_never_stores_raw_secrets() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    logholizon_core::observability::record(
        &pool,
        "warn",
        "security",
        "login_failed",
        Some("carol"),
        None,
        None,
        Some("user"),
        Some("carol"),
        Some(401),
        None,
        "invalid username or password",
        &serde_json::json!({"password": "hunter2", "token": "abc"}),
    )
    .await
    .unwrap();
    let (_, items) = logholizon_core::observability::list(
        &pool,
        &logholizon_core::observability::ObservabilityFilter::default(),
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].metadata["password"], "[redacted]");
    assert_eq!(items[0].metadata["token"], "[redacted]");
}
