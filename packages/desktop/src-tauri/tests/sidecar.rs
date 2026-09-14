//! Sidecar lifecycle test: boot the in-process core on an ephemeral port,
//! health-gate it, prove `/v1` auth works, then shut down cleanly.

use logholizon_desktop::{boot, probe_runtime, shutdown};

#[tokio::test]
async fn desktop_sidecar_boots_and_serves_v1() {
    let data_dir = std::env::temp_dir().join(format!(
        "logholizon-test-sidecar-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let runtime = boot(&data_dir).await.unwrap();
    assert!(runtime.addr.ip().is_loopback());

    let probe = probe_runtime(&runtime).await.unwrap();
    assert_eq!(probe["health"]["status"], "ok");
    // First-run setup created the admin user.
    assert_eq!(probe["auth_status"]["has_users"], true);

    // Direct /v1 login works against the sidecar (what the SPA will do).
    let client = reqwest::Client::new();
    let session: serde_json::Value = client
        .post(format!("{}/v1/auth/login", runtime.base_url()))
        .json(&serde_json::json!({ "username": "admin", "password": "admin12345" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(session["token"].as_str().is_some());

    shutdown(&runtime.pool).await.unwrap();
}
