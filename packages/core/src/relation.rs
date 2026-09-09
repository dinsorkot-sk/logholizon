use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::AppError;

pub const RELATION_TYPES: &[&str] = &["one_to_one", "one_to_many", "many_to_one", "many_to_many"];
pub const DELETE_RULES: &[&str] = &["restrict", "set_null", "cascade"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub source_entity_id: String,
    pub source_field_id: Option<String>,
    pub target_entity_id: String,
    pub target_field_id: Option<String>,
    pub name: String,
    pub relation_type: String,
    pub on_delete: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationLink {
    pub relation_id: String,
    pub source_doc_id: String,
    pub target_doc_id: String,
}

fn validate_type(value: &str) -> Result<(), AppError> {
    if RELATION_TYPES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "unknown relation type: {value}"
        )))
    }
}

fn validate_delete(value: &str) -> Result<(), AppError> {
    if DELETE_RULES.contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "unknown on_delete rule: {value}"
        )))
    }
}

async fn entity_exists(pool: &SqlitePool, id: &str) -> Result<bool> {
    Ok(
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?,
    )
}

async fn field_entity(pool: &SqlitePool, id: &str) -> Result<Option<String>> {
    Ok(
        sqlx::query_scalar("SELECT entity_id FROM _meta_field WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn list_relations(pool: &SqlitePool, entity_id: &str) -> Result<Vec<Relation>> {
    if !entity_exists(pool, entity_id).await? {
        return Err(AppError::NotFound(format!("entity not found: {entity_id}")).into());
    }
    let rows = sqlx::query_as::<_, (String,String,Option<String>,String,Option<String>,String,String,String,String)>(
        "SELECT id, source_entity_id, source_field_id, target_entity_id, target_field_id, name, relation_type, on_delete, created_at FROM _meta_relation WHERE source_entity_id = ? OR target_entity_id = ? ORDER BY name"
    ).bind(entity_id).bind(entity_id).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| Relation {
            id: r.0,
            source_entity_id: r.1,
            source_field_id: r.2,
            target_entity_id: r.3,
            target_field_id: r.4,
            name: r.5,
            relation_type: r.6,
            on_delete: r.7,
            created_at: r.8,
        })
        .collect())
}

pub async fn get_relation(pool: &SqlitePool, id: &str) -> Result<Relation> {
    let row = sqlx::query_as::<_, (String,String,Option<String>,String,Option<String>,String,String,String,String)>(
        "SELECT id, source_entity_id, source_field_id, target_entity_id, target_field_id, name, relation_type, on_delete, created_at FROM _meta_relation WHERE id = ?"
    ).bind(id).fetch_optional(pool).await?.ok_or_else(|| AppError::NotFound(format!("relation not found: {id}")))?;
    Ok(Relation {
        id: row.0,
        source_entity_id: row.1,
        source_field_id: row.2,
        target_entity_id: row.3,
        target_field_id: row.4,
        name: row.5,
        relation_type: row.6,
        on_delete: row.7,
        created_at: row.8,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn create_relation(
    pool: &SqlitePool,
    source_entity_id: &str,
    source_field_id: Option<&str>,
    target_entity_id: &str,
    target_field_id: Option<&str>,
    name: &str,
    relation_type: &str,
    on_delete: &str,
) -> Result<Relation> {
    validate_type(relation_type)?;
    validate_delete(on_delete)?;
    if !entity_exists(pool, source_entity_id).await?
        || !entity_exists(pool, target_entity_id).await?
    {
        return Err(AppError::NotFound("source or target entity not found".into()).into());
    }
    if source_entity_id == target_entity_id
        && source_field_id.is_none()
        && relation_type != "many_to_many"
    {
        return Err(AppError::BadRequest("self-reference requires source_field_id".into()).into());
    }
    for (field, entity, label) in [
        (source_field_id, source_entity_id, "source"),
        (target_field_id, target_entity_id, "target"),
    ] {
        if let Some(field_id) = field {
            if field_entity(pool, field_id).await?.as_deref() != Some(entity) {
                return Err(AppError::BadRequest(format!(
                    "{label} field does not belong to entity"
                ))
                .into());
            }
        }
    }
    if relation_type != "many_to_many" && source_field_id.is_none() {
        return Err(AppError::BadRequest(
            "source_field_id is required for non many-to-many relations".into(),
        )
        .into());
    }
    let id = format!(
        "rel_{}_{}",
        source_entity_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    sqlx::query("INSERT INTO _meta_relation (id,source_entity_id,source_field_id,target_entity_id,target_field_id,name,relation_type,on_delete) VALUES (?,?,?,?,?,?,?,?)")
        .bind(&id).bind(source_entity_id).bind(source_field_id).bind(target_entity_id).bind(target_field_id).bind(name.trim()).bind(relation_type).bind(on_delete).execute(pool).await?;
    get_relation(pool, &id).await
}

pub async fn update_relation(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    relation_type: &str,
    on_delete: &str,
    source_field_id: Option<&str>,
    target_field_id: Option<&str>,
) -> Result<Relation> {
    let old = get_relation(pool, id).await?;
    validate_type(relation_type)?;
    validate_delete(on_delete)?;
    if let Some(f) = source_field_id {
        if field_entity(pool, f).await?.as_deref() != Some(old.source_entity_id.as_str()) {
            return Err(AppError::BadRequest(
                "source field does not belong to source entity".into(),
            )
            .into());
        }
    }
    if let Some(f) = target_field_id {
        if field_entity(pool, f).await?.as_deref() != Some(old.target_entity_id.as_str()) {
            return Err(AppError::BadRequest(
                "target field does not belong to target entity".into(),
            )
            .into());
        }
    }
    if relation_type != "many_to_many" && source_field_id.is_none() {
        return Err(AppError::BadRequest(
            "source_field_id is required for non many-to-many relations".into(),
        )
        .into());
    }
    sqlx::query("UPDATE _meta_relation SET name=?,relation_type=?,on_delete=?,source_field_id=?,target_field_id=? WHERE id=?").bind(name.trim()).bind(relation_type).bind(on_delete).bind(source_field_id).bind(target_field_id).bind(id).execute(pool).await?;
    get_relation(pool, id).await
}

pub async fn delete_relation(pool: &SqlitePool, id: &str) -> Result<()> {
    let r = sqlx::query("DELETE FROM _meta_relation WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("relation not found: {id}")).into());
    }
    Ok(())
}

pub async fn list_links(
    pool: &SqlitePool,
    relation_id: &str,
    source_doc_id: &str,
) -> Result<Vec<RelationLink>> {
    let _ = get_relation(pool, relation_id).await?;
    let rows=sqlx::query_as::<_,(String,String,String)>("SELECT relation_id,source_doc_id,target_doc_id FROM _meta_relation_link WHERE relation_id=? AND source_doc_id=? ORDER BY target_doc_id").bind(relation_id).bind(source_doc_id).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| RelationLink {
            relation_id: r.0,
            source_doc_id: r.1,
            target_doc_id: r.2,
        })
        .collect())
}

pub async fn set_links(
    pool: &SqlitePool,
    relation_id: &str,
    source_doc_id: &str,
    target_doc_ids: &[String],
) -> Result<Vec<RelationLink>> {
    let relation = get_relation(pool, relation_id).await?;
    if relation.relation_type != "many_to_many" {
        return Err(
            AppError::BadRequest("links are only valid for many_to_many relations".into()).into(),
        );
    }
    for target in target_doc_ids {
        let ok: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id=? AND entity_id=?)")
                .bind(target)
                .bind(&relation.target_entity_id)
                .fetch_one(pool)
                .await?;
        if !ok {
            return Err(
                AppError::BadRequest(format!("target document not found: {target}")).into(),
            );
        }
    }
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM _meta_relation_link WHERE relation_id=? AND source_doc_id=?")
        .bind(relation_id)
        .bind(source_doc_id)
        .execute(&mut *tx)
        .await?;
    for target in target_doc_ids {
        sqlx::query("INSERT INTO _meta_relation_link(relation_id,source_doc_id,target_doc_id) VALUES(?,?,?)").bind(relation_id).bind(source_doc_id).bind(target).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    list_links(pool, relation_id, source_doc_id).await
}

pub async fn related_documents(
    pool: &SqlitePool,
    relation_id: &str,
    doc_id: &str,
) -> Result<Vec<serde_json::Value>> {
    let r = get_relation(pool, relation_id).await?;
    let source_ok: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id=? AND entity_id=?)")
            .bind(doc_id)
            .bind(&r.source_entity_id)
            .fetch_one(pool)
            .await?;
    let target_ok: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id=? AND entity_id=?)")
            .bind(doc_id)
            .bind(&r.target_entity_id)
            .fetch_one(pool)
            .await?;
    if !source_ok && !target_ok {
        return Err(
            AppError::NotFound(format!("document not found for relation: {doc_id}")).into(),
        );
    }
    let mut ids = Vec::<String>::new();
    if r.relation_type == "many_to_many" {
        if source_ok {
            ids = sqlx::query_scalar("SELECT target_doc_id FROM _meta_relation_link WHERE relation_id=? AND source_doc_id=? ORDER BY target_doc_id").bind(relation_id).bind(doc_id).fetch_all(pool).await?;
        } else {
            ids = sqlx::query_scalar("SELECT source_doc_id FROM _meta_relation_link WHERE relation_id=? AND target_doc_id=? ORDER BY source_doc_id").bind(relation_id).bind(doc_id).fetch_all(pool).await?;
        }
    } else if let Some(field) = &r.source_field_id {
        let name: String = sqlx::query_scalar("SELECT name FROM _meta_field WHERE id=?")
            .bind(field)
            .fetch_one(pool)
            .await?;
        if source_ok {
            let payload: String = sqlx::query_scalar("SELECT payload FROM _doc WHERE id=?")
                .bind(doc_id)
                .fetch_one(pool)
                .await?;
            let v: serde_json::Value = serde_json::from_str(&payload)?;
            if let Some(id) = v.get(&name).and_then(|v| v.as_str()) {
                ids.push(id.to_string());
            }
        } else {
            let rows = sqlx::query_as::<_, (String, String)>(
                "SELECT id,payload FROM _doc WHERE entity_id=? ORDER BY created_at DESC LIMIT 5000",
            )
            .bind(&r.source_entity_id)
            .fetch_all(pool)
            .await?;
            for (id, payload) in rows {
                let v: serde_json::Value = serde_json::from_str(&payload)?;
                if v.get(&name).and_then(|v| v.as_str()) == Some(doc_id) {
                    ids.push(id);
                }
            }
        }
    }
    let mut out = Vec::new();
    for id in ids {
        if let Some((doc_id, payload)) =
            sqlx::query_as::<_, (String, String)>("SELECT id,payload FROM _doc WHERE id=?")
                .bind(id)
                .fetch_optional(pool)
                .await?
        {
            let mut v: serde_json::Value = serde_json::from_str(&payload)?;
            if let Some(o) = v.as_object_mut() {
                o.insert("id".into(), serde_json::Value::String(doc_id));
            }
            out.push(v);
        }
    }
    Ok(out)
}
