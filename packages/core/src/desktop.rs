//! Desktop boot helpers for the Tauri shell.
//!
//! The desktop app runs `logholizon-core` in-process on a loopback port and
//! keeps its SQLite database under the OS app-data directory. This module
//! contains the small, testable pieces of that boot sequence: resolving the
//! database URL from an app-data directory, building a desktop [`Config`],
//! and running the shared migrate + seed steps. The Tauri shell itself
//! (window, sidecar lifecycle) lives in `packages/desktop/src-tauri` and
//! calls into these helpers so workspace gates cover the logic without
//! needing WebKit system libraries.

use std::path::Path;

use crate::{auth, backup, db, seed, Config};

/// Tauri origins allowed to call the in-process core over HTTP.
///
/// The desktop frontend is served from the Tauri shell (custom protocol or
/// loopback), so the core must accept browser `Origin` headers from those
/// schemes. Everything else stays denied unless the operator extends
/// `CORE_ALLOWED_ORIGINS`.
pub const DESKTOP_ALLOWED_ORIGINS: &[&str] = &[
    "tauri://localhost",
    "https://tauri.localhost",
    "http://tauri.localhost",
    "http://localhost",
];

/// Resolve the desktop SQLite URL for an app-data directory.
///
/// Creates `<data_dir>/logholizon/core.db` (parent dirs included) and
/// returns it as a `sqlite://...?mode=rwc` URL. Reuses [`db::connect`]'s
/// parent-dir creation on connect as well; the explicit create here keeps
/// the path predictable for tests and first-run setup.
pub async fn desktop_database_url(data_dir: &Path) -> anyhow::Result<String> {
    let db_path = data_dir.join("logholizon").join("core.db");
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(format!(
        "sqlite://{}?mode=rwc",
        db_path.to_string_lossy().replace('\\', "/")
    ))
}

/// Build a desktop [`Config`] from a database URL.
///
/// Desktop keeps every default from [`Config::from_env`] except:
/// - `database_url` points at the app-data SQLite file;
/// - `allowed_origins` always includes [`DESKTOP_ALLOWED_ORIGINS`]
///   (plus any operator-provided `CORE_ALLOWED_ORIGINS` entries);
/// - scheduled backups and notify loops are disabled by default
///   (`backup_interval_hours = 0`, `notify_interval_secs = 0`) so a laptop
///   does not wake timers; explicit env values still win when set.
pub fn desktop_config(database_url: &str) -> Config {
    let mut config = Config::from_env();
    config.database_url = database_url.to_string();
    for origin in DESKTOP_ALLOWED_ORIGINS {
        if !config.allowed_origins.iter().any(|o| o == origin) {
            config.allowed_origins.push(origin.to_string());
        }
    }
    if std::env::var("CORE_BACKUP_INTERVAL_HOURS").is_err() {
        config.backup_interval_hours = 0;
    }
    if std::env::var("CORE_NOTIFY_INTERVAL_SECS").is_err() {
        config.notify_interval_secs = 0;
    }
    config
}

/// Boot the desktop database: staged restore, connect, migrate.
///
/// Mirrors the server boot in `packages/core/src/main.rs` minus the Axum
/// listener and background loops (the Tauri shell owns those). Returns the
/// connected pool ready for [`crate::http::router`].
pub async fn boot_desktop_pool(database_url: &str) -> anyhow::Result<sqlx::SqlitePool> {
    if backup::apply_staged_restore(database_url).await? {
        tracing::info!("desktop staged restore applied");
    }
    let pool = db::connect(database_url).await?;
    db::migrate(&pool).await?;
    Ok(pool)
}

/// First-run setup: ensure at least one admin exists, then load demo data.
///
/// Idempotent: when users already exist nothing is created. Returns `true`
/// when this call created the first admin (i.e. it was a first run).
pub async fn ensure_first_run(pool: &sqlx::SqlitePool) -> anyhow::Result<bool> {
    if auth::has_users(pool).await? {
        return Ok(false);
    }
    auth::register(pool, "admin", "admin12345").await?;
    seed::seed_demo(pool).await?;
    Ok(true)
}
