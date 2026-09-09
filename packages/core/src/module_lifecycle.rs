use anyhow::Result;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    repository::{self, Module},
};

pub async fn submit_module_for_review(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = repository::get_module(pool, id, owner, role).await?;
    if module.status != "draft" {
        return Err(AppError::Conflict(format!("module must be draft: {}", module.id)).into());
    }
    repository::validate_module_definition(&module.definition)?;
    sqlx::query(
        "UPDATE _module SET status = 'review', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&module.id)
    .execute(pool)
    .await?;
    repository::get_module(pool, id, owner, role).await
}

pub async fn enable_module(pool: &SqlitePool, id: &str, owner: &str, role: &str) -> Result<Module> {
    let module = repository::get_module(pool, id, owner, role).await?;
    if module.status != "published" {
        return Err(AppError::Conflict(format!("module must be published: {}", module.id)).into());
    }
    sqlx::query(
        "UPDATE _module SET status = 'enabled', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&module.id)
    .execute(pool)
    .await?;
    repository::get_module(pool, id, owner, role).await
}

pub async fn disable_module(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = repository::get_module(pool, id, owner, role).await?;
    if module.status != "enabled" {
        return Err(AppError::Conflict(format!("module must be enabled: {}", module.id)).into());
    }
    sqlx::query(
        "UPDATE _module SET status = 'disabled', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&module.id)
    .execute(pool)
    .await?;
    repository::get_module(pool, id, owner, role).await
}
