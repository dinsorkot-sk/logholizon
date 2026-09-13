use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

/// SQLite single-host concurrency policy (Phase C).
///
/// - WAL journal mode: readers never block writers; writers never block readers.
/// - `busy_timeout` 5000ms: a writer waits for the lock instead of failing fast.
/// - `synchronous = NORMAL`: durability balanced for single-host WAL
///   (checkpoints are crash-safe; a power loss may lose the last transaction).
/// - `foreign_keys = ON`: relational integrity enforced on every connection.
/// - Pool size 5 (file) / 1 (memory): SQLite has a single writer, so
///   concurrent writes serialize. A small pool plus `busy_timeout` absorbs
///   bursts without `SQLITE_BUSY` errors. In-memory DBs use one connection
///   because pooled connections would each get an isolated database.
///
/// Multi-instance deployments require a server database (see the Phase C
/// Postgres readiness doc). The in-process auth rate limiter (Phase B) has
/// the same single-host scope.
pub const MAX_CONNECTIONS_FILE: usize = 5;
pub const MAX_CONNECTIONS_MEMORY: usize = 1;
pub const BUSY_TIMEOUT_MS: i64 = 5000;

pub async fn connect(url: &str) -> Result<SqlitePool> {
    let normalized = normalize_url(url).await?;
    // In-memory SQLite needs a single connection; pooled connections each
    // get an isolated database. Use file DBs for concurrency tests.
    let max_connections = if normalized == "sqlite::memory:" {
        MAX_CONNECTIONS_MEMORY
    } else {
        MAX_CONNECTIONS_FILE
    };
    // Pragmas are per-connection, so apply them on every connection the
    // pool opens (not just the first one) to enforce the policy uniformly.
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections as u32)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA journal_mode = WAL")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query(&format!("PRAGMA busy_timeout = {BUSY_TIMEOUT_MS}"))
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA synchronous = NORMAL")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&normalized)
        .await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

async fn normalize_url(url: &str) -> Result<String> {
    if url == "sqlite::memory:" || url == "sqlite://:memory:" {
        return Ok("sqlite::memory:".to_string());
    }
    if let Some(path) = url.strip_prefix("sqlite://") {
        let (path, query) = match path.split_once('?') {
            Some((p, q)) => (p, format!("?{q}")),
            None => (path, "?mode=rwc".to_string()),
        };
        if !path.is_empty() {
            if let Some(parent) = Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }
        }
        return Ok(format!("sqlite://{path}{query}"));
    }
    Ok(url.to_string())
}

pub fn database_path(url: &str) -> Result<&Path> {
    let path = url
        .strip_prefix("sqlite://")
        .ok_or_else(|| anyhow::anyhow!("database URL must start with sqlite://"))?
        .split('?')
        .next()
        .unwrap();
    if path.is_empty() || path == ":memory:" {
        anyhow::bail!("restore requires a file-backed SQLite database");
    }
    Ok(Path::new(path))
}

pub async fn integrity_check(pool: &SqlitePool) -> Result<bool> {
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await?;
    Ok(result == "ok")
}

/// Snapshot of the live SQLite pragmas for the `/ready` probe.
/// Lets operators verify WAL/busy_timeout/synchronous/foreign_keys
/// without shelling into the container.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PragmaSnapshot {
    pub journal_mode: String,
    pub busy_timeout_ms: i64,
    pub synchronous: String,
    pub foreign_keys: bool,
    /// Currently open pool connections (informational; the configured cap
    /// is [`MAX_CONNECTIONS_FILE`] / [`MAX_CONNECTIONS_MEMORY`]).
    pub open_connections: usize,
}

pub async fn pragma_snapshot(pool: &SqlitePool) -> Result<PragmaSnapshot> {
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(pool)
        .await?;
    let busy_timeout_ms: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
        .fetch_one(pool)
        .await?;
    // PRAGMA synchronous returns an integer code (0 OFF, 1 NORMAL, 2 FULL).
    let synchronous_code: i64 = sqlx::query_scalar("PRAGMA synchronous")
        .fetch_one(pool)
        .await?;
    let synchronous = match synchronous_code {
        0 => "OFF",
        1 => "NORMAL",
        2 => "FULL",
        3 => "EXTRA",
        other => return Err(anyhow::anyhow!("unknown synchronous mode: {other}")),
    }
    .to_string();
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(pool)
        .await?;
    Ok(PragmaSnapshot {
        journal_mode: journal_mode.to_uppercase(),
        busy_timeout_ms,
        synchronous,
        foreign_keys: foreign_keys != 0,
        open_connections: pool.size() as usize,
    })
}
