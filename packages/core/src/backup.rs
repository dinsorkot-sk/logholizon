use anyhow::{bail, Result};
use sqlx::SqlitePool;
use std::path::Path;

pub async fn backup(pool: &SqlitePool, destination: &Path) -> Result<()> {
    if destination.exists() {
        bail!(
            "backup destination already exists: {}",
            destination.display()
        );
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let path = destination.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{path}'"))
        .execute(pool)
        .await?;
    Ok(())
}

/// Validate that a file is a readable, integrity-clean SQLite database.
pub async fn validate(source: &Path) -> Result<()> {
    if !source.is_file() {
        bail!("backup source does not exist: {}", source.display());
    }
    let source_url = format!("sqlite://{}?mode=ro", source.to_string_lossy());
    let source_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&source_url)
        .await?;
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&source_pool)
        .await?;
    source_pool.close().await;
    if result != "ok" {
        bail!("backup integrity check failed");
    }
    Ok(())
}

pub async fn restore(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_file() {
        bail!("backup source does not exist: {}", source.display());
    }
    if source == destination {
        bail!("backup source and destination must differ");
    }
    validate(source).await?;
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = destination.with_extension("restore.tmp");
    tokio::fs::copy(source, &temporary).await?;
    tokio::fs::rename(&temporary, destination).await?;
    Ok(())
}

/// Apply a staged restore file (if present) before the pool connects.
/// Returns true when a restore was applied.
pub async fn apply_staged_restore(database_url: &str) -> Result<bool> {
    let destination = crate::db::database_path(database_url)?;
    let staging = destination
        .parent()
        .map(|p| p.join("restore-pending.db"))
        .unwrap_or_else(|| Path::new("restore-pending.db").to_path_buf());
    if !staging.is_file() {
        return Ok(false);
    }
    restore(&staging, destination).await?;
    tokio::fs::remove_file(&staging).await?;
    Ok(true)
}
