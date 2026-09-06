use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use logholizon_core::{db, http, Config};
use tower::ServiceExt;

fn test_config() -> Config {
    Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: "sqlite://:memory:".into(),
    }
}

static TEST_DIR_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

async fn authed_app() -> (axum::Router, String, sqlx::SqlitePool, std::path::PathBuf) {
    // Unique per test even with parallel tests on coarse Windows clocks.
    let seq = TEST_DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir =
        std::env::temp_dir().join(format!("logholizon-test-auth-{}-{seq}", std::process::id()));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    let db_path = dir.join("core.db");
    let url = format!("sqlite://{}", db_path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&url).await.unwrap();
    db::migrate(&pool).await.unwrap();
    logholizon_core::seed::seed(&pool).await.unwrap();
    logholizon_core::auth::register(&pool, "admin", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "admin", "password123")
        .await
        .unwrap();
    let config = Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: url,
    };
    let app = http::router(&config, pool.clone());
    (app, session.token, pool, dir)
}

async fn authed_user_app() -> (axum::Router, String) {
    let (app, _admin_token, pool, _dir) = authed_app().await;
    logholizon_core::auth::register(&pool, "alice", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "alice", "password123")
        .await
        .unwrap();
    (app, session.token)
}

#[tokio::test]
async fn admin_status_reports_counts() {
    let (app, token, _pool, _dir) = authed_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/admin/status")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["entities"], 2);
    assert_eq!(json["integrity"], true);
}

#[tokio::test]
async fn admin_restore_requires_force() {
    let (app, token, _pool, _dir) = authed_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/admin/restore")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(r#"{"path":"x.db","force":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn protected_routes_require_token() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    let app = http::router(&test_config(), pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/meta/entities")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn meta_routes_require_admin_role() {
    let (app, token) = authed_user_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/meta/entities")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn documents_routes_allow_regular_user() {
    let (app, token) = authed_user_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/documents?entity_id=work_order")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_returns_ok() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    let app = http::router(&test_config(), pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn version_returns_package_version() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    let app = http::router(&test_config(), pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "logholizon-core");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}
