use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub options: Vec<FieldOption>,
}

#[derive(Debug, Serialize)]
pub struct FieldOption {
    pub id: String,
    pub value: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct EntityDetail {
    pub id: String,
    pub name: String,
    pub label: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize)]
pub struct Document {
    pub id: String,
    pub entity_id: String,
    pub payload: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct DocumentList {
    pub items: Vec<Document>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub id: String,
    pub entity_id: String,
    pub doc_id: String,
    pub action: String,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuditList {
    pub items: Vec<AuditEntry>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowState {
    pub name: String,
    pub label: String,
    pub position: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowTransition {
    pub action: String,
    pub from_state: String,
    pub to_state: String,
}

#[derive(Debug, Serialize)]
pub struct WorkflowDefinition {
    pub states: Vec<WorkflowState>,
    pub transitions: Vec<WorkflowTransition>,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocument {
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocument {
    pub payload: Value,
}

pub async fn list_entities(pool: &SqlitePool) -> Result<Vec<Entity>> {
    let rows = sqlx::query("SELECT id, name, label FROM _meta_entity ORDER BY name")
        .fetch_all(pool)
        .await?;
    let mut entities = Vec::new();
    for row in rows {
        use sqlx::Row;
        entities.push(Entity {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            label: row.try_get("label")?,
        });
    }
    Ok(entities)
}

pub async fn create_entity(pool: &SqlitePool, id: &str, name: &str, label: &str) -> Result<Entity> {
    if id.trim().is_empty() || name.trim().is_empty() || label.trim().is_empty() {
        bail!("id, name, and label are required");
    }
    sqlx::query("INSERT INTO _meta_entity (id, name, label) VALUES (?, ?, ?)")
        .bind(id)
        .bind(name)
        .bind(label)
        .execute(pool)
        .await?;
    Ok(Entity {
        id: id.to_string(),
        name: name.to_string(),
        label: label.to_string(),
    })
}

pub async fn get_entity_detail(pool: &SqlitePool, entity_id: &str) -> Result<EntityDetail> {
    let entity = sqlx::query("SELECT id, name, label FROM _meta_entity WHERE id = ?")
        .bind(entity_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("entity not found: {entity_id}")))?;
    use sqlx::Row;
    let fields = list_fields(pool, entity_id).await?;
    Ok(EntityDetail {
        id: entity.try_get("id")?,
        name: entity.try_get("name")?,
        label: entity.try_get("label")?,
        fields,
    })
}

pub async fn list_fields(pool: &SqlitePool, entity_id: &str) -> Result<Vec<Field>> {
    let rows = sqlx::query(
        "SELECT id, name, type, required FROM _meta_field WHERE entity_id = ? ORDER BY name",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    let mut fields = Vec::new();
    for row in rows {
        use sqlx::Row;
        let field_id: String = row.try_get("id")?;
        let options = list_field_options(pool, &field_id).await?;
        fields.push(Field {
            id: field_id,
            name: row.try_get("name")?,
            r#type: row.try_get("type")?,
            required: row.try_get::<i64, _>("required")? != 0,
            options,
        });
    }
    Ok(fields)
}

pub async fn list_field_options(pool: &SqlitePool, field_id: &str) -> Result<Vec<FieldOption>> {
    let rows = sqlx::query(
        "SELECT id, value, label FROM _meta_field_option WHERE field_id = ? ORDER BY value",
    )
    .bind(field_id)
    .fetch_all(pool)
    .await?;
    let mut options = Vec::new();
    for row in rows {
        use sqlx::Row;
        options.push(FieldOption {
            id: row.try_get("id")?,
            value: row.try_get("value")?,
            label: row.try_get("label")?,
        });
    }
    Ok(options)
}

pub async fn create_document(
    pool: &SqlitePool,
    id: &str,
    entity_id: &str,
    payload: &Value,
) -> Result<Document> {
    if id.trim().is_empty() {
        bail!("id is required");
    }
    validate_payload(pool, entity_id, payload).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO _doc (id, entity_id, payload) VALUES (?, ?, ?)")
        .bind(id)
        .bind(entity_id)
        .bind(payload.to_string())
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload) VALUES (?, ?, ?, 'create', ?)",
    )
    .bind(audit_id(id, "create"))
    .bind(entity_id)
    .bind(id)
    .bind(payload.to_string())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

pub async fn get_document(pool: &SqlitePool, id: &str) -> Result<Document> {
    let row =
        sqlx::query("SELECT id, entity_id, payload, created_at, updated_at FROM _doc WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("document not found: {id}")))?;
    use sqlx::Row;
    Ok(Document {
        id: row.try_get("id")?,
        entity_id: row.try_get("entity_id")?,
        payload: serde_json::from_str(&row.try_get::<String, _>("payload")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn list_documents(
    pool: &SqlitePool,
    entity_id: &str,
    limit: i64,
    offset: i64,
) -> Result<DocumentList> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc WHERE entity_id = ?")
        .bind(entity_id)
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query(
        "SELECT id, entity_id, payload, created_at, updated_at FROM _doc WHERE entity_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(entity_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let mut items = Vec::new();
    for row in rows {
        use sqlx::Row;
        items.push(Document {
            id: row.try_get("id")?,
            entity_id: row.try_get("entity_id")?,
            payload: serde_json::from_str(&row.try_get::<String, _>("payload")?)?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }
    Ok(DocumentList { items, total })
}

pub async fn update_document(pool: &SqlitePool, id: &str, payload: &Value) -> Result<Document> {
    let existing = get_document(pool, id).await?;
    validate_payload(pool, &existing.entity_id, payload).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE _doc SET payload = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(payload.to_string())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload) VALUES (?, ?, ?, 'update', ?)",
    )
    .bind(audit_id(id, "update"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(payload.to_string())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

pub async fn transition_document(pool: &SqlitePool, id: &str, action: &str) -> Result<Document> {
    let existing = get_document(pool, id).await?;
    let current = existing
        .payload
        .get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("document has no status".into()))?;
    let target: Option<String> = sqlx::query_scalar(
        "SELECT to_state FROM _workflow_transition WHERE entity_id = ? AND from_state = ? AND action = ?",
    )
    .bind(&existing.entity_id)
    .bind(current)
    .bind(action)
    .fetch_optional(pool)
    .await?;
    let target = target.ok_or_else(|| {
        AppError::BadRequest(format!("invalid transition: {current} cannot {action}"))
    })?;
    let mut next = existing.payload;
    next["status"] = Value::String(target);
    validate_payload(pool, &existing.entity_id, &next).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE _doc SET payload = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(next.to_string())
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload) VALUES (?, ?, ?, 'transition', ?)",
    )
    .bind(audit_id(id, "transition"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(next.to_string())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

pub async fn get_workflow(pool: &SqlitePool, entity_id: &str) -> Result<WorkflowDefinition> {
    let states = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT name, label, position FROM _workflow_state WHERE entity_id = ? ORDER BY position",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(name, label, position)| WorkflowState {
        name,
        label,
        position,
    })
    .collect();
    let transitions = sqlx::query_as::<_, (String, String, String)>(
        "SELECT action, from_state, to_state FROM _workflow_transition WHERE entity_id = ? ORDER BY from_state, action",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(action, from_state, to_state)| WorkflowTransition { action, from_state, to_state })
    .collect();
    Ok(WorkflowDefinition {
        states,
        transitions,
    })
}

pub async fn count_documents_by_status(
    pool: &SqlitePool,
    entity_id: &str,
) -> Result<Vec<StatusCount>> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT json_extract(payload, '$.status'), COUNT(*) FROM _doc WHERE entity_id = ? GROUP BY json_extract(payload, '$.status') ORDER BY 1",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(status, count)| StatusCount { status, count })
        .collect())
}

pub async fn delete_document(pool: &SqlitePool, id: &str) -> Result<()> {
    let existing = get_document(pool, id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM _doc WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload) VALUES (?, ?, ?, 'delete', ?)",
    )
    .bind(audit_id(id, "delete"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(existing.payload.to_string())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_document_audit(
    pool: &SqlitePool,
    doc_id: &str,
    limit: i64,
    offset: i64,
) -> Result<AuditList> {
    if get_document(pool, doc_id).await.is_err() {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _audit_log WHERE doc_id = ?")
            .bind(doc_id)
            .fetch_one(pool)
            .await?;
        if count == 0 {
            return Err(AppError::NotFound(format!("document not found: {doc_id}")).into());
        }
    }
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _audit_log WHERE doc_id = ?")
        .bind(doc_id)
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query(
        "SELECT id, entity_id, doc_id, action, payload, created_at FROM _audit_log WHERE doc_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(doc_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let mut items = Vec::new();
    for row in rows {
        use sqlx::Row;
        items.push(AuditEntry {
            id: row.try_get("id")?,
            entity_id: row.try_get("entity_id")?,
            doc_id: row.try_get("doc_id")?,
            action: row.try_get("action")?,
            payload: serde_json::from_str(&row.try_get::<String, _>("payload")?)?,
            created_at: row.try_get("created_at")?,
        });
    }
    Ok(AuditList { items, total })
}

fn audit_id(doc_id: &str, action: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // ponytail: use ulid/uuid when ordering/content-addressing needed.
    format!("{doc_id}-{action}-{nanos}")
}

async fn validate_payload(pool: &SqlitePool, entity_id: &str, payload: &Value) -> Result<()> {
    let object = payload
        .as_object()
        .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".into()))?;
    let fields = list_fields(pool, entity_id).await?;
    for field in &fields {
        let value = object.get(&field.name);
        if field.required && value.is_none() {
            return Err(
                AppError::BadRequest(format!("missing required field: {}", field.name)).into(),
            );
        }
        if let Some(value) = value {
            validate_field_value(field, value)?;
        }
    }
    for key in object.keys() {
        if !fields.iter().any(|f| f.name == *key) {
            return Err(AppError::BadRequest(format!("unknown field: {key}")).into());
        }
    }
    Ok(())
}

fn validate_field_value(field: &Field, value: &Value) -> Result<()> {
    let valid = match field.r#type.as_str() {
        "text" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "date" => value.is_string(),
        "select" => {
            value.is_string()
                && field
                    .options
                    .iter()
                    .any(|o| o.value == value.as_str().unwrap())
        }
        _ => false,
    };
    if !valid {
        return Err(AppError::BadRequest(format!(
            "invalid value for field {}: expected {}",
            field.name, field.r#type
        ))
        .into());
    }
    Ok(())
}
