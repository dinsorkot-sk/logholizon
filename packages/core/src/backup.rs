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

pub async fn restore(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_file() {
        bail!("backup source does not exist: {}", source.display());
    }
    if source == destination {
        bail!("backup source and destination must differ");
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
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = destination.with_extension("restore.tmp");
    tokio::fs::copy(source, &temporary).await?;
    tokio::fs::rename(&temporary, destination).await?;
    Ok(())
}
