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
    pub is_status: bool,
    pub position: i64,
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
pub struct EntityWithPermission {
    pub id: String,
    pub name: String,
    pub label: String,
    pub fields: Vec<FieldWithPermission>,
    pub permission: EntityPermission,
}

#[derive(Debug, Serialize)]
pub struct FieldWithPermission {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub is_status: bool,
    pub position: i64,
    pub options: Vec<FieldOption>,
    pub can_view: bool,
    pub can_edit: bool,
}

#[derive(Debug, Serialize)]
pub struct FieldPermission {
    pub field_id: String,
    pub role: String,
    pub can_view: bool,
    pub can_edit: bool,
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
    pub actor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditList {
    pub items: Vec<AuditEntry>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct GlobalAuditEntry {
    pub id: String,
    pub entity_id: String,
    pub entity_label: String,
    pub doc_id: String,
    pub action: String,
    pub payload: Value,
    pub created_at: String,
    pub actor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GlobalAuditList {
    pub items: Vec<GlobalAuditEntry>,
    pub total: i64,
}

#[derive(Debug, Default, Clone)]
pub struct GlobalAuditFilter {
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    pub label: String,
    pub position: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowTransition {
    pub id: String,
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
pub struct EntityPermission {
    pub role: String,
    pub can_view: bool,
    pub can_edit: bool,
}

#[derive(Debug, Serialize)]
pub struct EntityView {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub config: Value,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct FormLayout {
    pub entity_id: String,
    pub config: Value,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PmSummary {
    pub open: i64,
    pub overdue: i64,
    pub done_this_week: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocument {
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocument {
    pub payload: Value,
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

pub async fn list_entities(pool: &SqlitePool) -> Result<Vec<Entity>> {
    list_entities_for_role(pool, "admin").await
}

pub async fn list_entities_for_role(pool: &SqlitePool, role: &str) -> Result<Vec<Entity>> {
    let rows = sqlx::query(
        "SELECT e.id, e.name, e.label FROM _meta_entity e \
         LEFT JOIN _entity_permission p ON p.entity_id = e.id AND p.role = ? \
         WHERE COALESCE(p.can_view, 1) != 0 ORDER BY e.name",
    )
    .bind(role)
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
    // Default permissions: both roles can view and edit.
    for role in ["admin", "user"] {
        sqlx::query(
            "INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
        )
        .bind(id)
        .bind(role)
        .execute(pool)
        .await?;
    }
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
        "SELECT id, name, type, required, is_status, position FROM _meta_field WHERE entity_id = ? ORDER BY position, name",
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
            is_status: row.try_get::<i64, _>("is_status")? != 0,
            position: row.try_get("position")?,
            options,
        });
    }
    Ok(fields)
}

pub async fn get_entity_permissions(
    pool: &SqlitePool,
    entity_id: &str,
) -> Result<Vec<EntityPermission>> {
    require_entity(pool, entity_id).await?;
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT role, can_view, can_edit FROM _entity_permission WHERE entity_id = ? ORDER BY role",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(role, can_view, can_edit)| EntityPermission {
            role,
            can_view: can_view != 0,
            can_edit: can_edit != 0,
        })
        .collect())
}

pub async fn update_entity_permissions(
    pool: &SqlitePool,
    entity_id: &str,
    permissions: &[(String, bool, bool)],
) -> Result<Vec<EntityPermission>> {
    require_entity(pool, entity_id).await?;
    for (role, _, _) in permissions {
        if !matches!(role.as_str(), "admin" | "user") {
            return Err(AppError::BadRequest(format!("invalid role: {role}")).into());
        }
    }
    let mut tx = pool.begin().await?;
    for (role, can_view, can_edit) in permissions {
        sqlx::query(
            "INSERT INTO _entity_permission (entity_id, role, can_view, can_edit) VALUES (?, ?, ?, ?) \
             ON CONFLICT(entity_id, role) DO UPDATE SET can_view = excluded.can_view, can_edit = excluded.can_edit",
        )
        .bind(entity_id)
        .bind(role)
        .bind(*can_view as i64)
        .bind(*can_edit as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    get_entity_permissions(pool, entity_id).await
}

pub async fn check_permission(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
    need_edit: bool,
) -> Result<()> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT can_view, can_edit FROM _entity_permission WHERE entity_id = ? AND role = ?",
    )
    .bind(entity_id)
    .bind(role)
    .fetch_optional(pool)
    .await?;
    // Missing row = default allow (entities created before the migration).
    let (can_view, can_edit) = row.unwrap_or((1, 1));
    if can_view == 0 {
        return Err(AppError::Forbidden(format!("no view access to entity: {entity_id}")).into());
    }
    if need_edit && can_edit == 0 {
        return Err(AppError::Forbidden(format!("no edit access to entity: {entity_id}")).into());
    }
    Ok(())
}

pub async fn get_entity_permission_for_role(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<EntityPermission> {
    require_entity(pool, entity_id).await?;
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT can_view, can_edit FROM _entity_permission WHERE entity_id = ? AND role = ?",
    )
    .bind(entity_id)
    .bind(role)
    .fetch_optional(pool)
    .await?;
    // Missing row = default allow (entities created before the migration).
    let (can_view, can_edit) = row.unwrap_or((1, 1));
    Ok(EntityPermission {
        role: role.to_string(),
        can_view: can_view != 0,
        can_edit: can_edit != 0,
    })
}

pub async fn get_entity_with_permission(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<EntityWithPermission> {
    let detail = get_entity_detail(pool, entity_id).await?;
    let permission = get_entity_permission_for_role(pool, entity_id, role).await?;
    if !permission.can_view {
        return Err(AppError::Forbidden(format!("no view access to entity: {entity_id}")).into());
    }
    let field_map = field_permission_map(pool, entity_id, role).await?;
    let fields = detail
        .fields
        .into_iter()
        .map(|f| {
            let (can_view, can_edit) = field_map.get(&f.name).copied().unwrap_or((true, true));
            FieldWithPermission {
                id: f.id,
                name: f.name,
                r#type: f.r#type,
                required: f.required,
                is_status: f.is_status,
                position: f.position,
                options: f.options,
                can_view,
                can_edit,
            }
        })
        .collect();
    Ok(EntityWithPermission {
        id: detail.id,
        name: detail.name,
        label: detail.label,
        fields,
        permission,
    })
}

pub async fn get_field_permissions(
    pool: &SqlitePool,
    entity_id: &str,
) -> Result<Vec<FieldPermission>> {
    require_entity(pool, entity_id).await?;
    let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT p.field_id, p.role, p.can_view, p.can_edit FROM _field_permission p \
         JOIN _meta_field f ON f.id = p.field_id WHERE f.entity_id = ? ORDER BY f.position, f.name, p.role",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(field_id, role, can_view, can_edit)| FieldPermission {
            field_id,
            role,
            can_view: can_view != 0,
            can_edit: can_edit != 0,
        })
        .collect())
}

pub async fn update_field_permissions(
    pool: &SqlitePool,
    entity_id: &str,
    permissions: &[(String, String, bool, bool)],
) -> Result<Vec<FieldPermission>> {
    require_entity(pool, entity_id).await?;
    for (field_id, role, _, _) in permissions {
        if !matches!(role.as_str(), "admin" | "user") {
            return Err(AppError::BadRequest(format!("invalid role: {role}")).into());
        }
        let owner: Option<String> =
            sqlx::query_scalar("SELECT entity_id FROM _meta_field WHERE id = ?")
                .bind(field_id)
                .fetch_optional(pool)
                .await?;
        match owner {
            Some(owner) if owner == entity_id => {}
            _ => {
                return Err(
                    AppError::BadRequest(format!("field not found in entity: {field_id}")).into(),
                );
            }
        }
    }
    let mut tx = pool.begin().await?;
    for (field_id, role, can_view, can_edit) in permissions {
        sqlx::query(
            "INSERT INTO _field_permission (field_id, role, can_view, can_edit) VALUES (?, ?, ?, ?) \
             ON CONFLICT(field_id, role) DO UPDATE SET can_view = excluded.can_view, can_edit = excluded.can_edit",
        )
        .bind(field_id)
        .bind(role)
        .bind(*can_view as i64)
        .bind(*can_edit as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    get_field_permissions(pool, entity_id).await
}

/// Per-field (can_view, can_edit) keyed by field name for a role.
/// Admin bypasses the matrix (full access); missing rows default to allow.
pub async fn field_permission_map(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<std::collections::HashMap<String, (bool, bool)>> {
    let fields = list_fields(pool, entity_id).await?;
    if role == "admin" {
        return Ok(fields.into_iter().map(|f| (f.name, (true, true))).collect());
    }
    let rows: Vec<(String, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT f.name, p.can_view, p.can_edit FROM _meta_field f \
         LEFT JOIN _field_permission p ON p.field_id = f.id AND p.role = ? \
         WHERE f.entity_id = ?",
    )
    .bind(role)
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(name, can_view, can_edit)| {
            // Missing row (NULL) = default allow.
            (
                name,
                (can_view.unwrap_or(1) != 0, can_edit.unwrap_or(1) != 0),
            )
        })
        .collect())
}

pub async fn check_field_permission(
    pool: &SqlitePool,
    entity_id: &str,
    field_name: &str,
    role: &str,
    need_edit: bool,
) -> Result<()> {
    if role == "admin" {
        return Ok(());
    }
    let map = field_permission_map(pool, entity_id, role).await?;
    let (can_view, can_edit) = map.get(field_name).copied().unwrap_or((true, true));
    if !can_view {
        return Err(AppError::Forbidden(format!("no view access to field: {field_name}")).into());
    }
    if need_edit && !can_edit {
        return Err(AppError::Forbidden(format!("no edit access to field: {field_name}")).into());
    }
    Ok(())
}

pub async fn list_entity_views(pool: &SqlitePool, entity_id: &str) -> Result<Vec<EntityView>> {
    require_entity(pool, entity_id).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id, entity_id, name, config, created_at FROM _entity_view WHERE entity_id = ? ORDER BY name",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    let mut views = Vec::new();
    for (id, entity_id, name, config, created_at) in rows {
        views.push(EntityView {
            id,
            entity_id,
            name,
            config: serde_json::from_str(&config)?,
            created_at,
        });
    }
    Ok(views)
}

pub async fn create_entity_view(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    config: &Value,
) -> Result<EntityView> {
    require_entity(pool, entity_id).await?;
    if name.trim().is_empty() {
        anyhow::bail!("view name is required");
    }
    let id = format!(
        "{entity_id}_{}",
        name.trim().to_lowercase().replace(' ', "_")
    );
    sqlx::query("INSERT INTO _entity_view (id, entity_id, name, config) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(entity_id)
        .bind(name.trim())
        .bind(config.to_string())
        .execute(pool)
        .await?;
    get_entity_view(pool, &id).await
}

pub async fn get_entity_view(pool: &SqlitePool, id: &str) -> Result<EntityView> {
    let row = sqlx::query(
        "SELECT id, entity_id, name, config, created_at FROM _entity_view WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("view not found: {id}")))?;
    use sqlx::Row;
    Ok(EntityView {
        id: row.try_get("id")?,
        entity_id: row.try_get("entity_id")?,
        name: row.try_get("name")?,
        config: serde_json::from_str(&row.try_get::<String, _>("config")?)?,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn delete_entity_view(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _entity_view WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("view not found: {id}")).into());
    }
    Ok(())
}

/// Form layout designer (Visual Builder Phase 2). The layout is a singleton
/// config per entity: `{ "sections": [{ "id", "label", "fields": [field_id] }] }`.
/// Missing layout = default flat render. Unknown field ids are rejected on
/// write (400) and ignored at render time (tolerant policy).
pub async fn get_entity_form_layout(pool: &SqlitePool, entity_id: &str) -> Result<FormLayout> {
    require_entity(pool, entity_id).await?;
    let config: Option<String> =
        sqlx::query_scalar("SELECT config FROM _entity_form_layout WHERE entity_id = ?")
            .bind(entity_id)
            .fetch_optional(pool)
            .await?;
    Ok(FormLayout {
        entity_id: entity_id.to_string(),
        config: config
            .map(|raw| serde_json::from_str(&raw))
            .transpose()?
            .unwrap_or_else(|| serde_json::json!({})),
    })
}

fn validate_form_layout_config(config: &Value, field_ids: &[String]) -> Result<()> {
    let object = config.as_object().ok_or_else(|| {
        AppError::BadRequest("form layout config must be a JSON object".to_string())
    })?;
    let sections = object.get("sections").ok_or_else(|| {
        AppError::BadRequest("form layout config requires a sections array".to_string())
    })?;
    let sections = sections
        .as_array()
        .ok_or_else(|| AppError::BadRequest("sections must be an array".to_string()))?;
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for section in sections {
        let section = section
            .as_object()
            .ok_or_else(|| AppError::BadRequest("each section must be an object".to_string()))?;
        let id = section
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::BadRequest("each section requires a non-empty id".to_string())
            })?;
        if !seen.insert(id) {
            return Err(AppError::BadRequest(format!("duplicate section id: {id}")).into());
        }
        if let Some(label) = section.get("label") {
            if !label.is_string() {
                return Err(
                    AppError::BadRequest(format!("section label must be a string: {id}")).into(),
                );
            }
        }
        let fields = section.get("fields").ok_or_else(|| {
            AppError::BadRequest(format!("section requires a fields array: {id}"))
        })?;
        let fields = fields.as_array().ok_or_else(|| {
            AppError::BadRequest(format!("section fields must be an array: {id}"))
        })?;
        for field in fields {
            let field_id = field.as_str().ok_or_else(|| {
                AppError::BadRequest(format!("section field ids must be strings: {id}"))
            })?;
            if !field_ids.iter().any(|known| known == field_id) {
                return Err(AppError::BadRequest(format!("unknown field: {field_id}")).into());
            }
            if !seen.insert(field_id) {
                return Err(AppError::BadRequest(format!("duplicate field: {field_id}")).into());
            }
        }
    }
    Ok(())
}

pub async fn update_entity_form_layout(
    pool: &SqlitePool,
    entity_id: &str,
    config: &Value,
) -> Result<FormLayout> {
    require_entity(pool, entity_id).await?;
    let fields = list_fields(pool, entity_id).await?;
    let field_ids: Vec<String> = fields.into_iter().map(|f| f.id).collect();
    validate_form_layout_config(config, &field_ids)?;
    sqlx::query(
        "INSERT INTO _entity_form_layout (entity_id, config) VALUES (?, ?) \
         ON CONFLICT(entity_id) DO UPDATE SET config = excluded.config",
    )
    .bind(entity_id)
    .bind(config.to_string())
    .execute(pool)
    .await?;
    get_entity_form_layout(pool, entity_id).await
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

pub async fn update_entity(pool: &SqlitePool, id: &str, name: &str, label: &str) -> Result<Entity> {
    if name.trim().is_empty() || label.trim().is_empty() {
        bail!("name and label are required");
    }
    let result = sqlx::query("UPDATE _meta_entity SET name = ?, label = ? WHERE id = ?")
        .bind(name)
        .bind(label)
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("entity not found: {id}")).into());
    }
    Ok(Entity {
        id: id.to_string(),
        name: name.to_string(),
        label: label.to_string(),
    })
}

pub async fn delete_entity(pool: &SqlitePool, id: &str) -> Result<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
        .bind(id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound(format!("entity not found: {id}")).into());
    }
    let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc WHERE entity_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    if doc_count > 0 {
        return Err(AppError::Conflict(format!(
            "entity has {doc_count} records; delete them first"
        ))
        .into());
    }
    sqlx::query("DELETE FROM _meta_entity WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_field(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
) -> Result<Field> {
    validate_field_name(name)?;
    validate_field_type(field_type)?;
    validate_status_field(pool, entity_id, None, field_type, is_status).await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
        .bind(entity_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound(format!("entity not found: {entity_id}")).into());
    }
    let position: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM _meta_field WHERE entity_id = ?",
    )
    .bind(entity_id)
    .fetch_one(pool)
    .await?;
    let field_id = format!("{entity_id}_{name}");
    sqlx::query(
        "INSERT INTO _meta_field (id, entity_id, name, type, required, is_status, position) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&field_id)
    .bind(entity_id)
    .bind(name)
    .bind(field_type)
    .bind(required as i64)
    .bind(is_status as i64)
    .bind(position)
    .execute(pool)
    .await?;
    // Default field permissions: both roles can view and edit.
    for role in ["admin", "user"] {
        sqlx::query(
            "INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
        )
        .bind(&field_id)
        .bind(role)
        .execute(pool)
        .await?;
    }
    get_field(pool, &field_id).await
}

pub async fn get_field(pool: &SqlitePool, field_id: &str) -> Result<Field> {
    let row = sqlx::query(
        "SELECT id, name, type, required, is_status, position FROM _meta_field WHERE id = ?",
    )
    .bind(field_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("field not found: {field_id}")))?;
    use sqlx::Row;
    let options = list_field_options(pool, field_id).await?;
    Ok(Field {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        r#type: row.try_get("type")?,
        required: row.try_get::<i64, _>("required")? != 0,
        is_status: row.try_get::<i64, _>("is_status")? != 0,
        position: row.try_get("position")?,
        options,
    })
}

pub async fn update_field(
    pool: &SqlitePool,
    field_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
) -> Result<Field> {
    validate_field_name(name)?;
    validate_field_type(field_type)?;
    let entity_id: String = sqlx::query_scalar("SELECT entity_id FROM _meta_field WHERE id = ?")
        .bind(field_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("field not found: {field_id}")))?;
    validate_status_field(pool, &entity_id, Some(field_id), field_type, is_status).await?;
    let result = sqlx::query(
        "UPDATE _meta_field SET name = ?, type = ?, required = ?, is_status = ? WHERE id = ?",
    )
    .bind(name)
    .bind(field_type)
    .bind(required as i64)
    .bind(is_status as i64)
    .bind(field_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("field not found: {field_id}")).into());
    }
    get_field(pool, field_id).await
}

pub async fn delete_field(pool: &SqlitePool, field_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _meta_field WHERE id = ?")
        .bind(field_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("field not found: {field_id}")).into());
    }
    Ok(())
}

pub async fn create_field_option(
    pool: &SqlitePool,
    field_id: &str,
    value: &str,
    label: &str,
) -> Result<FieldOption> {
    if value.trim().is_empty() || label.trim().is_empty() {
        bail!("value and label are required");
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_field WHERE id = ?)")
        .bind(field_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound(format!("field not found: {field_id}")).into());
    }
    let option_id = format!("{field_id}_{value}");
    sqlx::query("INSERT INTO _meta_field_option (id, field_id, value, label) VALUES (?, ?, ?, ?)")
        .bind(&option_id)
        .bind(field_id)
        .bind(value)
        .bind(label)
        .execute(pool)
        .await?;
    get_field_option(pool, &option_id).await
}

pub async fn get_field_option(pool: &SqlitePool, option_id: &str) -> Result<FieldOption> {
    let row = sqlx::query("SELECT id, value, label FROM _meta_field_option WHERE id = ?")
        .bind(option_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("option not found: {option_id}")))?;
    use sqlx::Row;
    Ok(FieldOption {
        id: row.try_get("id")?,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
    })
}

pub async fn update_field_option(
    pool: &SqlitePool,
    option_id: &str,
    value: &str,
    label: &str,
) -> Result<FieldOption> {
    if value.trim().is_empty() || label.trim().is_empty() {
        bail!("value and label are required");
    }
    let result = sqlx::query("UPDATE _meta_field_option SET value = ?, label = ? WHERE id = ?")
        .bind(value)
        .bind(label)
        .bind(option_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("option not found: {option_id}")).into());
    }
    get_field_option(pool, option_id).await
}

pub async fn delete_field_option(pool: &SqlitePool, option_id: &str) -> Result<()> {
    let field_id: Option<String> =
        sqlx::query_scalar("SELECT field_id FROM _meta_field_option WHERE id = ?")
            .bind(option_id)
            .fetch_optional(pool)
            .await?;
    let Some(field_id) = field_id else {
        return Err(AppError::NotFound(format!("option not found: {option_id}")).into());
    };
    let field_type: String = sqlx::query_scalar("SELECT type FROM _meta_field WHERE id = ?")
        .bind(&field_id)
        .fetch_one(pool)
        .await?;
    if field_type == "select" {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _meta_field_option WHERE field_id = ?")
                .bind(&field_id)
                .fetch_one(pool)
                .await?;
        if count <= 1 {
            return Err(
                AppError::BadRequest("select field must have at least one option".into()).into(),
            );
        }
    }
    sqlx::query("DELETE FROM _meta_field_option WHERE id = ?")
        .bind(option_id)
        .execute(pool)
        .await?;
    Ok(())
}

fn validate_field_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("field name is required");
    }
    let valid = name.chars().enumerate().all(|(index, c)| {
        if index == 0 {
            c.is_ascii_lowercase()
        } else {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
        }
    });
    if !valid {
        return Err(AppError::BadRequest(
            "field name must be lowercase snake_case (e.g. work_order)".into(),
        )
        .into());
    }
    Ok(())
}

fn validate_field_type(field_type: &str) -> Result<()> {
    if !matches!(field_type, "text" | "number" | "date" | "select") {
        return Err(AppError::BadRequest(format!("invalid field type: {field_type}")).into());
    }
    Ok(())
}

async fn validate_status_field(
    pool: &SqlitePool,
    entity_id: &str,
    exclude_field_id: Option<&str>,
    field_type: &str,
    is_status: bool,
) -> Result<()> {
    if !is_status {
        return Ok(());
    }
    if field_type != "select" {
        return Err(AppError::BadRequest("status field must be of type select".into()).into());
    }
    let existing: bool = match exclude_field_id {
        Some(field_id) => {
            sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM _meta_field WHERE entity_id = ? AND is_status = 1 AND id != ?)",
            )
            .bind(entity_id)
            .bind(field_id)
            .fetch_one(pool)
            .await?
        }
        None => {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_field WHERE entity_id = ? AND is_status = 1)")
                .bind(entity_id)
                .fetch_one(pool)
                .await?
        }
    };
    if existing {
        return Err(
            AppError::Conflict(format!("entity already has a status field: {entity_id}")).into(),
        );
    }
    Ok(())
}

pub async fn create_document(
    pool: &SqlitePool,
    id: &str,
    entity_id: &str,
    payload: &Value,
    actor: Option<&str>,
) -> Result<Document> {
    create_document_as_role(pool, id, entity_id, payload, actor, "admin").await
}

pub async fn create_document_as_role(
    pool: &SqlitePool,
    id: &str,
    entity_id: &str,
    payload: &Value,
    actor: Option<&str>,
    role: &str,
) -> Result<Document> {
    if id.trim().is_empty() {
        bail!("id is required");
    }
    // Create: drop hidden fields, but keep view-only (non-editable) fields
    // so the initial values are stored.
    let payload = filter_hidden_payload(pool, entity_id, payload, role).await?;
    validate_payload_for_role(pool, entity_id, &payload, role).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO _doc (id, entity_id, payload) VALUES (?, ?, ?)")
        .bind(id)
        .bind(entity_id)
        .bind(payload.to_string())
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, 'create', ?, ?)",
    )
    .bind(audit_id(id, "create"))
    .bind(entity_id)
    .bind(id)
    .bind(payload.to_string())
    .bind(actor)
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

pub async fn export_documents_csv(pool: &SqlitePool, entity_id: &str) -> Result<String> {
    export_documents_csv_as_role(pool, entity_id, "admin").await
}

pub async fn export_documents_csv_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<String> {
    let fields = list_fields(pool, entity_id).await?;
    let viewable = viewable_field_names(pool, entity_id, role).await?;
    let fields: Vec<Field> = fields
        .into_iter()
        .filter(|f| viewable.contains(&f.name))
        .collect();
    if fields.is_empty() {
        return Err(AppError::BadRequest("entity has no fields".into()).into());
    }
    let rows = sqlx::query("SELECT id, payload FROM _doc WHERE entity_id = ? ORDER BY created_at DESC, id DESC LIMIT 1001")
        .bind(entity_id)
        .fetch_all(pool)
        .await?;
    if rows.len() > 1000 {
        return Err(AppError::BadRequest("export exceeds 1000 rows".into()).into());
    }
    let mut csv = String::from("id");
    for field in &fields {
        csv.push(',');
        csv.push_str(&csv_cell(&field.name));
    }
    csv.push('\n');
    for row in rows {
        use sqlx::Row;
        csv.push_str(&csv_cell(&row.try_get::<String, _>("id")?));
        let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        for field in &fields {
            csv.push(',');
            csv.push_str(&csv_cell(&csv_value(payload.get(&field.name))));
        }
        csv.push('\n');
    }
    Ok(csv)
}

fn csv_value(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(other) => other.to_string(),
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// One sheet of a multi-sheet workbook import preview.
#[derive(Debug, Serialize)]
pub struct MultiImportSheet {
    pub entity_id: String,
    pub rows: Vec<Value>,
    pub errors: Vec<String>,
}

/// Preview of a whole-workbook import: one entry per sheet.
#[derive(Debug, Serialize)]
pub struct MultiImportPreview {
    pub sheets: Vec<MultiImportSheet>,
}

/// Per-entity result of a whole-workbook import confirm.
#[derive(Debug, Serialize)]
pub struct MultiImportSheetResult {
    pub entity_id: String,
    pub created: usize,
    pub updated: usize,
}

/// Result of a whole-workbook import confirm.
#[derive(Debug, Serialize)]
pub struct MultiImportResult {
    pub sheets: Vec<MultiImportSheetResult>,
}

/// Export every entity visible to `role` as one `.xlsx` workbook,
/// one sheet per entity. Typed cells (number/bool/string) are written
/// so spreadsheet apps treat values natively; there is no formula
/// injection risk because no cell is written as a formula.
pub async fn export_workbook_xlsx(pool: &SqlitePool, role: &str) -> Result<Vec<u8>> {
    let entities = list_entities_for_role(pool, role).await?;
    if entities.is_empty() {
        return Err(AppError::BadRequest("no visible entities to export".into()).into());
    }
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let mut used_names = std::collections::HashSet::new();
    for entity in &entities {
        let fields = list_fields(pool, &entity.id).await?;
        let viewable = viewable_field_names(pool, &entity.id, role).await?;
        let fields: Vec<Field> = fields
            .into_iter()
            .filter(|f| viewable.contains(&f.name))
            .collect();
        if fields.is_empty() {
            continue;
        }
        let rows = sqlx::query("SELECT id, payload FROM _doc WHERE entity_id = ? ORDER BY created_at DESC, id DESC LIMIT 1001")
            .bind(&entity.id)
            .fetch_all(pool)
            .await?;
        if rows.len() > 1000 {
            return Err(
                AppError::BadRequest(format!("export exceeds 1000 rows: {}", entity.id)).into(),
            );
        }
        let sheet = workbook.add_worksheet();
        sheet
            .set_name(xlsx_sheet_name(&entity.id, &mut used_names))
            .map_err(|e| AppError::BadRequest(format!("invalid sheet name: {e}")))?;
        sheet
            .write_string(0, 0, "id")
            .map_err(|e| AppError::Internal(e.into()))?;
        for (col, field) in fields.iter().enumerate() {
            sheet
                .write_string(0, (col + 1) as u16, &field.name)
                .map_err(|e| AppError::Internal(e.into()))?;
        }
        for (row_index, row) in rows.iter().enumerate() {
            use sqlx::Row;
            let excel_row = (row_index + 1) as u32;
            sheet
                .write_string(excel_row, 0, row.try_get::<String, _>("id")?)
                .map_err(|e| AppError::Internal(e.into()))?;
            let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
            for (col, field) in fields.iter().enumerate() {
                let excel_col = (col + 1) as u16;
                match payload.get(&field.name) {
                    // Skip empty cells: an unwritten cell reads back as Empty.
                    None | Some(Value::Null) => {}
                    Some(Value::Number(number)) => {
                        if let Some(value) = number.as_f64() {
                            sheet
                                .write_number(excel_row, excel_col, value)
                                .map_err(|e| AppError::Internal(e.into()))?;
                        } else {
                            sheet
                                .write_string(excel_row, excel_col, number.to_string())
                                .map_err(|e| AppError::Internal(e.into()))?;
                        }
                    }
                    Some(Value::Bool(flag)) => {
                        sheet
                            .write_boolean(excel_row, excel_col, *flag)
                            .map_err(|e| AppError::Internal(e.into()))?;
                    }
                    Some(Value::String(text)) => {
                        sheet
                            .write_string(excel_row, excel_col, text)
                            .map_err(|e| AppError::Internal(e.into()))?;
                    }
                    Some(other) => {
                        sheet
                            .write_string(excel_row, excel_col, other.to_string())
                            .map_err(|e| AppError::Internal(e.into()))?;
                    }
                }
            }
        }
    }
    workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(e.into()).into())
}

/// Excel sheet names are limited to 31 chars and must be unique.
/// Entity ids are snake_case so they are safe; truncate + dedupe.
fn xlsx_sheet_name(entity_id: &str, used: &mut std::collections::HashSet<String>) -> String {
    let base: String = entity_id.chars().take(31).collect();
    if !used.contains(&base) {
        used.insert(base.clone());
        return base;
    }
    let mut suffix = 2;
    loop {
        let tail = format!("_{suffix}");
        let head: String = base.chars().take(31 - tail.len()).collect();
        let candidate = format!("{head}{tail}");
        if !used.contains(&candidate) {
            used.insert(candidate.clone());
            return candidate;
        }
        suffix += 1;
    }
}

/// Preview a whole-workbook `.xlsx` import: one entry per sheet.
/// Each sheet name maps to an entity id; the header must match that
/// entity's editable fields. Row validation mirrors the CSV preview.
pub async fn preview_workbook_xlsx(
    pool: &SqlitePool,
    input: &[u8],
    role: &str,
) -> Result<MultiImportPreview> {
    use calamine::{Reader, Xlsx};
    let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(std::io::Cursor::new(input))
        .map_err(|e| AppError::BadRequest(format!("invalid xlsx workbook: {e}")))?;
    let names = workbook.sheet_names();
    if names.is_empty() {
        return Err(AppError::BadRequest("workbook has no sheets".into()).into());
    }
    let mut sheets = Vec::new();
    for name in names {
        let range = workbook
            .worksheet_range(&name)
            .map_err(|e| AppError::BadRequest(format!("cannot read sheet {name}: {e}")))?;
        sheets.push(preview_workbook_sheet(pool, &name, range.rows(), role).await?);
    }
    Ok(MultiImportPreview { sheets })
}

async fn preview_workbook_sheet(
    pool: &SqlitePool,
    sheet_name: &str,
    rows: calamine::Rows<'_, calamine::Data>,
    role: &str,
) -> Result<MultiImportSheet> {
    let entity_id = sheet_name.to_string();
    require_entity(pool, &entity_id)
        .await
        .map_err(|_| AppError::BadRequest(format!("unknown entity for sheet: {sheet_name}")))?;
    check_permission(pool, &entity_id, role, true)
        .await
        .map_err(|_| AppError::Forbidden(format!("no edit access to entity: {entity_id}")))?;
    let all_fields = list_fields(pool, &entity_id).await?;
    let field_map = field_permission_map(pool, &entity_id, role).await?;
    let fields: Vec<Field> = all_fields
        .into_iter()
        .filter(|f| field_map.get(&f.name).copied().unwrap_or((true, true)).1)
        .collect();
    let expected: Vec<String> = std::iter::once("id".to_string())
        .chain(fields.iter().map(|f| f.name.clone()))
        .collect();
    let records: Vec<Vec<String>> = rows
        .map(|row| row.iter().map(xlsx_cell_text).collect())
        .collect();
    if records.is_empty() {
        return Err(AppError::BadRequest(format!("sheet {sheet_name} is empty")).into());
    }
    if records[0] != expected {
        return Err(AppError::BadRequest(format!(
            "sheet {sheet_name} header does not match entity fields"
        ))
        .into());
    }
    if records.len() > 1001 {
        return Err(AppError::BadRequest(format!("sheet {sheet_name} exceeds 1000 rows")).into());
    }
    let mut sheet_rows = Vec::new();
    let mut errors = Vec::new();
    let mut ids = std::collections::HashSet::new();
    for (index, record) in records.iter().skip(1).enumerate() {
        if record.len() != expected.len() {
            errors.push(format!(
                "{sheet_name} row {}: wrong column count",
                index + 2
            ));
            continue;
        }
        let id = &record[0];
        if id.trim().is_empty() {
            errors.push(format!("{sheet_name} row {}: id is required", index + 2));
            continue;
        }
        if !ids.insert(id.clone()) {
            errors.push(format!("{sheet_name} row {}: duplicate id", index + 2));
        }
        let mut payload = serde_json::Map::new();
        for (field, value) in fields.iter().zip(record.iter().skip(1)) {
            let parsed = match field.r#type.as_str() {
                "number" => value
                    .parse::<f64>()
                    .ok()
                    .and_then(|n| serde_json::Number::from_f64(n).map(Value::Number)),
                "boolean" => match value.to_ascii_lowercase().as_str() {
                    "true" => Some(Value::Bool(true)),
                    "false" => Some(Value::Bool(false)),
                    _ => None,
                },
                _ => Some(Value::String(value.clone())),
            };
            if let Some(value) = parsed {
                payload.insert(field.name.clone(), value);
            }
        }
        let value = Value::Object(payload);
        if let Err(error) = validate_payload_for_role(pool, &entity_id, &value, role).await {
            errors.push(format!("{sheet_name} row {}: {error}", index + 2));
        }
        sheet_rows.push(serde_json::json!({ "id": id, "payload": value }));
    }
    Ok(MultiImportSheet {
        entity_id,
        rows: sheet_rows,
        errors,
    })
}

/// Confirm a whole-workbook `.xlsx` import atomically: every sheet is
/// previewed first, and all sheets are applied in a single transaction.
pub async fn confirm_workbook_xlsx(
    pool: &SqlitePool,
    input: &[u8],
    actor: Option<&str>,
    role: &str,
) -> Result<MultiImportResult> {
    let preview = preview_workbook_xlsx(pool, input, role).await?;
    let blocked: Vec<String> = preview
        .sheets
        .iter()
        .flat_map(|sheet| sheet.errors.iter().cloned())
        .collect();
    if !blocked.is_empty() {
        return Err(AppError::BadRequest(blocked.join("; ")).into());
    }
    let mut transaction = pool.begin().await?;
    let mut sheets = Vec::new();
    for sheet in preview.sheets {
        let mut created = 0;
        let mut updated = 0;
        for row in sheet.rows {
            let id = row["id"].as_str().unwrap().to_string();
            let payload = row["payload"].clone();
            if upsert_document_in_tx(&mut transaction, &sheet.entity_id, &id, &payload, actor)
                .await?
            {
                updated += 1;
            } else {
                created += 1;
            }
        }
        sheets.push(MultiImportSheetResult {
            entity_id: sheet.entity_id,
            created,
            updated,
        });
    }
    transaction.commit().await?;
    Ok(MultiImportResult { sheets })
}

/// Insert or update one document inside an open transaction.
/// Returns `true` when the document already existed (updated).
async fn upsert_document_in_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entity_id: &str,
    id: &str,
    payload: &Value,
    actor: Option<&str>,
) -> Result<bool> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id = ?)")
        .bind(id)
        .fetch_one(&mut **transaction)
        .await?;
    if exists {
        let owner: String = sqlx::query_scalar("SELECT entity_id FROM _doc WHERE id = ?")
            .bind(id)
            .fetch_one(&mut **transaction)
            .await?;
        if owner != entity_id {
            return Err(AppError::Conflict(format!("id belongs to another entity: {id}")).into());
        }
        sqlx::query("UPDATE _doc SET payload = ?, updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = ?")
            .bind(payload.to_string())
            .bind(id)
            .execute(&mut **transaction)
            .await?;
    } else {
        sqlx::query("INSERT INTO _doc (id, entity_id, payload) VALUES (?, ?, ?)")
            .bind(id)
            .bind(entity_id)
            .bind(payload.to_string())
            .execute(&mut **transaction)
            .await?;
    }
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, 'import', ?, ?)",
    )
    .bind(audit_id(id, "import"))
    .bind(entity_id)
    .bind(id)
    .bind(payload.to_string())
    .bind(actor)
    .execute(&mut **transaction)
    .await?;
    Ok(exists)
}

/// Render a calamine cell as plain text for import parsing.
/// DateTime cells become `YYYY-MM-DD`; numbers keep full precision.
fn xlsx_cell_text(cell: &calamine::Data) -> String {
    match cell {
        calamine::Data::Empty => String::new(),
        calamine::Data::String(text) => text.clone(),
        calamine::Data::Float(value) => {
            if value.fract() == 0.0 && value.is_finite() {
                format!("{}", *value as i64)
            } else {
                value.to_string()
            }
        }
        calamine::Data::Int(value) => value.to_string(),
        calamine::Data::Bool(flag) => flag.to_string(),
        calamine::Data::DateTime(value) => excel_serial_to_date(value.as_f64()),
        calamine::Data::DateTimeIso(text) | calamine::Data::DurationIso(text) => {
            text.chars().take(10).collect()
        }
        calamine::Data::Error(_) => String::new(),
    }
}

/// Convert an Excel serial date (days since 1899-12-30) to `YYYY-MM-DD`.
fn excel_serial_to_date(serial: f64) -> String {
    let days = serial.floor() as i64;
    // Days between 1899-12-30 (Excel epoch) and 1970-01-01 (Unix epoch).
    const EXCEL_TO_UNIX_DAYS: i64 = 25569;
    let unix_days = days - EXCEL_TO_UNIX_DAYS;
    let seconds = unix_days.saturating_mul(86_400);
    chrono_date_from_unix_days(seconds)
}

fn chrono_date_from_unix_days(seconds: i64) -> String {
    // Civil-from-days (Howard Hinnant's algorithm), no extra dependency.
    let days = seconds.div_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    year += i64::from(month <= 2);
    format!("{year:04}-{:02}-{:02}", month, day)
}

#[derive(Debug, Serialize)]
pub struct ImportPreview {
    pub rows: Vec<Value>,
    pub errors: Vec<String>,
}

pub async fn preview_documents_csv(
    pool: &SqlitePool,
    entity_id: &str,
    input: &str,
) -> Result<ImportPreview> {
    preview_documents_csv_as_role(pool, entity_id, input, "admin").await
}

pub async fn preview_documents_csv_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    input: &str,
    role: &str,
) -> Result<ImportPreview> {
    let all_fields = list_fields(pool, entity_id).await?;
    // Import is a write: the header must match the editable fields.
    let field_map = field_permission_map(pool, entity_id, role).await?;
    let fields: Vec<Field> = all_fields
        .into_iter()
        .filter(|f| field_map.get(&f.name).copied().unwrap_or((true, true)).1)
        .collect();
    let records = parse_csv(input)?;
    if records.is_empty() {
        return Err(AppError::BadRequest("CSV is empty".into()).into());
    }
    let expected: Vec<String> = std::iter::once("id".to_string())
        .chain(fields.iter().map(|f| f.name.clone()))
        .collect();
    if records[0] != expected {
        return Err(AppError::BadRequest("CSV header does not match entity fields".into()).into());
    }
    if records.len() > 1001 {
        return Err(AppError::BadRequest("import exceeds 1000 rows".into()).into());
    }
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    let mut ids = std::collections::HashSet::new();
    for (index, record) in records.iter().skip(1).enumerate() {
        if record.len() != expected.len() {
            errors.push(format!("row {}: wrong column count", index + 2));
            continue;
        }
        let id = &record[0];
        if id.trim().is_empty() {
            errors.push(format!("row {}: id is required", index + 2));
            continue;
        }
        if !ids.insert(id.clone()) {
            errors.push(format!("row {}: duplicate id", index + 2));
        }
        if id.starts_with(['=', '+', '-', '@']) {
            errors.push(format!("row {}: formula values are not allowed", index + 2));
            continue;
        }
        let mut payload = serde_json::Map::new();
        for (field, value) in fields.iter().zip(record.iter().skip(1)) {
            let is_formula = match field.r#type.as_str() {
                // Valid numbers (including negative/positive like -5 or +5) are not formulas.
                "number" => {
                    value.parse::<f64>().is_err() && value.starts_with(['=', '+', '-', '@'])
                }
                _ => value.starts_with(['=', '+', '-', '@']),
            };
            if is_formula {
                errors.push(format!("row {}: formula values are not allowed", index + 2));
            }
            let parsed = match field.r#type.as_str() {
                "number" => value
                    .parse::<f64>()
                    .ok()
                    .and_then(|n| serde_json::Number::from_f64(n).map(Value::Number)),
                "boolean" => match value.to_ascii_lowercase().as_str() {
                    "true" => Some(Value::Bool(true)),
                    "false" => Some(Value::Bool(false)),
                    _ => None,
                },
                _ => Some(Value::String(value.clone())),
            };
            if let Some(value) = parsed {
                payload.insert(field.name.clone(), value);
            }
        }
        let value = Value::Object(payload);
        if let Err(error) = validate_payload_for_role(pool, entity_id, &value, role).await {
            errors.push(format!("row {}: {error}", index + 2));
        }
        rows.push(serde_json::json!({ "id": id, "payload": value }));
    }
    Ok(ImportPreview { rows, errors })
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub created: usize,
    pub updated: usize,
}

pub async fn confirm_documents_csv(
    pool: &SqlitePool,
    entity_id: &str,
    input: &str,
    actor: Option<&str>,
) -> Result<ImportResult> {
    confirm_documents_csv_as_role(pool, entity_id, input, actor, "admin").await
}

pub async fn confirm_documents_csv_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    input: &str,
    actor: Option<&str>,
    role: &str,
) -> Result<ImportResult> {
    let preview = preview_documents_csv_as_role(pool, entity_id, input, role).await?;
    if !preview.errors.is_empty() {
        return Err(AppError::BadRequest(preview.errors.join("; ")).into());
    }
    let mut transaction = pool.begin().await?;
    let mut created = 0;
    let mut updated = 0;
    for row in preview.rows {
        let id = row["id"].as_str().unwrap().to_string();
        let payload = row["payload"].clone();
        if upsert_document_in_tx(&mut transaction, entity_id, &id, &payload, actor).await? {
            updated += 1;
        } else {
            created += 1;
        }
    }
    transaction.commit().await?;
    Ok(ImportResult { created, updated })
}

fn parse_csv(input: &str) -> Result<Vec<Vec<String>>> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => {
                record.push(std::mem::take(&mut cell));
            }
            ('\n', false) => {
                record.push(std::mem::take(&mut cell));
                records.push(std::mem::take(&mut record));
            }
            ('\r', false) => {
                if chars.peek() != Some(&'\n') {
                    record.push(std::mem::take(&mut cell));
                    records.push(std::mem::take(&mut record));
                }
            }
            _ => cell.push(ch),
        }
    }
    if quoted {
        return Err(AppError::BadRequest("malformed CSV quote".into()).into());
    }
    if !cell.is_empty() || !record.is_empty() {
        record.push(cell);
        records.push(record);
    }
    Ok(records)
}

#[derive(Debug, Default, Clone)]
pub struct ListDocumentsFilter {
    pub search: Option<String>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub view_id: Option<String>,
}

/// View config keys understood by `list_documents`.
/// `{ "status": "open", "search": "pump", "sort_by": "title", "sort_dir": "asc" }`
fn apply_view_config(filter: &mut ListDocumentsFilter, config: &Value) {
    let Some(object) = config.as_object() else {
        return;
    };
    let get_str = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    if filter.status.is_none() {
        filter.status = get_str("status");
    }
    if filter.search.is_none() {
        filter.search = get_str("search");
    }
    if filter.sort_by.is_none() {
        filter.sort_by = get_str("sort_by");
    }
    if filter.sort_dir.is_none() {
        filter.sort_dir = get_str("sort_dir");
    }
}

pub async fn list_documents(
    pool: &SqlitePool,
    entity_id: &str,
    limit: i64,
    offset: i64,
    filter: &ListDocumentsFilter,
) -> Result<DocumentList> {
    list_documents_as_role(pool, entity_id, limit, offset, filter, "admin").await
}

pub async fn list_documents_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    limit: i64,
    offset: i64,
    filter: &ListDocumentsFilter,
    role: &str,
) -> Result<DocumentList> {
    use sqlx::Row;
    let mut filter = filter.clone();
    if let Some(view_id) = filter.view_id.clone().filter(|s| !s.trim().is_empty()) {
        let view = get_entity_view(pool, &view_id).await?;
        if view.entity_id != entity_id {
            return Err(AppError::BadRequest("view belongs to another entity".into()).into());
        }
        apply_view_config(&mut filter, &view.config);
    }
    let fields = list_fields(pool, entity_id).await?;
    let viewable = viewable_field_names(pool, entity_id, role).await?;
    let mut where_sql = String::from("entity_id = ?");
    let mut params: Vec<String> = vec![entity_id.to_string()];

    if let Some(status) = filter.status.as_deref().filter(|s| !s.trim().is_empty()) {
        if let Some(status_field) = fields.iter().find(|f| f.is_status) {
            // Hidden status field: ignore the filter rather than leak existence.
            if viewable.contains(&status_field.name) {
                where_sql.push_str(&format!(
                    " AND json_extract(payload, '$.{}') = ?",
                    status_field.name
                ));
                params.push(status.to_string());
            }
        }
    }
    if let Some(search) = filter.search.as_deref().filter(|s| !s.trim().is_empty()) {
        let like = format!("%{}%", search.trim());
        let mut clauses = vec!["id LIKE ?".to_string()];
        params.push(like.clone());
        for field in fields
            .iter()
            .filter(|f| f.r#type == "text" && viewable.contains(&f.name))
        {
            clauses.push(format!("json_extract(payload, '$.{}') LIKE ?", field.name));
            params.push(like.clone());
        }
        where_sql.push_str(&format!(" AND ({})", clauses.join(" OR ")));
    }

    let total: i64 = {
        let query = format!("SELECT COUNT(*) FROM _doc WHERE {where_sql}");
        let mut q = sqlx::query(&query);
        for p in &params {
            q = q.bind(p);
        }
        q.fetch_one(pool).await?.try_get(0)?
    };

    let sort_col = match filter.sort_by.as_deref() {
        Some(name) if fields.iter().any(|f| f.name == name) && viewable.contains(name) => {
            format!("json_extract(payload, '$.{name}')")
        }
        _ => "created_at".to_string(),
    };
    let dir = match filter.sort_dir.as_deref() {
        Some(d) if d.eq_ignore_ascii_case("asc") => "ASC",
        _ => "DESC",
    };

    let query = format!(
        "SELECT id, entity_id, payload, created_at, updated_at FROM _doc WHERE {where_sql} ORDER BY {sort_col} {dir}, id DESC LIMIT ? OFFSET ?"
    );
    let mut q = sqlx::query(&query);
    for p in &params {
        q = q.bind(p);
    }
    let rows = q.bind(limit).bind(offset).fetch_all(pool).await?;
    let mut items = Vec::new();
    for row in rows {
        let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        items.push(Document {
            id: row.try_get("id")?,
            entity_id: row.try_get("entity_id")?,
            payload: redact_payload(&payload, &viewable),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }
    Ok(DocumentList { items, total })
}

pub async fn update_document(
    pool: &SqlitePool,
    id: &str,
    payload: &Value,
    actor: Option<&str>,
    expected_updated_at: Option<&str>,
) -> Result<Document> {
    update_document_as_role(pool, id, payload, actor, expected_updated_at, "admin").await
}

pub async fn update_document_as_role(
    pool: &SqlitePool,
    id: &str,
    payload: &Value,
    actor: Option<&str>,
    expected_updated_at: Option<&str>,
    role: &str,
) -> Result<Document> {
    let existing = get_document(pool, id).await?;
    // Drop hidden keys from the incoming payload first (unknown-field
    // tolerance), then merge over stored values and restore stored values
    // for non-editable fields so a forced write cannot change them.
    let incoming = filter_hidden_payload(pool, &existing.entity_id, payload, role).await?;
    let merged = merge_editable_payload(&existing.payload, &incoming);
    let merged =
        restore_readonly_fields(pool, &existing.entity_id, &existing.payload, &merged, role)
            .await?;
    validate_payload_for_role(pool, &existing.entity_id, &merged, role).await?;
    let payload = merged;
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE _doc SET payload = ?, updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = ? AND (? IS NULL OR updated_at = ?)",
    )
    .bind(payload.to_string())
    .bind(id)
    .bind(expected_updated_at)
    .bind(expected_updated_at)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(
            AppError::Conflict(format!("stale record: {id} was modified by another user")).into(),
        );
    }
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, 'update', ?, ?)",
    )
    .bind(audit_id(id, "update"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(payload.to_string())
    .bind(actor)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

pub async fn transition_document(
    pool: &SqlitePool,
    id: &str,
    action: &str,
    actor: Option<&str>,
    expected_updated_at: Option<&str>,
) -> Result<Document> {
    transition_document_as_role(pool, id, action, actor, expected_updated_at, "admin").await
}

pub async fn transition_document_as_role(
    pool: &SqlitePool,
    id: &str,
    action: &str,
    actor: Option<&str>,
    expected_updated_at: Option<&str>,
    role: &str,
) -> Result<Document> {
    let existing = get_document(pool, id).await?;
    let fields = list_fields(pool, &existing.entity_id).await?;
    let status_field = fields
        .iter()
        .find(|f| f.is_status)
        .ok_or_else(|| AppError::BadRequest("entity has no status field".into()))?;
    let current = existing
        .payload
        .get(&status_field.name)
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
    check_field_permission(pool, &existing.entity_id, &status_field.name, role, true).await?;
    let mut next = existing.payload;
    next[status_field.name.as_str()] = Value::String(target);
    validate_payload_for_role(pool, &existing.entity_id, &next, role).await?;
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE _doc SET payload = ?, updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = ? AND (? IS NULL OR updated_at = ?)",
    )
    .bind(next.to_string())
    .bind(id)
    .bind(expected_updated_at)
    .bind(expected_updated_at)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(
            AppError::Conflict(format!("stale record: {id} was modified by another user")).into(),
        );
    }
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, 'transition', ?, ?)",
    )
    .bind(audit_id(id, "transition"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(next.to_string())
    .bind(actor)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

pub async fn get_workflow(pool: &SqlitePool, entity_id: &str) -> Result<WorkflowDefinition> {
    let states = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT id, name, label, position FROM _workflow_state WHERE entity_id = ? ORDER BY position",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, name, label, position)| WorkflowState {
        id,
        name,
        label,
        position,
    })
    .collect();
    let transitions = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, action, from_state, to_state FROM _workflow_transition WHERE entity_id = ? ORDER BY from_state, action",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, action, from_state, to_state)| WorkflowTransition {
        id,
        action,
        from_state,
        to_state,
    })
    .collect();
    Ok(WorkflowDefinition {
        states,
        transitions,
    })
}

fn validate_state_name(name: &str) -> Result<()> {
    validate_field_name(name).map_err(|_| {
        AppError::BadRequest("state name must be lowercase snake_case (e.g. open)".into()).into()
    })
}

fn validate_action_name(action: &str) -> Result<()> {
    validate_field_name(action).map_err(|_| {
        AppError::BadRequest("action must be lowercase snake_case (e.g. submit)".into()).into()
    })
}

async fn require_entity(pool: &SqlitePool, entity_id: &str) -> Result<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
        .bind(entity_id)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound(format!("entity not found: {entity_id}")).into());
    }
    Ok(())
}

pub async fn create_workflow_state(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    label: &str,
) -> Result<WorkflowState> {
    validate_state_name(name)?;
    if label.trim().is_empty() {
        anyhow::bail!("label is required");
    }
    require_entity(pool, entity_id).await?;
    let position: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM _workflow_state WHERE entity_id = ?",
    )
    .bind(entity_id)
    .fetch_one(pool)
    .await?;
    let id = format!("{entity_id}_{name}");
    sqlx::query(
        "INSERT INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(name)
    .bind(label.trim())
    .bind(position)
    .execute(pool)
    .await?;
    get_workflow_state(pool, &id).await
}

pub async fn get_workflow_state(pool: &SqlitePool, id: &str) -> Result<WorkflowState> {
    let row = sqlx::query("SELECT id, name, label, position FROM _workflow_state WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("workflow state not found: {id}")))?;
    use sqlx::Row;
    Ok(WorkflowState {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        label: row.try_get("label")?,
        position: row.try_get("position")?,
    })
}

pub async fn update_workflow_state(
    pool: &SqlitePool,
    id: &str,
    label: &str,
) -> Result<WorkflowState> {
    if label.trim().is_empty() {
        anyhow::bail!("label is required");
    }
    let result = sqlx::query("UPDATE _workflow_state SET label = ? WHERE id = ?")
        .bind(label.trim())
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("workflow state not found: {id}")).into());
    }
    get_workflow_state(pool, id).await
}

pub async fn delete_workflow_state(pool: &SqlitePool, id: &str) -> Result<()> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT entity_id, name FROM _workflow_state WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    let Some((entity_id, name)) = row else {
        return Err(AppError::NotFound(format!("workflow state not found: {id}")).into());
    };
    let in_use: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM _workflow_transition WHERE entity_id = ? AND (from_state = ? OR to_state = ?))",
    )
    .bind(&entity_id)
    .bind(&name)
    .bind(&name)
    .fetch_one(pool)
    .await?;
    if in_use {
        return Err(AppError::Conflict(format!("state is used by transitions: {name}")).into());
    }
    let result = sqlx::query("DELETE FROM _workflow_state WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("workflow state not found: {id}")).into());
    }
    Ok(())
}

pub async fn create_workflow_transition(
    pool: &SqlitePool,
    entity_id: &str,
    from_state: &str,
    to_state: &str,
    action: &str,
) -> Result<WorkflowTransition> {
    validate_action_name(action)?;
    require_entity(pool, entity_id).await?;
    for state in [from_state, to_state] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _workflow_state WHERE entity_id = ? AND name = ?)",
        )
        .bind(entity_id)
        .bind(state)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(AppError::BadRequest(format!("unknown state: {state}")).into());
        }
    }
    if from_state == to_state {
        return Err(AppError::BadRequest("from_state and to_state must differ".into()).into());
    }
    let id = format!("{entity_id}_{from_state}_{action}");
    sqlx::query(
        "INSERT INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(from_state)
    .bind(to_state)
    .bind(action)
    .execute(pool)
    .await?;
    get_workflow_transition(pool, &id).await
}

pub async fn get_workflow_transition(pool: &SqlitePool, id: &str) -> Result<WorkflowTransition> {
    let row = sqlx::query(
        "SELECT id, action, from_state, to_state FROM _workflow_transition WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("workflow transition not found: {id}")))?;
    use sqlx::Row;
    Ok(WorkflowTransition {
        id: row.try_get("id")?,
        action: row.try_get("action")?,
        from_state: row.try_get("from_state")?,
        to_state: row.try_get("to_state")?,
    })
}

pub async fn delete_workflow_transition(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _workflow_transition WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("workflow transition not found: {id}")).into());
    }
    Ok(())
}

pub async fn count_documents_by_status(
    pool: &SqlitePool,
    entity_id: &str,
) -> Result<Vec<StatusCount>> {
    count_documents_by_status_as_role(pool, entity_id, "admin").await
}

pub async fn count_documents_by_status_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<Vec<StatusCount>> {
    let fields = list_fields(pool, entity_id).await?;
    let Some(status_field) = fields.iter().find(|f| f.is_status) else {
        return Ok(Vec::new());
    };
    check_field_permission(pool, entity_id, &status_field.name, role, false).await?;
    let query = format!(
        "SELECT json_extract(payload, '$.{}'), COUNT(*) FROM _doc WHERE entity_id = ? GROUP BY json_extract(payload, '$.{}') ORDER BY 1",
        status_field.name, status_field.name
    );
    let rows = sqlx::query_as::<_, (String, i64)>(&query)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(status, count)| StatusCount { status, count })
        .collect())
}

/// PM summary: open (not done), overdue (not done and due_date < today),
/// done this week (done and updated_at >= start of current UTC week).
pub async fn pm_summary(pool: &SqlitePool, entity_id: &str) -> Result<PmSummary> {
    pm_summary_as_role(pool, entity_id, "admin").await
}

pub async fn pm_summary_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<PmSummary> {
    let fields = list_fields(pool, entity_id).await?;
    let Some(status_field) = fields.iter().find(|f| f.is_status) else {
        return Ok(PmSummary {
            open: 0,
            overdue: 0,
            done_this_week: 0,
            total: 0,
        });
    };
    check_field_permission(pool, entity_id, &status_field.name, role, false).await?;
    let status_col = format!("json_extract(payload, '$.{}')", status_field.name);
    let due_col = fields
        .iter()
        .find(|f| f.name == "due_date")
        .map(|_| "json_extract(payload, '$.due_date')".to_string());

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc WHERE entity_id = ?")
        .bind(entity_id)
        .fetch_one(pool)
        .await?;

    let open: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM _doc WHERE entity_id = ? AND {status_col} != 'done'"
    ))
    .bind(entity_id)
    .fetch_one(pool)
    .await?;

    let overdue: i64 = match &due_col {
        Some(due) => sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM _doc WHERE entity_id = ? AND {status_col} != 'done' AND {due} IS NOT NULL AND {due} < date('now')"
        ))
        .bind(entity_id)
        .fetch_one(pool)
        .await?,
        None => 0,
    };

    let done_this_week: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM _doc WHERE entity_id = ? AND {status_col} = 'done' AND updated_at >= datetime('now', '-6 days', 'start of day')"
    ))
    .bind(entity_id)
    .fetch_one(pool)
    .await?;

    Ok(PmSummary {
        open,
        overdue,
        done_this_week,
        total,
    })
}

pub async fn delete_document(pool: &SqlitePool, id: &str, actor: Option<&str>) -> Result<()> {
    let existing = get_document(pool, id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM _doc WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, 'delete', ?, ?)",
    )
    .bind(audit_id(id, "delete"))
    .bind(&existing.entity_id)
    .bind(id)
    .bind(existing.payload.to_string())
    .bind(actor)
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
    list_document_audit_as_role(pool, doc_id, limit, offset, "admin").await
}

pub async fn list_document_audit_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    limit: i64,
    offset: i64,
    role: &str,
) -> Result<AuditList> {
    let entity_id = get_document(pool, doc_id).await.map(|d| d.entity_id);
    if entity_id.is_err() {
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
        "SELECT id, entity_id, doc_id, action, payload, created_at, actor FROM _audit_log WHERE doc_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(doc_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let viewable = match &entity_id {
        Ok(entity_id) => viewable_field_names(pool, entity_id, role).await?,
        Err(_) => std::collections::HashSet::new(),
    };
    let mut items = Vec::new();
    for row in rows {
        use sqlx::Row;
        let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        items.push(AuditEntry {
            id: row.try_get("id")?,
            entity_id: row.try_get("entity_id")?,
            doc_id: row.try_get("doc_id")?,
            action: row.try_get("action")?,
            payload: redact_payload(&payload, &viewable),
            created_at: row.try_get("created_at")?,
            actor: row.try_get("actor")?,
        });
    }
    Ok(AuditList { items, total })
}

pub async fn list_global_audit(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
    filter: &GlobalAuditFilter,
) -> Result<GlobalAuditList> {
    list_global_audit_as_role(pool, limit, offset, filter, "admin").await
}

pub async fn list_global_audit_as_role(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
    filter: &GlobalAuditFilter,
    role: &str,
) -> Result<GlobalAuditList> {
    use sqlx::Row;
    let mut where_sql = String::from("1 = 1");
    let mut params: Vec<String> = Vec::new();

    if let Some(entity_id) = filter.entity_id.as_deref().filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(" AND a.entity_id = ?");
        params.push(entity_id.to_string());
    }
    if let Some(action) = filter.action.as_deref().filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(" AND a.action = ?");
        params.push(action.to_string());
    }
    if let Some(search) = filter.search.as_deref().filter(|s| !s.trim().is_empty()) {
        where_sql.push_str(" AND a.doc_id LIKE ?");
        params.push(format!("%{}%", search.trim()));
    }

    let total: i64 = {
        let query = format!("SELECT COUNT(*) FROM _audit_log a WHERE {where_sql}");
        let mut q = sqlx::query(&query);
        for p in &params {
            q = q.bind(p);
        }
        q.fetch_one(pool).await?.try_get(0)?
    };

    let query = format!(
        "SELECT a.id, a.entity_id, e.label, a.doc_id, a.action, a.payload, a.created_at, a.actor \
         FROM _audit_log a JOIN _meta_entity e ON e.id = a.entity_id \
         WHERE {where_sql} ORDER BY a.created_at DESC, a.id DESC LIMIT ? OFFSET ?"
    );
    let mut q = sqlx::query(&query);
    for p in &params {
        q = q.bind(p);
    }
    let rows = q.bind(limit).bind(offset).fetch_all(pool).await?;
    let mut items = Vec::new();
    // Cache per-entity viewable sets (audit spans entities).
    let mut viewable_cache: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for row in rows {
        let entity_id: String = row.try_get("entity_id")?;
        let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
        let viewable = match viewable_cache.get(&entity_id) {
            Some(set) => set.clone(),
            None => {
                let set = viewable_field_names(pool, &entity_id, role).await?;
                viewable_cache.insert(entity_id.clone(), set.clone());
                set
            }
        };
        items.push(GlobalAuditEntry {
            id: row.try_get("id")?,
            entity_id,
            entity_label: row.try_get("label")?,
            doc_id: row.try_get("doc_id")?,
            action: row.try_get("action")?,
            payload: redact_payload(&payload, &viewable),
            created_at: row.try_get("created_at")?,
            actor: row.try_get("actor")?,
        });
    }
    Ok(GlobalAuditList { items, total })
}

fn audit_id(doc_id: &str, action: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // ponytail: use ulid/uuid when ordering/content-addressing needed.
    format!("{doc_id}-{action}-{nanos}")
}

/// Validate a payload for a role. Only viewable fields are validated;
/// hidden required fields are not required for that role.
#[allow(dead_code)]
async fn validate_payload(pool: &SqlitePool, entity_id: &str, payload: &Value) -> Result<()> {
    validate_payload_for_role(pool, entity_id, payload, "admin").await
}

async fn validate_payload_for_role(
    pool: &SqlitePool,
    entity_id: &str,
    payload: &Value,
    role: &str,
) -> Result<()> {
    let object = payload
        .as_object()
        .ok_or_else(|| AppError::BadRequest("payload must be a JSON object".into()))?;
    let fields = list_fields(pool, entity_id).await?;
    let field_map = field_permission_map(pool, entity_id, role).await?;
    let visible: Vec<&Field> = fields
        .iter()
        .filter(|f| field_map.get(&f.name).copied().unwrap_or((true, true)).0)
        .collect();
    for field in &visible {
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
        if !visible.iter().any(|f| f.name == *key) {
            return Err(AppError::BadRequest(format!("unknown field: {key}")).into());
        }
    }
    Ok(())
}

/// Drop hidden (non-viewable) fields from an incoming payload.
/// View-only (non-editable) fields are kept so their values are stored;
/// enforcement of edit rights happens by merging over stored values on
/// update. Admin payloads pass through unchanged.
async fn filter_hidden_payload(
    pool: &SqlitePool,
    entity_id: &str,
    payload: &Value,
    role: &str,
) -> Result<Value> {
    if role == "admin" {
        return Ok(payload.clone());
    }
    let Some(object) = payload.as_object() else {
        return Err(AppError::BadRequest("payload must be a JSON object".into()).into());
    };
    let field_map = field_permission_map(pool, entity_id, role).await?;
    let filtered: serde_json::Map<String, Value> = object
        .iter()
        .filter(|(key, _)| field_map.get(*key).copied().unwrap_or((true, true)).0)
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    Ok(Value::Object(filtered))
}

/// Restore stored values for non-editable (view-only) fields.
async fn restore_readonly_fields(
    pool: &SqlitePool,
    entity_id: &str,
    stored: &Value,
    merged: &Value,
    role: &str,
) -> Result<Value> {
    if role == "admin" {
        return Ok(merged.clone());
    }
    let Some(stored_object) = stored.as_object() else {
        return Ok(merged.clone());
    };
    let Some(merged_object) = merged.as_object() else {
        return Ok(merged.clone());
    };
    let field_map = field_permission_map(pool, entity_id, role).await?;
    let mut restored = merged_object.clone();
    for (key, stored_value) in stored_object {
        let (_, can_edit) = field_map.get(key).copied().unwrap_or((true, true));
        if !can_edit {
            restored.insert(key.clone(), stored_value.clone());
        }
    }
    Ok(Value::Object(restored))
}

/// Merge an incoming partial payload over the stored payload.
fn merge_editable_payload(stored: &Value, incoming: &Value) -> Value {
    let Some(stored_object) = stored.as_object() else {
        return incoming.clone();
    };
    let Some(incoming_object) = incoming.as_object() else {
        return incoming.clone();
    };
    let mut merged = stored_object.clone();
    for (key, value) in incoming_object {
        merged.insert(key.clone(), value.clone());
    }
    Value::Object(merged)
}

/// Strip non-viewable fields from a stored payload for a role.
pub fn redact_payload(payload: &Value, viewable: &std::collections::HashSet<String>) -> Value {
    let Some(object) = payload.as_object() else {
        return payload.clone();
    };
    Value::Object(
        object
            .iter()
            .filter(|(key, _)| viewable.contains(*key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

pub async fn viewable_field_names(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
) -> Result<std::collections::HashSet<String>> {
    if role == "admin" {
        return Ok(list_fields(pool, entity_id)
            .await?
            .into_iter()
            .map(|f| f.name)
            .collect());
    }
    let map = field_permission_map(pool, entity_id, role).await?;
    Ok(map
        .into_iter()
        .filter(|(_, (can_view, _))| *can_view)
        .map(|(name, _)| name)
        .collect())
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
