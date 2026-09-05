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
