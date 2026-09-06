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

/// Directory holding `core-<timestamp>.db` backups next to the database file.
pub fn backups_dir_for(database_url: &str) -> Result<std::path::PathBuf> {
    let db_path = crate::db::database_path(database_url)?;
    Ok(db_path
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| Path::new("backups").to_path_buf()))
}

/// Remove oldest `core-*.db` files so at most `keep` remain.
/// Returns the number of files removed.
pub async fn prune_backups(dir: &Path, keep: usize) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut names: Vec<String> = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("core-") && name.ends_with(".db") {
            names.push(name.to_string());
        }
    }
    names.sort();
    if names.len() <= keep {
        return Ok(0);
    }
    let remove = names.len() - keep;
    for name in names.iter().take(remove) {
        tokio::fs::remove_file(dir.join(name)).await?;
    }
    Ok(remove)
}

/// Create a timestamped backup and prune old ones. Returns the new file path.
pub async fn scheduled_backup(
    pool: &SqlitePool,
    database_url: &str,
    keep: usize,
) -> Result<std::path::PathBuf> {
    let dir = backups_dir_for(database_url)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let destination = dir.join(format!("core-{timestamp}.db"));
    backup(pool, &destination).await?;
    prune_backups(&dir, keep).await?;
    Ok(destination)
}
