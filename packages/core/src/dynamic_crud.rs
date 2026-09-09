use anyhow::Result;
use serde_json::Value;
use sqlx::SqlitePool;

use crate::{error::AppError, repository};

pub async fn resolve_entity(
    pool: &SqlitePool,
    module: &str,
    entity: &str,
) -> Result<repository::Entity> {
    let module = module.trim();
    let entity = entity.trim();
    if module.is_empty() || entity.is_empty() {
        return Err(AppError::BadRequest("module and entity are required".into()).into());
    }
    let row = sqlx::query_as::<_, (String, String, String, String, String, Option<String>)>(
        "SELECT id, name, label, description, settings, module FROM _meta_entity WHERE name = ? AND module_id = ? LIMIT 1",
    )
    .bind(entity)
    .bind(module)
    .fetch_optional(pool)
    .await?;
    let Some((id, name, label, description, settings, module_name)) = row else {
        return Err(AppError::NotFound(format!("entity not found: {module}/{entity}")).into());
    };
    Ok(repository::Entity {
        id,
        name,
        label,
        description,
        settings: serde_json::from_str(&settings)?,
        module: module_name,
    })
}

pub fn parse_filter(value: Option<&str>) -> Result<Value> {
    let Some(raw) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(Value::Object(Default::default()));
    };
    let parsed: Value = serde_json::from_str(raw)
        .map_err(|e| AppError::BadRequest(format!("invalid filter JSON: {e}")))?;
    if !parsed.is_object() {
        return Err(AppError::BadRequest("filter must be a JSON object".into()).into());
    }
    Ok(parsed)
}

pub async fn bulk_delete(
    pool: &SqlitePool,
    entity_id: &str,
    ids: &[String],
    actor: Option<&str>,
    role: &str,
) -> Result<usize> {
    if ids.is_empty() || ids.len() > 100 {
        return Err(AppError::BadRequest("ids must contain 1..=100 items".into()).into());
    }
    repository::check_permission(pool, entity_id, role, true).await?;
    let mut tx = pool.begin().await?;
    let mut count = 0usize;
    for id in ids {
        let existing = sqlx::query_as::<_, (String, String)>(
            "SELECT id, payload FROM _doc WHERE id=? AND entity_id=?",
        )
        .bind(id)
        .bind(entity_id)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((doc_id, payload)) = existing else {
            continue;
        };
        sqlx::query("DELETE FROM _doc WHERE id=? AND entity_id=?")
            .bind(&doc_id)
            .bind(entity_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO _audit_log (id,entity_id,doc_id,action,payload,actor) VALUES (?,?,?,?,?,?)")
            .bind(format!("{}:bulk_delete", id))
            .bind(entity_id)
            .bind(id)
            .bind("delete")
            .bind(payload)
            .bind(actor)
            .execute(&mut *tx)
            .await?;
        count += 1;
    }
    tx.commit().await?;
    Ok(count)
}
