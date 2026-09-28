//! Stress and load tests for the Axum Core API.
//!
//! These tests measure concurrent throughput, p95/p99 latency, and SQLite WAL
//! lock behaviour under realistic multi-tenant load. They exercise the same
//! `axum::Router` used in production via `tower::ServiceExt::oneshot`, so the
//! full middleware stack (auth, rate limiting, request logging) is included.
//!
//! Run with: `cargo test -p logholizon-core --test stress -- --nocapture`

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use logholizon_core::{db, http, Config};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
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
        allowed_origins: Vec::new(),
        auth_rate_limit_max_attempts: 10_000,
        auth_rate_limit_window_secs: 60,
    }
}

async fn setup_app() -> (axum::Router, String) {
    let dir = std::env::temp_dir().join(format!(
        "logholizon-stress-{}",
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
    logholizon_core::auth::register(&pool, "stress_user", "password123")
        .await
        .unwrap();
    let session = logholizon_core::auth::login(&pool, "stress_user", "password123")
        .await
        .unwrap();
    let app = http::router(&test_config(&url), pool);
    (app, session.token)
}

async fn call(
    app: &axum::Router,
    token: &str,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let request = builder
        .body(
            body.map(|b| Body::from(b.to_string()))
                .unwrap_or_else(Body::empty),
        )
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
                 "options":[{"value":"available","label":"Available"},{"value":"rented","label":"Rented"}]}
            ]}
        ]
    })
}

/// Publish a fresh module and return its id, ready for the entity runtime.
async fn publish_module(app: &axum::Router, token: &str, name: &str) -> String {
    let (status, created) = call(
        app,
        token,
        "POST",
        "/v1/modules",
        Some(json!({"name": name, "label": name, "definition": vehicle_definition()})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create module: {created:?}");
    let module_id = created["id"].as_str().unwrap().to_string();

    for action in ["review", "publish", "enable"] {
        let (status, body) = call(
            app,
            token,
            "POST",
            &format!("/v1/modules/{module_id}/{action}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{action}: {body:?}");
    }
    module_id
}

/// Compute a percentile from a slice of durations (sorts a clone internally).
fn percentile(durations: &[Duration], pct: f64) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }
    let mut sorted = durations.to_vec();
    sorted.sort();
    let idx = ((sorted.len() as f64 * pct) as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Test 1: Concurrent health checks (baseline throughput + middleware cost).
#[tokio::test]
async fn stress_concurrent_health_checks() {
    let (app, token) = setup_app().await;
    let mut handles = vec![];

    let start = Instant::now();
    for _ in 0..100 {
        let app_clone = app.clone();
        let token_clone = token.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, _) = call(&app_clone, &token_clone, "GET", "/health", None).await;
            (status, req_start.elapsed())
        }));
    }

    let mut durations = vec![];
    for handle in handles {
        let (status, duration) = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
        durations.push(duration);
    }

    let total = start.elapsed();
    let p50 = percentile(&durations, 0.50);
    let p95 = percentile(&durations, 0.95);
    let p99 = percentile(&durations, 0.99);

    println!("Health check stress test:");
    println!("  Total time: {total:?}");
    println!("  p50: {p50:?}  p95: {p95:?}  p99: {p99:?}");
    println!("  Requests/sec: {:.2}", 100.0 / total.as_secs_f64());

    // Debug (unoptimized) builds are noticeably slower than release; the
    // threshold here guards against pathological regressions (e.g. an
    // accidental blocking call or N+1 query), not micro-latency.
    assert!(
        p95 < Duration::from_millis(750),
        "health p95 latency too high: {p95:?}"
    );
}

/// Test 2: Concurrent document writes.
/// Exercises SQLite WAL behaviour under concurrent inserts.
#[tokio::test]
async fn stress_concurrent_document_writes() {
    let (app, token) = setup_app().await;
    let module_id = publish_module(&app, &token, "stress_writes").await;

    const WRITES: usize = 50;
    let mut handles = vec![];
    let start = Instant::now();

    for i in 0..WRITES {
        let app_clone = app.clone();
        let token_clone = token.clone();
        let module_id_clone = module_id.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, resp) = call(
                &app_clone,
                &token_clone,
                "POST",
                &format!("/v1/modules/{module_id_clone}/entities/vehicle"),
                Some(json!({
                    "id": format!("veh-{i}"),
                    "payload": {"code": format!("V{i}"), "plate_number": format!("AB-{i}"), "status": "available"}
                })),
            )
            .await;
            (status, resp, req_start.elapsed())
        }));
    }

    let mut durations = vec![];
    let mut success = 0;
    for handle in handles {
        let (status, resp, duration) = handle.await.unwrap();
        if status == StatusCode::CREATED {
            success += 1;
            durations.push(duration);
        } else {
            eprintln!("write failed: {status} {resp:?}");
        }
    }

    let total = start.elapsed();
    let p50 = percentile(&durations, 0.50);
    let p95 = percentile(&durations, 0.95);

    println!("Document write stress test:");
    println!("  Total time: {total:?}");
    println!("  Success: {success}/{WRITES}");
    println!("  p50: {p50:?}  p95: {p95:?}");
    println!("  Writes/sec: {:.2}", success as f64 / total.as_secs_f64());

    assert_eq!(success, WRITES, "all concurrent writes must succeed");
    assert!(
        p95 < Duration::from_millis(2000),
        "write p95 latency too high: {p95:?}"
    );
}

/// Test 3: Concurrent entity listing (read throughput).
#[tokio::test]
async fn stress_concurrent_entity_listing() {
    let (app, token) = setup_app().await;
    let module_id = publish_module(&app, &token, "stress_reads").await;

    // Seed a handful of documents so reads have real data.
    for i in 0..20 {
        let (status, _) = call(
            &app,
            &token,
            "POST",
            &format!("/v1/modules/{module_id}/entities/vehicle"),
            Some(json!({
                "id": format!("seed-{i}"),
                "payload": {"code": format!("S{i}"), "plate_number": format!("S-{i}"), "status": "available"}
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    const READS: usize = 200;
    let uri = format!("/v1/modules/{module_id}/entities/vehicle");
    let mut handles = vec![];
    let start = Instant::now();

    for _ in 0..READS {
        let app_clone = app.clone();
        let token_clone = token.clone();
        let uri_clone = uri.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, _) = call(&app_clone, &token_clone, "GET", &uri_clone, None).await;
            (status, req_start.elapsed())
        }));
    }

    let mut durations = vec![];
    for handle in handles {
        let (status, duration) = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
        durations.push(duration);
    }

    let total = start.elapsed();
    let p50 = percentile(&durations, 0.50);
    let p95 = percentile(&durations, 0.95);
    let p99 = percentile(&durations, 0.99);

    println!("Entity listing stress test:");
    println!("  Total time: {total:?}");
    println!("  p50: {p50:?}  p95: {p95:?}  p99: {p99:?}");
    println!("  Reads/sec: {:.2}", READS as f64 / total.as_secs_f64());

    // 200 concurrent tasks all spawned at t=0 will queue on the SQLite
    // connection pool; in unoptimized debug builds total run time is ~1.3s.
    assert!(
        p95 < Duration::from_millis(2500),
        "read p95 latency too high: {p95:?}"
    );
}

/// Test 4: Mixed read/write workload — validates WAL allows concurrent
/// readers while a writer holds the write lock (no SQLITE_BUSY failures).
#[tokio::test]
async fn stress_mixed_read_write_workload() {
    let (app, token) = setup_app().await;
    let module_id = publish_module(&app, &token, "stress_mixed").await;

    const WRITES: usize = 20;
    const READS: usize = 30;
    let uri = format!("/v1/modules/{module_id}/entities/vehicle");

    let mut handles = vec![];
    let start = Instant::now();

    for i in 0..WRITES {
        let app_clone = app.clone();
        let token_clone = token.clone();
        let uri_clone = uri.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, _) = call(
                &app_clone,
                &token_clone,
                "POST",
                &uri_clone,
                Some(json!({
                    "id": format!("m-{i}"),
                    "payload": {"code": format!("M{i}"), "plate_number": format!("M-{i}"), "status": "available"}
                })),
            )
            .await;
            (status, true, req_start.elapsed())
        }));
    }
    for _ in 0..READS {
        let app_clone = app.clone();
        let token_clone = token.clone();
        let uri_clone = uri.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, _) = call(&app_clone, &token_clone, "GET", &uri_clone, None).await;
            (status, false, req_start.elapsed())
        }));
    }

    let mut read_durations = vec![];
    let mut write_durations = vec![];
    let mut read_success = 0;
    let mut write_success = 0;

    for handle in handles {
        let (status, is_write, duration) = handle.await.unwrap();
        if status.is_success() {
            if is_write {
                write_success += 1;
                write_durations.push(duration);
            } else {
                read_success += 1;
                read_durations.push(duration);
            }
        }
    }

    let total = start.elapsed();
    let read_p95 = percentile(&read_durations, 0.95);
    let write_p95 = percentile(&write_durations, 0.95);

    println!("Mixed read/write stress test:");
    println!("  Total time: {total:?}");
    println!("  Reads: {read_success}/{READS} ok, p95: {read_p95:?}");
    println!("  Writes: {write_success}/{WRITES} ok, p95: {write_p95:?}");

    assert_eq!(
        read_success, READS,
        "all reads should succeed under write load"
    );
    assert_eq!(write_success, WRITES, "all writes should succeed");
    assert!(
        read_p95 < Duration::from_millis(750),
        "mixed read p95 too high: {read_p95:?}"
    );
    assert!(
        write_p95 < Duration::from_millis(2000),
        "mixed write p95 too high: {write_p95:?}"
    );
}

/// Test 5: Connection pool saturation.
/// Verifies the pool queues rather than fails under heavy concurrency.
#[tokio::test]
async fn stress_connection_pool_saturation() {
    let (app, token) = setup_app().await;

    const REQUESTS: usize = 500;
    let mut handles = vec![];
    let start = Instant::now();

    for _ in 0..REQUESTS {
        let app_clone = app.clone();
        let token_clone = token.clone();
        handles.push(tokio::spawn(async move {
            let req_start = Instant::now();
            let (status, _) = call(&app_clone, &token_clone, "GET", "/health", None).await;
            (status, req_start.elapsed())
        }));
    }

    let mut durations = vec![];
    let mut success = 0;
    for handle in handles {
        let (status, duration) = handle.await.unwrap();
        if status == StatusCode::OK {
            success += 1;
            durations.push(duration);
        }
    }

    let total = start.elapsed();
    let p50 = percentile(&durations, 0.50);
    let p95 = percentile(&durations, 0.95);
    let p99 = percentile(&durations, 0.99);

    println!("Connection pool saturation test:");
    println!("  Total time: {total:?}");
    println!("  Success: {success}/{REQUESTS}");
    println!("  p50: {p50:?}  p95: {p95:?}  p99: {p99:?}");
    println!(
        "  Requests/sec: {:.2}",
        success as f64 / total.as_secs_f64()
    );

    assert_eq!(
        success, REQUESTS,
        "all requests should succeed under saturation"
    );
    assert!(
        p95 < Duration::from_millis(2000),
        "p95 under saturation too high: {p95:?}"
    );
}
