//! Tauri shell lifecycle for the LOGHOLIZON desktop app.
//!
//! The shell boots `logholizon-core` in-process on an ephemeral loopback
//! port, health-gates `/health`, then hands the base URL to the frontend.
//! Shutdown stops the server and checkpoints the SQLite WAL so no data is
//! lost when the window closes.
//!
//! Native window code (`tauri::Builder`) lives behind the `ui` feature so
//! `cargo test` / `clippy` run on machines without WebKit system libraries.
//! The bootable core logic here is feature-independent and covered by tests.

use std::net::SocketAddr;
use std::path::PathBuf;

use logholizon_core::{db, desktop, http, notification, notify, Config};

/// Resolved desktop runtime: config, pool, and the loopback address.
pub struct DesktopRuntime {
    pub config: Config,
    pub pool: sqlx::SqlitePool,
    pub addr: SocketAddr,
}

impl DesktopRuntime {
    /// Base URL the frontend uses for direct `/v1` calls, e.g.
    /// `http://127.0.0.1:54321`.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

/// Resolve the app-data directory for the desktop database.
///
/// Prefers the platform data dir (`$XDG_DATA_HOME` / `%APPDATA%` /
/// `~/Library/Application Support`), falling back to the current dir so
/// tests and portable runs keep working.
pub fn app_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        return dir.join("logholizon-desktop");
    }
    #[cfg(target_os = "windows")]
    if let Some(dir) = std::env::var_os("APPDATA").map(PathBuf::from) {
        return dir.join("logholizon-desktop");
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        return home
            .join("Library")
            .join("Application Support")
            .join("logholizon-desktop");
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        return home.join(".local").join("share").join("logholizon-desktop");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Boot the in-process core on an ephemeral loopback port.
///
/// Steps: resolve DB URL under `data_dir`, build the desktop config,
/// boot the pool (staged restore + migrate), ensure first-run admin/demo
/// data, bind `127.0.0.1:0`, and serve the shared Axum router. Background
/// backup/notify/automation loops only spawn when the config enables them
/// (desktop disables them by default; explicit env still wins).
pub async fn boot(data_dir: &std::path::Path) -> anyhow::Result<DesktopRuntime> {
    let database_url = desktop::desktop_database_url(data_dir).await?;
    let mut config = desktop::desktop_config(&database_url);
    config.host = "127.0.0.1".to_string();
    config.port = 0;

    let pool = desktop::boot_desktop_pool(&config.database_url).await?;
    let first_run = desktop::ensure_first_run(&pool).await?;
    if first_run {
        tracing::info!("desktop first run: admin + demo data created");
    }

    spawn_background_loops(&config, &pool);

    let app = http::router(&config, pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tracing::info!("logholizon-desktop core listening on {addr}");
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::warn!("desktop core server stopped: {error:#}");
        }
    });

    Ok(DesktopRuntime { config, pool, addr })
}

/// Wait until the sidecar answers `/health` (or the timeout elapses).
pub async fn wait_for_health(base_url: &str, timeout: std::time::Duration) -> anyhow::Result<()> {
    let client = reqwest_client();
    let deadline = std::time::Instant::now() + timeout;
    let url = format!("{base_url}/health");
    loop {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            _ => {
                if std::time::Instant::now() >= deadline {
                    anyhow::bail!("timed out waiting for {url}");
                }
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        }
    }
}

/// Checkpoint the WAL and close the pool so desktop shutdown loses nothing.
pub async fn shutdown(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await;
    pool.close().await;
    Ok(())
}

fn spawn_background_loops(config: &Config, pool: &sqlx::SqlitePool) {
    if config.backup_interval_hours > 0 {
        let task_pool = pool.clone();
        let task_url = config.database_url.clone();
        let interval = std::time::Duration::from_secs(config.backup_interval_hours * 3600);
        let keep = config.backup_keep;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match logholizon_core::backup::scheduled_backup(&task_pool, &task_url, keep).await {
                    Ok(path) => tracing::info!("desktop backup: {}", path.display()),
                    Err(error) => tracing::warn!("desktop backup failed: {error:#}"),
                }
            }
        });
    }
    if config.notify_interval_secs > 0 {
        let task_pool = pool.clone();
        let interval = std::time::Duration::from_secs(config.notify_interval_secs);
        let timeout_secs = config.notify_timeout_secs;
        let max_attempts = config.notify_max_attempts;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match notify::deliver_pending(&task_pool, timeout_secs, max_attempts).await {
                    Ok(0) => {}
                    Ok(delivered) => tracing::info!("desktop deliveries: {delivered}"),
                    Err(error) => tracing::warn!("desktop delivery failed: {error:#}"),
                }
                let _ = logholizon_core::automation::enqueue_events(&task_pool).await;
                let _ = logholizon_core::automation::enqueue_scheduled(&task_pool).await;
                let _ = logholizon_core::automation::process_pending(&task_pool).await;
                let _ = notification::deliver_pending(&task_pool).await;
                let _ = notification::deliver_notifications(&task_pool).await;
            }
        });
    }
}

fn reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Verify the booted runtime answers `/health` and serves `/v1` auth status.
pub async fn probe_runtime(runtime: &DesktopRuntime) -> anyhow::Result<serde_json::Value> {
    let client = reqwest_client();
    let health: serde_json::Value = client
        .get(format!("{}/health", runtime.base_url()))
        .send()
        .await?
        .json()
        .await?;
    let status: serde_json::Value = client
        .get(format!("{}/v1/auth/status", runtime.base_url()))
        .send()
        .await?
        .json()
        .await?;
    assert!(db::integrity_check(&runtime.pool).await?);
    Ok(serde_json::json!({ "health": health, "auth_status": status }))
}
