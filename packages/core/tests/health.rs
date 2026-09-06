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

#[tokio::test]
async fn admin_status_reports_counts() {
    let dir = std::env::temp_dir().join(format!(
        "logholizon-test-status-{}",
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
    let config = Config {
        host: "127.0.0.1".into(),
        port: 0,
        database_url: url,
    };
    let app = http::router(&config, pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/admin/status")
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
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    let app = http::router(&test_config(), pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/admin/restore")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"path":"x.db","force":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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
