use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub async fn connect(url: &str) -> Result<SqlitePool> {
    let normalized = normalize_url(url).await?;
    // ponytail: in-memory SQLite needs a single connection; pooled
    // connections each get an isolated database. Use file DBs for concurrency tests.
    let max_connections = if normalized == "sqlite::memory:" {
        1
    } else {
        5
    };
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(&normalized)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
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
