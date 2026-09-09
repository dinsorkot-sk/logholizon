use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::metadata;

pub use crate::module_lifecycle::{disable_module, enable_module, submit_module_for_review};

#[derive(Debug, Serialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub label: String,
    pub module: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub is_status: bool,
    pub position: i64,
    pub ref_entity: Option<String>,
    pub computed_expr: Option<String>,
    pub is_unique: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub min_length: Option<i64>,
    pub max_length: Option<i64>,
    pub default_value: Option<String>,
    pub auto_number_prefix: Option<String>,
    pub auto_number_width: Option<i64>,
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
    pub module: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize)]
pub struct EntityWithPermission {
    pub id: String,
    pub name: String,
    pub label: String,
    pub module: Option<String>,
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
    pub ref_entity: Option<String>,
    pub computed_expr: Option<String>,
    pub is_unique: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub min_length: Option<i64>,
    pub max_length: Option<i64>,
    pub default_value: Option<String>,
    pub auto_number_prefix: Option<String>,
    pub auto_number_width: Option<i64>,
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
pub struct DocComment {
    pub id: String,
    pub entity_id: String,
    pub doc_id: String,
    pub body: String,
    pub actor: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DocCommentList {
    pub items: Vec<DocComment>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DocFollowerList {
    pub followers: Vec<String>,
    pub total: i64,
    pub is_following: bool,
}

#[derive(Debug, Serialize)]
pub struct DocActivity {
    pub id: String,
    pub entity_id: String,
    pub doc_id: String,
    pub title: String,
    pub due_date: Option<String>,
    pub assignee: Option<String>,
    pub done: bool,
    pub actor: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DocActivityList {
    pub items: Vec<DocActivity>,
    pub total: i64,
    pub open: i64,
}

pub const ATTACHMENT_MAX_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct DocAttachment {
    pub id: String,
    pub entity_id: String,
    pub doc_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub actor: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DocAttachmentList {
    pub items: Vec<DocAttachment>,
    pub total: i64,
}

#[derive(Debug)]
pub struct DocAttachmentData {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
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
pub struct NotificationRule {
    pub id: String,
    pub entity_id: String,
    pub trigger: String,
    pub target_url: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationDelivery {
    pub id: String,
    pub rule_id: String,
    pub document_id: String,
    pub action: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationDeliveryList {
    pub items: Vec<NotificationDelivery>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub config: Value,
    pub created_by: Option<String>,
    pub created_at: String,
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

// --- User-defined modules: registry, versioning, tenant owner ---

#[derive(Debug, Serialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub owner: String,
    pub status: String,
    pub version: i64,
    pub definition: Value,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleVersion {
    pub id: String,
    pub module_id: String,
    pub version: i64,
    pub definition: Value,
    pub actor: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct Automation {
    pub id: String,
    pub entity_id: String,
    pub trigger: String,
    pub action: String,
    pub target_url: String,
    pub active: bool,
    pub created_at: String,
}

fn validate_module_name(name: &str) -> Result<()> {
    validate_field_name(name).map_err(|_| {
        AppError::BadRequest("module name must be lowercase snake_case (e.g. vehicle)".into())
            .into()
    })
}

type ModuleRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    Option<String>,
    String,
    String,
);

#[allow(clippy::too_many_arguments)]
fn module_row(
    id: String,
    name: String,
    label: String,
    description: String,
    icon: String,
    color: String,
    owner: String,
    status: String,
    version: i64,
    definition: String,
    created_by: Option<String>,
    created_at: String,
    updated_at: String,
) -> Result<Module> {
    Ok(Module {
        id,
        name,
        label,
        description,
        icon,
        color,
        owner,
        status,
        version,
        definition: serde_json::from_str(&definition)?,
        created_by,
        created_at,
        updated_at,
    })
}

async fn get_module_row(pool: &SqlitePool, id: &str) -> Result<Module> {
    #[allow(clippy::type_complexity)]
    let row: Option<ModuleRow> = sqlx::query_as(
        "SELECT id, name, label, description, icon, color, owner, status, version, definition, created_by, created_at, updated_at FROM _module WHERE id = ?",
    )
    .bind(id.trim())
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Err(AppError::NotFound(format!("module not found: {id}")).into());
    };
    module_row(
        row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9, row.10, row.11,
        row.12,
    )
}

fn check_module_access(module: &Module, owner: &str, role: &str) -> Result<()> {
    if role == "admin" {
        return Ok(());
    }
    if !module.owner.trim().is_empty() && module.owner != owner.trim() {
        return Err(AppError::Forbidden(format!("no access to module: {}", module.id)).into());
    }
    Ok(())
}

/// Validate a module definition: entities with snake_case names, known field
/// types, reference targets inside the same definition, no self references,
/// at most one status field per entity, and valid workflow states.
pub fn validate_module_definition(definition: &Value) -> Result<()> {
    let entities = definition
        .get("entities")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("definition.entities must be an array".into()))?;
    if entities.is_empty() {
        return Err(
            AppError::BadRequest("definition must declare at least one entity".into()).into(),
        );
    }
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entity in entities {
        let name = entity
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::BadRequest("each entity requires a name".into()))?;
        validate_module_name(name)?;
        if !names.insert(name.to_string()) {
            return Err(AppError::BadRequest(format!("duplicate entity: {name}")).into());
        }
    }
    for entity in entities {
        let name = entity.get("name").and_then(Value::as_str).unwrap_or("");
        let fields = entity
            .get("fields")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AppError::BadRequest(format!("entity {name} requires a fields array"))
            })?;
        let mut status_count = 0;
        for field in fields {
            let field_name = field
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    AppError::BadRequest(format!("entity {name} has a field without a name"))
                })?;
            validate_field_name(field_name)?;
            let field_type = field.get("type").and_then(Value::as_str).unwrap_or("text");
            validate_field_type(field_type)?;
            if field
                .get("is_status")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                if field_type != "select" {
                    return Err(AppError::BadRequest(format!(
                        "status field must be of type select: {name}.{field_name}"
                    ))
                    .into());
                }
                status_count += 1;
            }
            if field_type == "reference" {
                let target = field
                    .get("ref_entity")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(format!(
                            "reference field requires ref_entity: {name}.{field_name}"
                        ))
                    })?;
                if target == name {
                    return Err(AppError::BadRequest(format!(
                        "reference field cannot point at its own entity: {name}.{field_name}"
                    ))
                    .into());
                }
                if !names.contains(target) {
                    return Err(AppError::BadRequest(format!(
                        "unknown ref_entity {target} in {name}.{field_name}"
                    ))
                    .into());
                }
            }
            if field_type == "computed" {
                let expr = field
                    .get("computed_expr")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                validate_computed_field(field_type, Some(expr))?;
            }
            validate_field_rules(field_type, &field_rules_from_definition(field))?;
        }
        if status_count > 1 {
            return Err(AppError::BadRequest(format!(
                "entity {name} must have at most one status field"
            ))
            .into());
        }
        if let Some(workflow) = entity.get("workflow") {
            let states: Vec<String> = workflow
                .get("states")
                .and_then(Value::as_array)
                .map(|states| {
                    states
                        .iter()
                        .filter_map(|s| s.get("name").and_then(Value::as_str).map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            if let Some(transitions) = workflow.get("transitions").and_then(Value::as_array) {
                for transition in transitions {
                    let from = transition
                        .get("from_state")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let to = transition
                        .get("to_state")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if from == to {
                        return Err(AppError::BadRequest(format!(
                            "from_state and to_state must differ in {name}"
                        ))
                        .into());
                    }
                    for state in [from, to] {
                        if !states.iter().any(|s| s == state) {
                            return Err(AppError::BadRequest(format!(
                                "unknown state {state} in {name}"
                            ))
                            .into());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_module(
    pool: &SqlitePool,
    name: &str,
    label: &str,
    description: Option<&str>,
    icon: Option<&str>,
    color: Option<&str>,
    owner: &str,
    definition: &Value,
    actor: Option<&str>,
) -> Result<Module> {
    let name = name.trim();
    let label = label.trim();
    if name.is_empty() || label.is_empty() {
        bail!("name and label are required");
    }
    validate_module_name(name)?;
    validate_module_definition(definition)?;
    let owner = owner.trim();
    if owner.is_empty() {
        bail!("owner is required");
    }
    let id = format!("{owner}_{name}");
    let definition_string = serde_json::to_string(definition)?;
    sqlx::query(
        "INSERT INTO _module (id, name, label, description, icon, color, owner, status, version, definition, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, 'draft', 1, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(label)
    .bind(description.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(""))
    .bind(icon.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(""))
    .bind(color.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(""))
    .bind(owner)
    .bind(&definition_string)
    .bind(actor)
    .execute(pool)
    .await?;
    get_module_row(pool, &id).await
}

pub async fn list_modules(pool: &SqlitePool, owner: &str, role: &str) -> Result<Vec<Module>> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<ModuleRow> = if role == "admin" {
        sqlx::query_as(
            "SELECT id, name, label, description, icon, color, owner, status, version, definition, created_by, created_at, updated_at FROM _module ORDER BY owner, name",
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT id, name, label, description, icon, color, owner, status, version, definition, created_by, created_at, updated_at FROM _module WHERE owner = ? ORDER BY name",
        )
        .bind(owner.trim())
        .fetch_all(pool)
        .await?
    };
    let mut modules = Vec::new();
    for row in rows {
        modules.push(module_row(
            row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7, row.8, row.9, row.10, row.11,
            row.12,
        )?);
    }
    Ok(modules)
}

pub async fn get_module(pool: &SqlitePool, id: &str, owner: &str, role: &str) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    Ok(module)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_module_draft(
    pool: &SqlitePool,
    id: &str,
    label: Option<&str>,
    description: Option<&str>,
    icon: Option<&str>,
    color: Option<&str>,
    definition: Option<&Value>,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    if module.status != "draft" {
        return Err(
            AppError::Conflict(format!("module is {}: {}", module.status, module.id)).into(),
        );
    }
    if let Some(definition) = definition {
        validate_module_definition(definition)?;
    }
    let label = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&module.label);
    let definition_string = match definition {
        Some(definition) => serde_json::to_string(definition)?,
        None => serde_json::to_string(&module.definition)?,
    };
    sqlx::query(
        "UPDATE _module SET label = ?, description = ?, icon = ?, color = ?, definition = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(label)
    .bind(description.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(&module.description))
    .bind(icon.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(&module.icon))
    .bind(color.map(str::trim).filter(|s| !s.is_empty()).unwrap_or(&module.color))
    .bind(&definition_string)
    .bind(module.id.trim())
    .execute(pool)
    .await?;
    get_module_row(pool, &module.id).await
}

pub async fn delete_module(pool: &SqlitePool, id: &str, owner: &str, role: &str) -> Result<()> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    if module.status == "published" {
        return Err(
            AppError::Conflict(format!("archive a published module first: {}", module.id)).into(),
        );
    }
    let result = sqlx::query("DELETE FROM _module WHERE id = ?")
        .bind(module.id.trim())
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("module not found: {id}")).into());
    }
    Ok(())
}

async fn materialize_module_definition(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    module: &Module,
) -> Result<()> {
    let entities = module
        .definition
        .get("entities")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for entity in &entities {
        let name = entity
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if name.is_empty() {
            continue;
        }
        let label = entity
            .get("label")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(name);
        let entity_id = format!("{}_{}", module.id, name);
        sqlx::query(
            "INSERT INTO _meta_entity (id, name, label, module, module_id) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET label = excluded.label, module = excluded.module, module_id = excluded.module_id",
        )
        .bind(&entity_id)
        .bind(name)
        .bind(label)
        .bind(&module.label)
        .bind(&module.id)
        .execute(&mut **tx)
        .await?;
        for role in ["admin", "user"] {
            sqlx::query(
                "INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
            )
            .bind(&entity_id)
            .bind(role)
            .execute(&mut **tx)
            .await?;
        }
        if let Some(fields) = entity.get("fields").and_then(Value::as_array) {
            for (position, field) in fields.iter().enumerate() {
                let field_name = field
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim();
                if field_name.is_empty() {
                    continue;
                }
                let field_type = field.get("type").and_then(Value::as_str).unwrap_or("text");
                let required = field
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let is_status = field
                    .get("is_status")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let ref_entity = field
                    .get("ref_entity")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|target| format!("{}_{}", module.id, target));
                let computed_expr = field
                    .get("computed_expr")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);
                let rules = field_rules_from_definition(field);
                let field_id = format!("{entity_id}_{field_name}");
                sqlx::query(
                    "INSERT INTO _meta_field (id, entity_id, name, type, required, is_status, position, ref_entity, computed_expr, is_unique, min_value, max_value, pattern, min_length, max_length, default_value, auto_number_prefix, auto_number_width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                     ON CONFLICT(id) DO UPDATE SET type = excluded.type, required = excluded.required, is_status = excluded.is_status, position = excluded.position, ref_entity = excluded.ref_entity, computed_expr = excluded.computed_expr, is_unique = excluded.is_unique, min_value = excluded.min_value, max_value = excluded.max_value, pattern = excluded.pattern, min_length = excluded.min_length, max_length = excluded.max_length, default_value = excluded.default_value, auto_number_prefix = excluded.auto_number_prefix, auto_number_width = excluded.auto_number_width",
                )
                .bind(&field_id)
                .bind(&entity_id)
                .bind(field_name)
                .bind(field_type)
                .bind(required as i64)
                .bind(is_status as i64)
                .bind(position as i64)
                .bind(ref_entity.as_deref())
                .bind(computed_expr.as_deref())
                .bind(rules.is_unique as i64)
                .bind(rules.min_value)
                .bind(rules.max_value)
                .bind(rules.pattern.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
                .bind(rules.min_length)
                .bind(rules.max_length)
                .bind(rules.default_value.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
                .bind(rules.auto_number_prefix.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
                .bind(rules.auto_number_width)
                .execute(&mut **tx)
                .await?;
                for role in ["admin", "user"] {
                    sqlx::query(
                        "INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
                    )
                    .bind(&field_id)
                    .bind(role)
                    .execute(&mut **tx)
                    .await?;
                }
                if let Some(options) = field.get("options").and_then(Value::as_array) {
                    for option in options {
                        let value = option
                            .get("value")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .trim();
                        let option_label = option
                            .get("label")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .unwrap_or(value);
                        if value.is_empty() {
                            continue;
                        }
                        let option_id = format!("{field_id}_{value}");
                        sqlx::query(
                            "INSERT OR IGNORE INTO _meta_field_option (id, field_id, value, label) VALUES (?, ?, ?, ?)",
                        )
                        .bind(&option_id)
                        .bind(&field_id)
                        .bind(value)
                        .bind(option_label)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }
        if let Some(workflow) = entity.get("workflow") {
            if let Some(states) = workflow.get("states").and_then(Value::as_array) {
                for (position, state) in states.iter().enumerate() {
                    let state_name = state
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    if state_name.is_empty() {
                        continue;
                    }
                    let state_label = state
                        .get("label")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .unwrap_or(state_name);
                    let state_id = format!("{entity_id}_{state_name}");
                    sqlx::query(
                        "INSERT INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, ?, ?, ?, ?) \
                         ON CONFLICT(id) DO UPDATE SET label = excluded.label, position = excluded.position",
                    )
                    .bind(&state_id)
                    .bind(&entity_id)
                    .bind(state_name)
                    .bind(state_label)
                    .bind(position as i64)
                    .execute(&mut **tx)
                    .await?;
                }
            }
            if let Some(transitions) = workflow.get("transitions").and_then(Value::as_array) {
                for transition in transitions {
                    let from = transition
                        .get("from_state")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    let to = transition
                        .get("to_state")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    let action = transition
                        .get("action")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    if from.is_empty() || to.is_empty() || action.is_empty() {
                        continue;
                    }
                    let transition_id = format!("{entity_id}_{from}_{action}");
                    sqlx::query(
                        "INSERT OR IGNORE INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(&transition_id)
                    .bind(&entity_id)
                    .bind(from)
                    .bind(to)
                    .bind(action)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        if let Some(views) = entity.get("views").and_then(Value::as_array) {
            for view in views {
                let view_name = view
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim();
                if view_name.is_empty() {
                    continue;
                }
                let config = view
                    .get("config")
                    .cloned()
                    .unwrap_or(Value::Object(Default::default()));
                let view_id = format!("{entity_id}_view_{}", slugify_view_name(view_name));
                sqlx::query(
                    "INSERT OR IGNORE INTO _entity_view (id, entity_id, name, config) VALUES (?, ?, ?, ?)",
                )
                .bind(&view_id)
                .bind(&entity_id)
                .bind(view_name)
                .bind(config.to_string())
                .execute(&mut **tx)
                .await?;
            }
        }
        if let Some(layout) = entity.get("form_layout") {
            let config = layout
                .get("config")
                .cloned()
                .unwrap_or_else(|| layout.clone());
            sqlx::query(
                "INSERT INTO _entity_form_layout (entity_id, config) VALUES (?, ?) \
                 ON CONFLICT(entity_id) DO UPDATE SET config = excluded.config",
            )
            .bind(&entity_id)
            .bind(config.to_string())
            .execute(&mut **tx)
            .await?;
        }
        if let Some(reports) = entity.get("reports").and_then(Value::as_array) {
            for report in reports {
                let report_name = report
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim();
                if report_name.is_empty() {
                    continue;
                }
                let config = report
                    .get("config")
                    .cloned()
                    .unwrap_or(Value::Object(Default::default()));
                let report_id = format!("{entity_id}_report_{}", slugify_view_name(report_name));
                sqlx::query(
                    "INSERT OR IGNORE INTO _report (id, entity_id, name, config) VALUES (?, ?, ?, ?)",
                )
                .bind(&report_id)
                .bind(&entity_id)
                .bind(report_name)
                .bind(config.to_string())
                .execute(&mut **tx)
                .await?;
            }
        }
        if let Some(notifications) = entity.get("notifications").and_then(Value::as_array) {
            for rule in notifications {
                let target_url = rule
                    .get("target_url")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim();
                if target_url.is_empty() {
                    continue;
                }
                let trigger = rule
                    .get("trigger")
                    .and_then(Value::as_str)
                    .unwrap_or("transition");
                let rule_id = format!("{entity_id}_rule_{}", chrono_nanos());
                sqlx::query(
                    "INSERT INTO _notification_rule (id, entity_id, trigger, target_url, active) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&rule_id)
                .bind(&entity_id)
                .bind(trigger)
                .bind(target_url)
                .bind(rule.get("active").and_then(Value::as_bool).unwrap_or(true) as i64)
                .execute(&mut **tx)
                .await?;
            }
        }
    }
    Ok(())
}

fn slugify_view_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

async fn check_publish_compatibility(pool: &SqlitePool, module: &Module) -> Result<()> {
    let entities = module
        .definition
        .get("entities")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for entity in &entities {
        let name = entity
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if name.is_empty() {
            continue;
        }
        let entity_id = format!("{}_{}", module.id, name);
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
                .bind(&entity_id)
                .fetch_one(pool)
                .await?;
        if !exists {
            continue;
        }
        let stored_fields = list_fields(pool, &entity_id).await?;
        let declared: std::collections::HashSet<String> = entity
            .get("fields")
            .and_then(Value::as_array)
            .map(|fields| {
                fields
                    .iter()
                    .filter_map(|f| f.get("name").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        for stored in &stored_fields {
            if !declared.contains(&stored.name) {
                let in_use: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE entity_id = ?)")
                        .bind(&entity_id)
                        .fetch_one(pool)
                        .await?;
                if in_use {
                    return Err(AppError::Conflict(format!(
                        "entity {entity_id} has records; removing field {} is blocked",
                        stored.name
                    ))
                    .into());
                }
            }
        }
    }
    Ok(())
}

pub async fn publish_module(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
    actor: Option<&str>,
) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    if module.status != "review" {
        return Err(AppError::Conflict(format!(
            "module must be in review before publishing: {}",
            module.id
        ))
        .into());
    }
    validate_module_definition(&module.definition)?;
    check_publish_compatibility(pool, &module).await?;
    let mut tx = pool.begin().await?;
    materialize_module_definition(&mut tx, &module).await?;
    let next_version = module.version + 1;
    let definition_string = serde_json::to_string(&module.definition)?;
    let version_id = format!("{}_{}", module.id, next_version);
    sqlx::query(
        "INSERT INTO _module_version (id, module_id, version, definition, actor) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(&module.id)
    .bind(next_version)
    .bind(&definition_string)
    .bind(actor)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE _module SET status = 'published', version = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(next_version)
    .bind(&module.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_module_row(pool, &module.id).await
}

pub async fn archive_module(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    if !matches!(module.status.as_str(), "published" | "disabled") {
        return Err(AppError::Conflict(format!(
            "module must be published or disabled before archive: {}",
            module.id
        ))
        .into());
    }
    sqlx::query(
        "UPDATE _module SET status = 'archived', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&module.id)
    .execute(pool)
    .await?;
    get_module_row(pool, &module.id).await
}

pub async fn restore_module(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    if module.status != "archived" {
        return Err(
            AppError::Conflict(format!("only archived modules restore: {}", module.id)).into(),
        );
    }
    sqlx::query("UPDATE _module SET status = 'draft', updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&module.id)
        .execute(pool)
        .await?;
    get_module_row(pool, &module.id).await
}

pub async fn list_module_versions(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Vec<ModuleVersion>> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    let rows: Vec<(String, String, i64, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, module_id, version, definition, actor, created_at FROM _module_version WHERE module_id = ? ORDER BY version DESC",
    )
    .bind(&module.id)
    .fetch_all(pool)
    .await?;
    let mut versions = Vec::new();
    for (id, module_id, version, definition, actor, created_at) in rows {
        versions.push(ModuleVersion {
            id,
            module_id,
            version,
            definition: serde_json::from_str(&definition)?,
            actor,
            created_at,
        });
    }
    Ok(versions)
}

pub async fn rollback_module(
    pool: &SqlitePool,
    id: &str,
    version: i64,
    owner: &str,
    role: &str,
    actor: Option<&str>,
) -> Result<Module> {
    let module = get_module_row(pool, id).await?;
    check_module_access(&module, owner, role)?;
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT definition FROM _module_version WHERE module_id = ? AND version = ?",
    )
    .bind(&module.id)
    .bind(version)
    .fetch_optional(pool)
    .await?;
    let Some((definition,)) = row else {
        return Err(AppError::NotFound(format!("module version not found: {version}")).into());
    };
    let definition: Value = serde_json::from_str(&definition)?;
    validate_module_definition(&definition)?;
    let definition_string = serde_json::to_string(&definition)?;
    sqlx::query(
        "UPDATE _module SET definition = ?, status = 'draft', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(&definition_string)
    .bind(&module.id)
    .execute(pool)
    .await?;
    let rolled_back = get_module_row(pool, &module.id).await?;
    check_publish_compatibility(pool, &rolled_back).await?;
    let mut tx = pool.begin().await?;
    materialize_module_definition(&mut tx, &rolled_back).await?;
    let next_version = rolled_back.version + 1;
    let version_id = format!("{}_{}", rolled_back.id, next_version);
    sqlx::query(
        "INSERT INTO _module_version (id, module_id, version, definition, actor) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&version_id)
    .bind(&rolled_back.id)
    .bind(next_version)
    .bind(&definition_string)
    .bind(actor)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE _module SET status = 'published', version = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(next_version)
    .bind(&rolled_back.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    get_module_row(pool, &rolled_back.id).await
}

pub async fn get_module_manifest(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    get_module(pool, id, owner, role).await
}

// --- Generic automations (create/update/delete/transition triggers) ---

pub async fn list_automations(pool: &SqlitePool, entity_id: &str) -> Result<Vec<Automation>> {
    require_entity(pool, entity_id).await?;
    let rows: Vec<(String, String, String, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, entity_id, trigger, action, target_url, active, created_at FROM _automation WHERE entity_id = ? ORDER BY created_at",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, entity_id, trigger, action, target_url, active, created_at)| Automation {
                id,
                entity_id,
                trigger,
                action,
                target_url,
                active: active != 0,
                created_at,
            },
        )
        .collect())
}

pub async fn create_automation(
    pool: &SqlitePool,
    entity_id: &str,
    trigger: &str,
    action: &str,
    target_url: &str,
    active: bool,
) -> Result<Automation> {
    require_entity(pool, entity_id).await?;
    let trigger = trigger.trim();
    let action = action.trim();
    if !matches!(trigger, "create" | "update" | "delete" | "transition") {
        return Err(AppError::BadRequest(format!("invalid trigger: {trigger}")).into());
    }
    if !matches!(action, "webhook" | "notify") {
        return Err(AppError::BadRequest(format!("invalid action: {action}")).into());
    }
    if target_url.trim().is_empty() {
        bail!("target_url is required");
    }
    let id = format!("{entity_id}_automation_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _automation (id, entity_id, trigger, action, target_url, active) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(trigger)
    .bind(action)
    .bind(target_url.trim())
    .bind(active as i64)
    .execute(pool)
    .await?;
    get_automation(pool, &id).await
}

pub async fn get_automation(pool: &SqlitePool, id: &str) -> Result<Automation> {
    let row: Option<(String, String, String, String, String, i64, String)> = sqlx::query_as(
        "SELECT id, entity_id, trigger, action, target_url, active, created_at FROM _automation WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((id, entity_id, trigger, action, target_url, active, created_at)) = row else {
        return Err(AppError::NotFound(format!("automation not found: {id}")).into());
    };
    Ok(Automation {
        id,
        entity_id,
        trigger,
        action,
        target_url,
        active: active != 0,
        created_at,
    })
}

pub async fn delete_automation(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _automation WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("automation not found: {id}")).into());
    }
    Ok(())
}

// --- Native extension contract (empty registry; no built-in domains) ---

/// Stable contract for optional native Rust domain engines.
/// Modules stay metadata-driven; an engine plugs in only when deterministic
/// invariants cannot be expressed as generic rules.
pub trait DomainExtension: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate(&self, _definition: &Value) -> Result<()> {
        Ok(())
    }
}

pub fn domain_extensions() -> Vec<&'static str> {
    Vec::new()
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
        "SELECT e.id, e.name, e.label, e.module FROM _meta_entity e \
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
            module: row.try_get("module")?,
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
        module: None,
    })
}

pub async fn get_entity_detail(pool: &SqlitePool, entity_id: &str) -> Result<EntityDetail> {
    let entity = sqlx::query("SELECT id, name, label, module FROM _meta_entity WHERE id = ?")
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
        module: entity.try_get("module")?,
        fields,
    })
}

pub async fn list_fields(pool: &SqlitePool, entity_id: &str) -> Result<Vec<Field>> {
    let rows = sqlx::query(
        "SELECT id, name, type, required, is_status, position, ref_entity, computed_expr, is_unique, min_value, max_value, pattern, min_length, max_length, default_value, auto_number_prefix, auto_number_width FROM _meta_field WHERE entity_id = ? ORDER BY position, name",
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
            ref_entity: row.try_get("ref_entity")?,
            computed_expr: row.try_get("computed_expr")?,
            is_unique: row.try_get::<i64, _>("is_unique")? != 0,
            min_value: row.try_get("min_value")?,
            max_value: row.try_get("max_value")?,
            pattern: row.try_get("pattern")?,
            min_length: row.try_get("min_length")?,
            max_length: row.try_get("max_length")?,
            default_value: row.try_get("default_value")?,
            auto_number_prefix: row.try_get("auto_number_prefix")?,
            auto_number_width: row.try_get("auto_number_width")?,
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
                ref_entity: f.ref_entity,
                computed_expr: f.computed_expr,
                is_unique: f.is_unique,
                min_value: f.min_value,
                max_value: f.max_value,
                pattern: f.pattern,
                min_length: f.min_length,
                max_length: f.max_length,
                default_value: f.default_value,
                auto_number_prefix: f.auto_number_prefix,
                auto_number_width: f.auto_number_width,
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
        module: detail.module,
        fields,
        permission,
    })
}

#[derive(Debug, Serialize)]
pub struct EntityOption {
    pub id: String,
    pub label: String,
}

/// Dropdown options for reference fields: id + display label, where the
/// label is the first text field value (falling back to the document id).
/// Respects the caller's view permission on the target entity.
/// `search` filters by id or label substring (case-insensitive);
/// `limit` caps rows at 1..=100 (default 50).
pub async fn list_entity_options(
    pool: &SqlitePool,
    entity_id: &str,
    role: &str,
    search: Option<&str>,
    limit: i64,
) -> Result<Vec<EntityOption>> {
    check_permission(pool, entity_id, role, false).await?;
    let limit = limit.clamp(1, 100);
    let fields = list_fields(pool, entity_id).await?;
    let label_field = fields
        .iter()
        .find(|f| f.r#type == "text")
        .map(|f| f.name.clone());
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, payload FROM _doc WHERE entity_id = ? ORDER BY created_at DESC LIMIT 500",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    let mut options = Vec::new();
    for (id, payload) in rows {
        let label = label_field
            .as_deref()
            .and_then(|name| {
                serde_json::from_str::<Value>(&payload)
                    .ok()?
                    .get(name)?
                    .as_str()
                    .map(str::to_string)
            })
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| id.clone());
        if let Some(query) = search.map(str::trim).filter(|s| !s.is_empty()) {
            let query = query.to_lowercase();
            if !id.to_lowercase().contains(&query) && !label.to_lowercase().contains(&query) {
                continue;
            }
        }
        options.push(EntityOption { id, label });
        if options.len() as i64 >= limit {
            break;
        }
    }
    Ok(options)
}

/// Apply computed fields to a stored payload (on read). Each computed
/// field's `{placeholder}` template is interpolated from the payload.
pub fn apply_computed_fields(fields: &[Field], payload: &Value) -> Value {
    let computed: Vec<(&str, &str)> = fields
        .iter()
        .filter_map(|f| {
            if f.r#type == "computed" {
                f.computed_expr
                    .as_deref()
                    .map(|expr| (f.name.as_str(), expr))
            } else {
                None
            }
        })
        .collect();
    if computed.is_empty() {
        return payload.clone();
    }
    let mut object = payload.as_object().cloned().unwrap_or_default();
    for (name, expr) in computed {
        object.insert(
            name.to_string(),
            Value::String(compute_field_value(expr, payload)),
        );
    }
    Value::Object(object)
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

/// Webhook notifications (Phase 3, transition-only). Rules are admin-managed
/// per entity; deliveries are enqueued atomically with the transition audit
/// row and sent by a background worker with retry.
fn validate_notification_rule(trigger: &str, target_url: &str) -> Result<()> {
    if trigger != "transition" {
        return Err(AppError::BadRequest(format!("unknown trigger: {trigger}")).into());
    }
    let url = target_url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(
            AppError::BadRequest("target_url must start with http:// or https://".into()).into(),
        );
    }
    Ok(())
}

fn notification_rule_row(
    id: String,
    entity_id: String,
    trigger: String,
    target_url: String,
    active: i64,
    created_at: String,
) -> NotificationRule {
    NotificationRule {
        id,
        entity_id,
        trigger,
        target_url,
        active: active != 0,
        created_at,
    }
}

pub async fn list_notification_rules(
    pool: &SqlitePool,
    entity_id: &str,
) -> Result<Vec<NotificationRule>> {
    require_entity(pool, entity_id).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        "SELECT id, entity_id, trigger, target_url, active, created_at FROM _notification_rule WHERE entity_id = ? ORDER BY created_at",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, entity_id, trigger, target_url, active, created_at)| {
            notification_rule_row(id, entity_id, trigger, target_url, active, created_at)
        })
        .collect())
}

pub async fn create_notification_rule(
    pool: &SqlitePool,
    entity_id: &str,
    trigger: &str,
    target_url: &str,
    active: bool,
) -> Result<NotificationRule> {
    require_entity(pool, entity_id).await?;
    validate_notification_rule(trigger, target_url)?;
    let id = format!("{entity_id}_rule_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _notification_rule (id, entity_id, trigger, target_url, active) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(trigger)
    .bind(target_url.trim())
    .bind(i64::from(active))
    .execute(pool)
    .await?;
    get_notification_rule(pool, &id).await
}

pub async fn get_notification_rule(pool: &SqlitePool, id: &str) -> Result<NotificationRule> {
    let row = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        "SELECT id, entity_id, trigger, target_url, active, created_at FROM _notification_rule WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("notification rule not found: {id}")))?;
    let (id, entity_id, trigger, target_url, active, created_at) = row;
    Ok(notification_rule_row(
        id, entity_id, trigger, target_url, active, created_at,
    ))
}

pub async fn update_notification_rule(
    pool: &SqlitePool,
    id: &str,
    trigger: Option<&str>,
    target_url: Option<&str>,
    active: Option<bool>,
) -> Result<NotificationRule> {
    let existing = get_notification_rule(pool, id).await?;
    let next_trigger = trigger.unwrap_or(&existing.trigger).to_string();
    let next_url = target_url.unwrap_or(&existing.target_url).to_string();
    let next_active = active.unwrap_or(existing.active);
    validate_notification_rule(&next_trigger, &next_url)?;
    let result = sqlx::query(
        "UPDATE _notification_rule SET trigger = ?, target_url = ?, active = ? WHERE id = ?",
    )
    .bind(&next_trigger)
    .bind(next_url.trim())
    .bind(i64::from(next_active))
    .bind(id)
    .execute(pool)
    .await?;
    debug_assert_eq!(result.rows_affected(), 1);
    get_notification_rule(pool, id).await
}

pub async fn delete_notification_rule(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _notification_rule WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("notification rule not found: {id}")).into());
    }
    Ok(())
}

/// Saved reports (Phase 4). Config shape: `{ "group_by": "<field>", "chart_type": "bar|pie" }`.
/// Admin-managed; users read reports for entities they can view.
fn validate_report_config(config: &Value) -> Result<()> {
    let object = config
        .as_object()
        .ok_or_else(|| AppError::BadRequest("report config must be a JSON object".to_string()))?;
    let group_by = object
        .get("group_by")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            AppError::BadRequest("report config requires a group_by field".to_string())
        })?;
    let _ = group_by;
    if let Some(chart_type) = object.get("chart_type") {
        let chart_type = chart_type
            .as_str()
            .ok_or_else(|| AppError::BadRequest("chart_type must be a string".to_string()))?;
        if chart_type != "bar" && chart_type != "pie" {
            return Err(AppError::BadRequest(format!("unknown chart_type: {chart_type}")).into());
        }
    }
    Ok(())
}

fn report_row(
    id: String,
    entity_id: String,
    name: String,
    config: String,
    created_by: Option<String>,
    created_at: String,
) -> Result<Report> {
    Ok(Report {
        id,
        entity_id,
        name,
        config: serde_json::from_str(&config)?,
        created_by,
        created_at,
    })
}

pub async fn list_reports(pool: &SqlitePool, entity_id: &str) -> Result<Vec<Report>> {
    require_entity(pool, entity_id).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, entity_id, name, config, created_by, created_at FROM _report WHERE entity_id = ? ORDER BY name",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|(id, entity_id, name, config, created_by, created_at)| {
            report_row(id, entity_id, name, config, created_by, created_at)
        })
        .collect()
}

pub async fn create_report(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    config: &Value,
    created_by: Option<&str>,
) -> Result<Report> {
    require_entity(pool, entity_id).await?;
    if name.trim().is_empty() {
        anyhow::bail!("report name is required");
    }
    validate_report_config(config)?;
    let id = format!("{entity_id}_report_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _report (id, entity_id, name, config, created_by) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(entity_id)
    .bind(name.trim())
    .bind(config.to_string())
    .bind(created_by)
    .execute(pool)
    .await?;
    get_report(pool, &id).await
}

pub async fn get_report(pool: &SqlitePool, id: &str) -> Result<Report> {
    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, entity_id, name, config, created_by, created_at FROM _report WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("report not found: {id}")))?;
    let (id, entity_id, name, config, created_by, created_at) = row;
    report_row(id, entity_id, name, config, created_by, created_at)
}

pub async fn delete_report(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _report WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("report not found: {id}")).into());
    }
    Ok(())
}

fn chrono_nanos() -> i64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    // Monotonic suffix: parallel tests on coarse Windows clocks can
    // otherwise generate duplicate ids within the same tick.
    (millis * 1_000_000 + COUNTER.fetch_add(1, Ordering::Relaxed) % 1_000_000) as i64
}

pub async fn list_notification_deliveries(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<NotificationDeliveryList> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _notification_delivery")
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, Option<String>, String)>(
        "SELECT id, rule_id, document_id, action, payload, status, attempts, last_error, created_at FROM _notification_delivery ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let mut items = Vec::new();
    for (id, rule_id, document_id, action, payload, status, attempts, last_error, created_at) in
        rows
    {
        items.push(NotificationDelivery {
            id,
            rule_id,
            document_id,
            action,
            payload: serde_json::from_str(&payload)?,
            status,
            attempts,
            last_error,
            created_at,
        });
    }
    Ok(NotificationDeliveryList { items, total })
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

pub async fn update_entity(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    label: &str,
    module: Option<&str>,
) -> Result<Entity> {
    if name.trim().is_empty() || label.trim().is_empty() {
        bail!("name and label are required");
    }
    let result =
        sqlx::query("UPDATE _meta_entity SET name = ?, label = ?, module = ? WHERE id = ?")
            .bind(name)
            .bind(label)
            .bind(module.map(str::trim).filter(|s| !s.is_empty()))
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
        module: module
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
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

#[derive(Debug, Clone, Default)]
pub struct FieldRules {
    pub is_unique: bool,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub min_length: Option<i64>,
    pub max_length: Option<i64>,
    pub default_value: Option<String>,
    pub auto_number_prefix: Option<String>,
    pub auto_number_width: Option<i64>,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_field(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
    ref_entity: Option<&str>,
    computed_expr: Option<&str>,
) -> Result<Field> {
    create_field_with_rules(
        pool,
        entity_id,
        name,
        field_type,
        required,
        is_status,
        ref_entity,
        computed_expr,
        &FieldRules::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn create_field_with_rules(
    pool: &SqlitePool,
    entity_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
    ref_entity: Option<&str>,
    computed_expr: Option<&str>,
    rules: &FieldRules,
) -> Result<Field> {
    validate_field_name(name)?;
    validate_field_type(field_type)?;
    validate_status_field(pool, entity_id, None, field_type, is_status).await?;
    validate_reference_field(pool, entity_id, field_type, ref_entity).await?;
    validate_computed_field(field_type, computed_expr)?;
    validate_field_rules(field_type, rules)?;
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
        "INSERT INTO _meta_field (id, entity_id, name, type, required, is_status, position, ref_entity, computed_expr, is_unique, min_value, max_value, pattern, min_length, max_length, default_value, auto_number_prefix, auto_number_width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&field_id)
    .bind(entity_id)
    .bind(name)
    .bind(field_type)
    .bind(required as i64)
    .bind(is_status as i64)
    .bind(position)
    .bind(ref_entity.map(str::trim).filter(|s| !s.is_empty()))
    .bind(computed_expr.map(str::trim).filter(|s| !s.is_empty()))
    .bind(rules.is_unique as i64)
    .bind(rules.min_value)
    .bind(rules.max_value)
    .bind(rules.pattern.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.min_length)
    .bind(rules.max_length)
    .bind(rules.default_value.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.auto_number_prefix.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.auto_number_width)
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
        "SELECT id, name, type, required, is_status, position, ref_entity, computed_expr, is_unique, min_value, max_value, pattern, min_length, max_length, default_value, auto_number_prefix, auto_number_width FROM _meta_field WHERE id = ?",
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
        ref_entity: row.try_get("ref_entity")?,
        computed_expr: row.try_get("computed_expr")?,
        is_unique: row.try_get::<i64, _>("is_unique")? != 0,
        min_value: row.try_get("min_value")?,
        max_value: row.try_get("max_value")?,
        pattern: row.try_get("pattern")?,
        min_length: row.try_get("min_length")?,
        max_length: row.try_get("max_length")?,
        default_value: row.try_get("default_value")?,
        auto_number_prefix: row.try_get("auto_number_prefix")?,
        auto_number_width: row.try_get("auto_number_width")?,
        options,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn update_field(
    pool: &SqlitePool,
    field_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
    ref_entity: Option<&str>,
    computed_expr: Option<&str>,
) -> Result<Field> {
    update_field_with_rules(
        pool,
        field_id,
        name,
        field_type,
        required,
        is_status,
        ref_entity,
        computed_expr,
        &FieldRules::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update_field_with_rules(
    pool: &SqlitePool,
    field_id: &str,
    name: &str,
    field_type: &str,
    required: bool,
    is_status: bool,
    ref_entity: Option<&str>,
    computed_expr: Option<&str>,
    rules: &FieldRules,
) -> Result<Field> {
    validate_field_name(name)?;
    validate_field_type(field_type)?;
    let entity_id: String = sqlx::query_scalar("SELECT entity_id FROM _meta_field WHERE id = ?")
        .bind(field_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("field not found: {field_id}")))?;
    validate_status_field(pool, &entity_id, Some(field_id), field_type, is_status).await?;
    validate_reference_field(pool, &entity_id, field_type, ref_entity).await?;
    validate_computed_field(field_type, computed_expr)?;
    validate_field_rules(field_type, rules)?;
    let result = sqlx::query(
        "UPDATE _meta_field SET name = ?, type = ?, required = ?, is_status = ?, ref_entity = ?, computed_expr = ?, is_unique = ?, min_value = ?, max_value = ?, pattern = ?, min_length = ?, max_length = ?, default_value = ?, auto_number_prefix = ?, auto_number_width = ? WHERE id = ?",
    )
    .bind(name)
    .bind(field_type)
    .bind(required as i64)
    .bind(is_status as i64)
    .bind(ref_entity.map(str::trim).filter(|s| !s.is_empty()))
    .bind(computed_expr.map(str::trim).filter(|s| !s.is_empty()))
    .bind(rules.is_unique as i64)
    .bind(rules.min_value)
    .bind(rules.max_value)
    .bind(rules.pattern.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.min_length)
    .bind(rules.max_length)
    .bind(rules.default_value.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.auto_number_prefix.clone().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
    .bind(rules.auto_number_width)
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
    metadata::validate_field_type(field_type)
}

async fn validate_reference_field(
    pool: &SqlitePool,
    entity_id: &str,
    field_type: &str,
    ref_entity: Option<&str>,
) -> Result<()> {
    if field_type != "reference" {
        if ref_entity
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            return Err(AppError::BadRequest(
                "ref_entity is only valid for reference fields".into(),
            )
            .into());
        }
        return Ok(());
    }
    let target = ref_entity
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("reference fields require a ref_entity".into()))?;
    if target == entity_id {
        return Err(
            AppError::BadRequest("reference field cannot point at its own entity".into()).into(),
        );
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
        .bind(target)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(AppError::NotFound(format!("entity not found: {target}")).into());
    }
    Ok(())
}

fn field_rules_from_definition(field: &Value) -> FieldRules {
    let rules = field.get("rules");
    FieldRules {
        is_unique: field
            .get("unique")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || rules
                .and_then(|rules| rules.get("unique"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        min_value: field
            .get("min_value")
            .and_then(Value::as_f64)
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("min_value"))
                    .and_then(Value::as_f64)
            })
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("min"))
                    .and_then(Value::as_f64)
            }),
        max_value: field
            .get("max_value")
            .and_then(Value::as_f64)
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("max_value"))
                    .and_then(Value::as_f64)
            })
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("max"))
                    .and_then(Value::as_f64)
            }),
        pattern: field
            .get("pattern")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("pattern"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            }),
        min_length: field.get("min_length").and_then(Value::as_i64).or_else(|| {
            rules
                .and_then(|rules| rules.get("min_length"))
                .and_then(Value::as_i64)
        }),
        max_length: field.get("max_length").and_then(Value::as_i64).or_else(|| {
            rules
                .and_then(|rules| rules.get("max_length"))
                .and_then(Value::as_i64)
        }),
        default_value: field
            .get("default")
            .and_then(|value| match value {
                Value::String(text) => Some(text.clone()),
                Value::Number(number) => Some(number.to_string()),
                Value::Bool(flag) => Some(flag.to_string()),
                _ => None,
            })
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("default"))
                    .and_then(|value| match value {
                        Value::String(text) => Some(text.clone()),
                        Value::Number(number) => Some(number.to_string()),
                        Value::Bool(flag) => Some(flag.to_string()),
                        _ => None,
                    })
            }),
        auto_number_prefix: field
            .get("auto_number")
            .and_then(|value| value.get("prefix"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("auto_number"))
                    .and_then(|value| value.get("prefix"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            }),
        auto_number_width: field
            .get("auto_number")
            .and_then(|value| value.get("width"))
            .and_then(Value::as_i64)
            .or_else(|| {
                rules
                    .and_then(|rules| rules.get("auto_number"))
                    .and_then(|value| value.get("width"))
                    .and_then(Value::as_i64)
            }),
    }
}

fn validate_field_rules(field_type: &str, rules: &FieldRules) -> Result<()> {
    if let Some(min) = rules.min_value {
        if !metadata::is_numeric_type(field_type) {
            return Err(
                AppError::BadRequest("min_value is only valid for number fields".into()).into(),
            );
        }
        if !min.is_finite() {
            return Err(AppError::BadRequest("min_value must be finite".into()).into());
        }
    }
    if let Some(max) = rules.max_value {
        if !metadata::is_numeric_type(field_type) {
            return Err(
                AppError::BadRequest("max_value is only valid for number fields".into()).into(),
            );
        }
        if !max.is_finite() {
            return Err(AppError::BadRequest("max_value must be finite".into()).into());
        }
    }
    if let (Some(min), Some(max)) = (rules.min_value, rules.max_value) {
        if min > max {
            return Err(AppError::BadRequest("min_value must be <= max_value".into()).into());
        }
    }
    if let Some(pattern) = rules
        .pattern
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if !metadata::is_text_type(field_type) {
            return Err(
                AppError::BadRequest("pattern is only valid for text fields".into()).into(),
            );
        }
        if pattern.len() > 500 {
            return Err(AppError::BadRequest("pattern is too long".into()).into());
        }
    }
    if (rules.min_length.is_some() || rules.max_length.is_some())
        && !metadata::is_text_type(field_type)
    {
        return Err(
            AppError::BadRequest("length limits are only valid for text fields".into()).into(),
        );
    }
    if let Some(min_length) = rules.min_length {
        if min_length < 0 {
            return Err(AppError::BadRequest("min_length must be >= 0".into()).into());
        }
    }
    if let Some(max_length) = rules.max_length {
        if max_length < 0 {
            return Err(AppError::BadRequest("max_length must be >= 0".into()).into());
        }
    }
    if let (Some(min_length), Some(max_length)) = (rules.min_length, rules.max_length) {
        if min_length > max_length {
            return Err(AppError::BadRequest("min_length must be <= max_length".into()).into());
        }
    }
    if let Some(default) = rules
        .default_value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if metadata::is_numeric_type(field_type) && default.parse::<f64>().is_err() {
            return Err(AppError::BadRequest("default_value must be a number".into()).into());
        }
        if metadata::is_boolean_type(field_type)
            && !matches!(default.to_ascii_lowercase().as_str(), "true" | "false")
        {
            return Err(AppError::BadRequest("default_value must be true or false".into()).into());
        }
    }
    if rules.auto_number_prefix.is_some() || rules.auto_number_width.is_some() {
        if !metadata::is_text_type(field_type) {
            return Err(
                AppError::BadRequest("auto_number is only valid for text fields".into()).into(),
            );
        }
        if let Some(width) = rules.auto_number_width {
            if !(1..=10).contains(&width) {
                return Err(AppError::BadRequest("auto_number width must be 1..=10".into()).into());
            }
        }
    }
    Ok(())
}

fn validate_computed_field(field_type: &str, computed_expr: Option<&str>) -> Result<()> {
    if field_type != "computed" {
        if computed_expr
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some()
        {
            return Err(AppError::BadRequest(
                "computed_expr is only valid for computed fields".into(),
            )
            .into());
        }
        return Ok(());
    }
    let expr = computed_expr
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("computed fields require a computed_expr".into()))?;
    if !expr.contains('{') || !expr.contains('}') {
        return Err(AppError::BadRequest(
            "computed_expr must reference fields like {title}".into(),
        )
        .into());
    }
    Ok(())
}

/// Template interpolation for computed fields: `{field_name}` placeholders
/// are replaced with the payload value (missing = empty string).
pub fn compute_field_value(expr: &str, payload: &Value) -> String {
    let mut output = String::with_capacity(expr.len());
    let mut rest = expr;
    while let Some(start) = rest.find('{') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                let key = after[..end].trim();
                let value = payload
                    .get(key)
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Null => String::new(),
                        other => other.to_string().trim_matches('"').to_string(),
                    })
                    .unwrap_or_default();
                output.push_str(&value);
                rest = &after[end + 1..];
            }
            None => {
                output.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
    }
    output.push_str(rest);
    output
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
    let filtered = filter_hidden_payload(pool, entity_id, payload, role).await?;
    let fields = list_fields(pool, entity_id).await?;
    let with_defaults = apply_field_defaults(&fields, &filtered);
    let mut with_defaults_object = with_defaults.as_object().cloned().unwrap_or_default();
    allocate_auto_numbers(pool, entity_id, &mut with_defaults_object).await?;
    let with_defaults = Value::Object(with_defaults_object);
    validate_payload_for_role(pool, entity_id, &with_defaults, role).await?;
    let payload = with_defaults;
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
    enqueue_automation_deliveries(&mut tx, entity_id, id, "create", &payload, actor).await?;
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
    let entity_id: String = row.try_get("entity_id")?;
    let payload: Value = serde_json::from_str(&row.try_get::<String, _>("payload")?)?;
    let fields = list_fields(pool, &entity_id).await?;
    Ok(Document {
        id: row.try_get("id")?,
        entity_id,
        payload: apply_computed_fields(&fields, &payload),
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
        let with_computed = apply_computed_fields(&fields, &payload);
        items.push(Document {
            id: row.try_get("id")?,
            entity_id: row.try_get("entity_id")?,
            payload: redact_payload(&with_computed, &viewable),
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
    validate_payload_for_role_excluding(pool, &existing.entity_id, &merged, role, Some(id)).await?;
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
    enqueue_automation_deliveries(&mut tx, &existing.entity_id, id, "update", &payload, actor)
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
        .ok_or_else(|| AppError::BadRequest("document has no status".into()))?
        .to_string();
    let target: Option<String> = sqlx::query_scalar(
        "SELECT to_state FROM _workflow_transition WHERE entity_id = ? AND from_state = ? AND action = ?",
    )
    .bind(&existing.entity_id)
    .bind(&current)
    .bind(action)
    .fetch_optional(pool)
    .await?;
    let target = target.ok_or_else(|| {
        AppError::BadRequest(format!("invalid transition: {current} cannot {action}"))
    })?;
    check_field_permission(pool, &existing.entity_id, &status_field.name, role, true).await?;
    let mut next = existing.payload;
    next[status_field.name.as_str()] = Value::String(target.clone());
    // Stored payloads may carry a previously computed value (get_document
    // interpolates on read); strip computed keys before re-validating.
    if let Some(object) = next.as_object_mut() {
        for field in fields.iter().filter(|f| f.r#type == "computed") {
            object.remove(&field.name);
        }
    }
    validate_payload_for_role_excluding(pool, &existing.entity_id, &next, role, Some(id)).await?;
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
    enqueue_transition_deliveries(
        &mut tx,
        &existing.entity_id,
        id,
        action,
        &current,
        &target,
        &next,
        actor,
    )
    .await?;
    enqueue_automation_deliveries(&mut tx, &existing.entity_id, id, "transition", &next, actor)
        .await?;
    tx.commit().await?;
    get_document(pool, id).await
}

/// Enqueue one pending webhook delivery per active transition rule. Runs
/// inside the transition transaction so a delivery row always matches its
/// audit row. Never performs network I/O; the worker sends later.
#[allow(clippy::too_many_arguments)]
async fn enqueue_transition_deliveries(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entity_id: &str,
    doc_id: &str,
    action: &str,
    from_state: &str,
    to_state: &str,
    payload: &Value,
    actor: Option<&str>,
) -> Result<()> {
    let rules: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM _notification_rule WHERE entity_id = ? AND trigger = 'transition' AND active != 0",
    )
    .bind(entity_id)
    .fetch_all(&mut **tx)
    .await?;
    for (rule_id,) in rules {
        let body = serde_json::json!({
            "entity_id": entity_id,
            "document_id": doc_id,
            "action": action,
            "from_state": from_state,
            "to_state": to_state,
            "actor": actor,
            "payload": payload,
        });
        let body_string = body.to_string();
        let body_string = if body_string.len() > 1_000_000 {
            serde_json::json!({
                "entity_id": entity_id,
                "document_id": doc_id,
                "action": action,
                "from_state": from_state,
                "to_state": to_state,
                "actor": actor,
                "truncated": true,
            })
            .to_string()
        } else {
            body_string
        };
        sqlx::query(
            "INSERT INTO _notification_delivery (id, rule_id, document_id, action, payload) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(format!("{doc_id}-{action}-{}", chrono_nanos()))
        .bind(rule_id)
        .bind(doc_id)
        .bind(action)
        .bind(body_string)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Enqueue one pending webhook delivery per active generic automation.
/// Runs inside the caller's transaction so the delivery row always matches
/// its audit row. Reuses the 1MB payload cap from transition deliveries.
async fn enqueue_automation_deliveries(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entity_id: &str,
    doc_id: &str,
    trigger: &str,
    payload: &Value,
    actor: Option<&str>,
) -> Result<()> {
    let automations: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, target_url FROM _automation WHERE entity_id = ? AND trigger = ? AND active != 0",
    )
    .bind(entity_id)
    .bind(trigger)
    .fetch_all(&mut **tx)
    .await?;
    for (automation_id, target_url) in automations {
        let body = serde_json::json!({
            "entity_id": entity_id,
            "document_id": doc_id,
            "trigger": trigger,
            "target_url": target_url,
            "actor": actor,
            "payload": payload,
        });
        let body_string = body.to_string();
        let body_string = if body_string.len() > 1_000_000 {
            serde_json::json!({
                "entity_id": entity_id,
                "document_id": doc_id,
                "trigger": trigger,
                "target_url": target_url,
                "actor": actor,
                "truncated": true,
            })
            .to_string()
        } else {
            body_string
        };
        sqlx::query(
            "INSERT INTO _notification_delivery (id, rule_id, document_id, action, payload) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(format!("{doc_id}-{trigger}-automation-{}", chrono_nanos()))
        .bind(automation_id)
        .bind(doc_id)
        .bind(trigger)
        .bind(body_string)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

// --- Generic Action Engine (transactional, audited, evented) ---

#[derive(Debug, Serialize)]
pub struct ModuleAction {
    pub id: String,
    pub entity_id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleActionResult {
    pub action: ModuleAction,
    pub document: Document,
}

fn module_action_definition(entity_id: &str, action_id: &str) -> Result<ModuleAction> {
    let (kind, label) = match action_id {
        "transition" => ("transition", "Transition"),
        "create" => ("create", "Create"),
        "update" => ("update", "Update"),
        "post" => ("post", "Post"),
        "allocate" => ("allocate", "Allocate"),
        "convert" => ("convert", "Convert"),
        "confirm" => ("confirm", "Confirm"),
        "complete" => ("complete", "Complete"),
        "close" => ("close", "Close"),
        _ => {
            return Err(AppError::BadRequest(format!("unknown action: {action_id}")).into());
        }
    };
    Ok(ModuleAction {
        id: action_id.to_string(),
        entity_id: entity_id.to_string(),
        kind: kind.to_string(),
        label: label.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_module_action(
    pool: &SqlitePool,
    entity_id: &str,
    action_id: &str,
    document_id: Option<&str>,
    payload: Option<&Value>,
    actor: Option<&str>,
    expected_updated_at: Option<&str>,
    role: &str,
) -> Result<ModuleActionResult> {
    require_entity(pool, entity_id).await?;
    check_permission(pool, entity_id, role, true).await?;
    let action = module_action_definition(entity_id, action_id)?;
    let document = match action.kind.as_str() {
        "transition" => {
            let (doc_id, transition) = match (document_id, payload) {
                (Some(doc_id), Some(payload)) => (
                    doc_id,
                    payload
                        .get("action")
                        .and_then(Value::as_str)
                        .unwrap_or(action_id),
                ),
                (Some(doc_id), None) => (doc_id, action_id),
                _ => {
                    return Err(
                        AppError::BadRequest("transition requires a document id".into()).into(),
                    );
                }
            };
            transition_document_as_role(pool, doc_id, transition, actor, expected_updated_at, role)
                .await?
        }
        "create" => {
            let body = payload
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            let new_id = document_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("{entity_id}_{}", chrono_nanos()));
            create_document_as_role(pool, &new_id, entity_id, &body, actor, role).await?
        }
        "update" => {
            let doc_id = document_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("update requires a document id".into()))?;
            let body = payload
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            update_document_as_role(pool, doc_id, &body, actor, expected_updated_at, role).await?
        }
        "post" | "allocate" | "convert" | "confirm" | "complete" | "close" => {
            let doc_id = document_id
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    AppError::BadRequest(format!("{} requires a document id", action.kind))
                })?;
            let existing = get_document(pool, doc_id).await?;
            if existing.entity_id != entity_id {
                return Err(
                    AppError::BadRequest("document belongs to another entity".into()).into(),
                );
            }
            let computed_names: Vec<String> = list_fields(pool, entity_id)
                .await?
                .iter()
                .filter(|f| f.r#type == "computed")
                .map(|f| f.name.clone())
                .collect();
            let mut next = existing.payload.clone();
            if let Some(object) = next.as_object_mut() {
                if let Some(extra) = payload.and_then(Value::as_object) {
                    for (key, value) in extra {
                        object.insert(key.clone(), value.clone());
                    }
                }
                for name in &computed_names {
                    object.remove(name);
                }
            }
            validate_payload_for_role_excluding(pool, entity_id, &next, role, Some(doc_id)).await?;
            let next_string = next.to_string();
            let mut tx = pool.begin().await?;
            let result = sqlx::query(
                "UPDATE _doc SET payload = ?, updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now') WHERE id = ? AND (? IS NULL OR updated_at = ?)",
            )
            .bind(&next_string)
            .bind(doc_id)
            .bind(expected_updated_at)
            .bind(expected_updated_at)
            .execute(&mut *tx)
            .await?;
            if result.rows_affected() == 0 {
                return Err(AppError::Conflict(format!(
                    "stale record: {doc_id} was modified by another user"
                ))
                .into());
            }
            sqlx::query(
                "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload, actor) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(audit_id(doc_id, &action.kind))
            .bind(entity_id)
            .bind(doc_id)
            .bind(action.kind.clone())
            .bind(next_string.clone())
            .bind(actor)
            .execute(&mut *tx)
            .await?;
            enqueue_automation_deliveries(&mut tx, entity_id, doc_id, &action.kind, &next, actor)
                .await?;
            tx.commit().await?;
            get_document(pool, doc_id).await?
        }
        _ => {
            return Err(AppError::BadRequest(format!("unknown action: {action_id}")).into());
        }
    };
    Ok(ModuleActionResult { action, document })
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

/// Report aggregation: count documents grouped by a select/status field.
/// Only `select` fields (including the status field) are allowed, keeping
/// bucket cardinality low. Hidden fields are rejected (400) rather than
/// leaking existence.
pub async fn report_aggregate_as_role(
    pool: &SqlitePool,
    entity_id: &str,
    group_by: &str,
    role: &str,
) -> Result<Vec<StatusCount>> {
    let fields = list_fields(pool, entity_id).await?;
    let Some(field) = fields.iter().find(|f| f.name == group_by) else {
        return Err(AppError::BadRequest(format!("unknown field: {group_by}")).into());
    };
    if field.r#type != "select" {
        return Err(AppError::BadRequest(format!(
            "group-by field must be a select field: {group_by}"
        ))
        .into());
    }
    check_field_permission(pool, entity_id, &field.name, role, false).await?;
    let query = format!(
        "SELECT json_extract(payload, '$.{}'), COUNT(*) FROM _doc WHERE entity_id = ? GROUP BY json_extract(payload, '$.{}') ORDER BY 1",
        field.name, field.name
    );
    let rows = sqlx::query_as::<_, (Option<String>, i64)>(&query)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(status, count)| StatusCount {
            status: status.unwrap_or_default(),
            count,
        })
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
    delete_document_as_role(pool, id, actor, "admin").await
}

pub async fn delete_document_as_role(
    pool: &SqlitePool,
    id: &str,
    actor: Option<&str>,
    role: &str,
) -> Result<()> {
    let existing = get_document(pool, id).await?;
    check_permission(pool, &existing.entity_id, role, true).await?;
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
    enqueue_automation_deliveries(
        &mut tx,
        &existing.entity_id,
        id,
        "delete",
        &existing.payload,
        actor,
    )
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

/// Record comments (chatter write). Any role with view access can read;
/// creating requires edit access. Bodies are trimmed, 1..=2000 chars.
pub async fn list_doc_comments_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    limit: i64,
    offset: i64,
    role: &str,
) -> Result<DocCommentList> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, false).await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc_comment WHERE doc_id = ?")
        .bind(doc_id)
        .fetch_one(pool)
        .await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, body, actor, created_at FROM _doc_comment WHERE doc_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(doc_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(DocCommentList {
        items: rows
            .into_iter()
            .map(
                |(id, entity_id, doc_id, body, actor, created_at)| DocComment {
                    id,
                    entity_id,
                    doc_id,
                    body,
                    actor,
                    created_at,
                },
            )
            .collect(),
        total,
    })
}

pub async fn create_doc_comment_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    body: &str,
    role: &str,
    actor: Option<&str>,
) -> Result<DocComment> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, true).await?;
    let body = body.trim();
    if body.is_empty() {
        return Err(AppError::BadRequest("comment body is required".into()).into());
    }
    if body.chars().count() > 2000 {
        return Err(
            AppError::BadRequest("comment body must be at most 2000 characters".into()).into(),
        );
    }
    let id = format!("{doc_id}_comment_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _doc_comment (id, entity_id, doc_id, body, actor) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&document.entity_id)
    .bind(doc_id)
    .bind(body)
    .bind(actor)
    .execute(pool)
    .await?;
    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, body, actor, created_at FROM _doc_comment WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(pool)
    .await?;
    let (id, entity_id, doc_id, body, actor, created_at) = row;
    Ok(DocComment {
        id,
        entity_id,
        doc_id,
        body,
        actor,
        created_at,
    })
}

/// Document followers: any role with view access can list; toggling
/// requires an actor identity and view access. Toggle is idempotent.
pub async fn list_doc_followers_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    role: &str,
    actor: Option<&str>,
) -> Result<DocFollowerList> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, false).await?;
    let followers: Vec<String> =
        sqlx::query_scalar("SELECT actor FROM _doc_follower WHERE doc_id = ? ORDER BY actor")
            .bind(doc_id)
            .fetch_all(pool)
            .await?;
    let total = followers.len() as i64;
    let is_following = actor
        .map(|a| followers.iter().any(|f| f == a))
        .unwrap_or(false);
    Ok(DocFollowerList {
        followers,
        total,
        is_following,
    })
}

pub async fn toggle_doc_follower_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    role: &str,
    actor: Option<&str>,
) -> Result<DocFollowerList> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, false).await?;
    let actor = actor
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Unauthorized("missing actor identity".into()))?;
    let existing: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM _doc_follower WHERE doc_id = ? AND actor = ?)",
    )
    .bind(doc_id)
    .bind(actor)
    .fetch_one(pool)
    .await?;
    if existing {
        sqlx::query("DELETE FROM _doc_follower WHERE doc_id = ? AND actor = ?")
            .bind(doc_id)
            .bind(actor)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("INSERT INTO _doc_follower (doc_id, actor) VALUES (?, ?)")
            .bind(doc_id)
            .bind(actor)
            .execute(pool)
            .await?;
    }
    list_doc_followers_as_role(pool, doc_id, role, Some(actor)).await
}

/// Document activities: any role with view access can list; creating and
/// toggling done require edit access. Title is trimmed, 1..=200 chars;
/// due_date must be YYYY-MM-DD when present.
pub async fn list_doc_activities_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    role: &str,
) -> Result<DocActivityList> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, false).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, i64, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, title, due_date, assignee, done, actor, created_at FROM _doc_activity WHERE doc_id = ? ORDER BY done, due_date, created_at DESC, id DESC",
    )
    .bind(doc_id)
    .fetch_all(pool)
    .await?;
    let total = rows.len() as i64;
    let open = rows.iter().filter(|row| row.6 == 0).count() as i64;
    Ok(DocActivityList {
        items: rows
            .into_iter()
            .map(
                |(id, entity_id, doc_id, title, due_date, assignee, done, actor, created_at)| {
                    DocActivity {
                        id,
                        entity_id,
                        doc_id,
                        title,
                        due_date,
                        assignee,
                        done: done != 0,
                        actor,
                        created_at,
                    }
                },
            )
            .collect(),
        total,
        open,
    })
}

pub async fn create_doc_activity_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    title: &str,
    due_date: Option<&str>,
    assignee: Option<&str>,
    role: &str,
    actor: Option<&str>,
) -> Result<DocActivity> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, true).await?;
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("activity title is required".into()).into());
    }
    if title.chars().count() > 200 {
        return Err(
            AppError::BadRequest("activity title must be at most 200 characters".into()).into(),
        );
    }
    let due_date = due_date.map(str::trim).filter(|s| !s.is_empty());
    if let Some(date) = due_date {
        if !valid_activity_date(date) {
            return Err(AppError::BadRequest("due_date must be YYYY-MM-DD".into()).into());
        }
    }
    let assignee = assignee.map(str::trim).filter(|s| !s.is_empty());
    let id = format!("{doc_id}_activity_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _doc_activity (id, entity_id, doc_id, title, due_date, assignee, actor) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&document.entity_id)
    .bind(doc_id)
    .bind(title)
    .bind(due_date)
    .bind(assignee)
    .bind(actor)
    .execute(pool)
    .await?;
    get_doc_activity(pool, &id).await
}

pub async fn toggle_doc_activity_as_role(
    pool: &SqlitePool,
    activity_id: &str,
    role: &str,
) -> Result<DocActivity> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT doc_id, entity_id FROM _doc_activity WHERE id = ?")
            .bind(activity_id)
            .fetch_optional(pool)
            .await?;
    let Some((doc_id, entity_id)) = row else {
        return Err(AppError::NotFound(format!("activity not found: {activity_id}")).into());
    };
    let _ = doc_id;
    check_permission(pool, &entity_id, role, true).await?;
    sqlx::query("UPDATE _doc_activity SET done = 1 - done WHERE id = ?")
        .bind(activity_id)
        .execute(pool)
        .await?;
    get_doc_activity(pool, activity_id).await
}

async fn get_doc_activity(pool: &SqlitePool, activity_id: &str) -> Result<DocActivity> {
    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, i64, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, title, due_date, assignee, done, actor, created_at FROM _doc_activity WHERE id = ?",
    )
    .bind(activity_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("activity not found: {activity_id}")))?;
    let (id, entity_id, doc_id, title, due_date, assignee, done, actor, created_at) = row;
    Ok(DocActivity {
        id,
        entity_id,
        doc_id,
        title,
        due_date,
        assignee,
        done: done != 0,
        actor,
        created_at,
    })
}

fn valid_activity_date(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let parsed: Vec<u32> = parts
        .iter()
        .filter_map(|part| part.parse::<u32>().ok())
        .collect();
    if parsed.len() != 3 {
        return false;
    }
    let (year, month, day) = (parsed[0], parsed[1], parsed[2]);
    if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    if year == 0 || month == 0 || month > 12 || day == 0 || day > 31 {
        return false;
    }
    true
}

/// Record attachments: any role with view access can list and download;
/// uploading and deleting require edit access. Files are stored as DB
/// blobs with a 5MB per-file cap; MIME allowlist blocks executables.
pub fn validate_attachment(filename: &str, content_type: &str, size: usize) -> Result<()> {
    let filename = filename.trim();
    if filename.is_empty() {
        return Err(AppError::BadRequest("filename is required".into()).into());
    }
    if filename.chars().count() > 255 {
        return Err(AppError::BadRequest("filename must be at most 255 characters".into()).into());
    }
    if size == 0 {
        return Err(AppError::BadRequest("file is empty".into()).into());
    }
    if size > ATTACHMENT_MAX_BYTES {
        return Err(AppError::BadRequest("file must be at most 5MB".into()).into());
    }
    let content_type = content_type.trim().to_lowercase();
    let allowed = content_type.starts_with("image/")
        || matches!(
            content_type.as_str(),
            "application/pdf"
                | "text/plain"
                | "text/csv"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
    if !allowed {
        return Err(AppError::BadRequest("unsupported file type".into()).into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn attachment_row(
    id: String,
    entity_id: String,
    doc_id: String,
    filename: String,
    content_type: String,
    size: i64,
    actor: Option<String>,
    created_at: String,
) -> DocAttachment {
    DocAttachment {
        id,
        entity_id,
        doc_id,
        filename,
        content_type,
        size,
        actor,
        created_at,
    }
}

pub async fn list_doc_attachments_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    role: &str,
) -> Result<DocAttachmentList> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, false).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, filename, content_type, size, actor, created_at FROM _doc_attachment WHERE doc_id = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(doc_id)
    .fetch_all(pool)
    .await?;
    let total = rows.len() as i64;
    Ok(DocAttachmentList {
        items: rows
            .into_iter()
            .map(
                |(id, entity_id, doc_id, filename, content_type, size, actor, created_at)| {
                    attachment_row(
                        id,
                        entity_id,
                        doc_id,
                        filename,
                        content_type,
                        size,
                        actor,
                        created_at,
                    )
                },
            )
            .collect(),
        total,
    })
}

pub async fn upload_doc_attachment_as_role(
    pool: &SqlitePool,
    doc_id: &str,
    filename: &str,
    content_type: &str,
    data: &[u8],
    role: &str,
    actor: Option<&str>,
) -> Result<DocAttachment> {
    let document = get_document(pool, doc_id)
        .await
        .map_err(|_| AppError::NotFound(format!("document not found: {doc_id}")))?;
    check_permission(pool, &document.entity_id, role, true).await?;
    validate_attachment(filename, content_type, data.len())?;
    let id = format!("{doc_id}_attachment_{}", chrono_nanos());
    sqlx::query(
        "INSERT INTO _doc_attachment (id, entity_id, doc_id, filename, content_type, size, data, actor) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&document.entity_id)
    .bind(doc_id)
    .bind(filename.trim())
    .bind(content_type.trim().to_lowercase())
    .bind(data.len() as i64)
    .bind(data)
    .bind(actor)
    .execute(pool)
    .await?;
    get_doc_attachment(pool, &id, role).await
}

pub async fn get_doc_attachment_data_as_role(
    pool: &SqlitePool,
    attachment_id: &str,
    role: &str,
) -> Result<DocAttachmentData> {
    let row: Option<(String, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT a.entity_id, a.filename, a.content_type, a.data FROM _doc_attachment a WHERE a.id = ?",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;
    let Some((entity_id, filename, content_type, data)) = row else {
        return Err(AppError::NotFound(format!("attachment not found: {attachment_id}")).into());
    };
    check_permission(pool, &entity_id, role, false).await?;
    Ok(DocAttachmentData {
        filename,
        content_type,
        data,
    })
}

pub async fn delete_doc_attachment_as_role(
    pool: &SqlitePool,
    attachment_id: &str,
    role: &str,
) -> Result<()> {
    let row: Option<String> =
        sqlx::query_scalar("SELECT entity_id FROM _doc_attachment WHERE id = ?")
            .bind(attachment_id)
            .fetch_optional(pool)
            .await?;
    let Some(entity_id) = row else {
        return Err(AppError::NotFound(format!("attachment not found: {attachment_id}")).into());
    };
    check_permission(pool, &entity_id, role, true).await?;
    sqlx::query("DELETE FROM _doc_attachment WHERE id = ?")
        .bind(attachment_id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn get_doc_attachment(
    pool: &SqlitePool,
    attachment_id: &str,
    role: &str,
) -> Result<DocAttachment> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>, String)>(
        "SELECT id, entity_id, doc_id, filename, content_type, size, actor, created_at FROM _doc_attachment WHERE id = ?",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("attachment not found: {attachment_id}")))?;
    let (id, entity_id, doc_id, filename, content_type, size, actor, created_at) = row;
    check_permission(pool, &entity_id, role, false).await?;
    Ok(attachment_row(
        id,
        entity_id,
        doc_id,
        filename,
        content_type,
        size,
        actor,
        created_at,
    ))
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
    validate_payload_for_role_excluding(pool, entity_id, payload, role, None).await
}

async fn validate_payload_for_role_excluding(
    pool: &SqlitePool,
    entity_id: &str,
    payload: &Value,
    role: &str,
    exclude_doc_id: Option<&str>,
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
        // Computed fields are derived on read; never accepted from clients.
        if field.r#type == "computed" {
            if object.contains_key(&field.name) {
                return Err(AppError::BadRequest(format!(
                    "computed field is read-only: {}",
                    field.name
                ))
                .into());
            }
            continue;
        }
        let value = object.get(&field.name);
        if field.required && value.is_none() {
            return Err(
                AppError::BadRequest(format!("missing required field: {}", field.name)).into(),
            );
        }
        if let Some(value) = value {
            validate_field_value(field, value)?;
            validate_field_rules_value(pool, entity_id, field, value, exclude_doc_id).await?;
            if field.r#type == "reference" {
                let target = field.ref_entity.as_deref().unwrap_or_default();
                let doc_id = value.as_str().unwrap_or_default();
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM _doc WHERE id = ? AND entity_id = ?)",
                )
                .bind(doc_id)
                .bind(target)
                .fetch_one(pool)
                .await?;
                if !exists {
                    return Err(AppError::BadRequest(format!(
                        "unknown reference: {} is not a {target}",
                        field.name
                    ))
                    .into());
                }
            }
        }
    }
    for key in object.keys() {
        if !visible.iter().any(|f| f.name == *key) {
            return Err(AppError::BadRequest(format!("unknown field: {key}")).into());
        }
    }
    Ok(())
}

async fn validate_field_rules_value(
    pool: &SqlitePool,
    entity_id: &str,
    field: &Field,
    value: &Value,
    exclude_doc_id: Option<&str>,
) -> Result<()> {
    if field.is_unique {
        if let Some(text) = value.as_str() {
            let query = match exclude_doc_id {
                Some(_) => "SELECT EXISTS(SELECT 1 FROM _doc WHERE entity_id = ? AND id != ? AND json_extract(payload, ?) = ?)",
                None => "SELECT EXISTS(SELECT 1 FROM _doc WHERE entity_id = ? AND json_extract(payload, ?) = ?)",
            };
            let path = format!("$.{}", field.name);
            let exists: bool = match exclude_doc_id {
                Some(exclude) => {
                    sqlx::query_scalar(query)
                        .bind(entity_id)
                        .bind(exclude)
                        .bind(path)
                        .bind(text)
                        .fetch_one(pool)
                        .await?
                }
                None => {
                    sqlx::query_scalar(query)
                        .bind(entity_id)
                        .bind(path)
                        .bind(text)
                        .fetch_one(pool)
                        .await?
                }
            };
            if exists {
                return Err(AppError::Conflict(format!(
                    "duplicate value for unique field: {}",
                    field.name
                ))
                .into());
            }
        } else if let Some(number) = value.as_f64() {
            let query = match exclude_doc_id {
                Some(_) => "SELECT EXISTS(SELECT 1 FROM _doc WHERE entity_id = ? AND id != ? AND json_extract(payload, ?) = ?)",
                None => "SELECT EXISTS(SELECT 1 FROM _doc WHERE entity_id = ? AND json_extract(payload, ?) = ?)",
            };
            let path = format!("$.{}", field.name);
            let exists: bool = match exclude_doc_id {
                Some(exclude) => {
                    sqlx::query_scalar(query)
                        .bind(entity_id)
                        .bind(exclude)
                        .bind(path)
                        .bind(number)
                        .fetch_one(pool)
                        .await?
                }
                None => {
                    sqlx::query_scalar(query)
                        .bind(entity_id)
                        .bind(path)
                        .bind(number)
                        .fetch_one(pool)
                        .await?
                }
            };
            if exists {
                return Err(AppError::Conflict(format!(
                    "duplicate value for unique field: {}",
                    field.name
                ))
                .into());
            }
        }
    }
    if matches!(field.r#type.as_str(), "number" | "currency") {
        if let Some(number) = value.as_f64() {
            if let Some(min) = field.min_value {
                if number < min {
                    return Err(AppError::BadRequest(format!(
                        "value below minimum for field {}: {number} < {min}",
                        field.name
                    ))
                    .into());
                }
            }
            if let Some(max) = field.max_value {
                if number > max {
                    return Err(AppError::BadRequest(format!(
                        "value above maximum for field {}: {number} > {max}",
                        field.name
                    ))
                    .into());
                }
            }
        }
    }
    if matches!(field.r#type.as_str(), "text" | "textarea") {
        if let Some(text) = value.as_str() {
            let length = text.chars().count() as i64;
            if let Some(min_length) = field.min_length {
                if length < min_length {
                    return Err(AppError::BadRequest(format!(
                        "value too short for field {}: {length} < {min_length}",
                        field.name
                    ))
                    .into());
                }
            }
            if let Some(max_length) = field.max_length {
                if length > max_length {
                    return Err(AppError::BadRequest(format!(
                        "value too long for field {}: {length} > {max_length}",
                        field.name
                    ))
                    .into());
                }
            }
            if let Some(pattern) = field
                .pattern
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                if !matches_simple_pattern(pattern, text) {
                    return Err(AppError::BadRequest(format!(
                        "value does not match pattern for field {}",
                        field.name
                    ))
                    .into());
                }
            }
        }
    }
    Ok(())
}

/// Minimal pattern matcher (no extra dependency): supports `^`, `$`, `.*`,
/// `.+`, literal prefixes/suffixes, and character classes like `[0-9]+`.
/// Anything else falls back to substring matching so rules stay predictable.
fn matches_simple_pattern(pattern: &str, text: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return true;
    }
    if pattern == ".*" || pattern == ".+" {
        return !text.is_empty() || pattern == ".*";
    }
    let anchored_start = pattern.starts_with('^');
    let anchored_end = pattern.ends_with('$');
    let core = pattern
        .strip_prefix('^')
        .unwrap_or(pattern)
        .strip_suffix('$')
        .unwrap_or_else(|| pattern.strip_prefix('^').unwrap_or(pattern));
    if core.contains("[0-9]") {
        let digits_only = core
            .replace("[0-9]+", "")
            .replace("[0-9]*", "")
            .replace("[0-9]", "");
        let has_plus = core.contains("[0-9]+");
        let letters: String = text.chars().filter(|c| !c.is_ascii_digit()).collect();
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if has_plus && digits.is_empty() {
            return false;
        }
        if digits_only.is_empty() {
            return text.chars().all(|c| c.is_ascii_digit()) && !text.is_empty();
        }
        if anchored_start && anchored_end {
            return text.starts_with(&digits_only)
                || text.ends_with(&digits_only)
                || letters == digits_only;
        }
        return text.contains(&digits_only);
    }
    if core.contains(".*") {
        let parts: Vec<&str> = core.split(".*").collect();
        if anchored_start && !text.starts_with(parts[0]) {
            return false;
        }
        if anchored_end && !text.ends_with(parts[parts.len() - 1]) {
            return false;
        }
        let mut rest = text;
        for (index, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            match rest.find(part) {
                Some(position) => {
                    if index == 0 && anchored_start && position != 0 {
                        return false;
                    }
                    rest = &rest[position + part.len()..];
                }
                None => return false,
            }
        }
        return true;
    }
    if anchored_start && anchored_end {
        return text == core;
    }
    if anchored_start {
        return text.starts_with(core);
    }
    if anchored_end {
        return text.ends_with(core);
    }
    text.contains(core)
}

fn field_default_payload(field: &Field) -> Option<Value> {
    if field.auto_number_prefix.is_some() || field.auto_number_width.is_some() {
        return None;
    }
    let default = field
        .default_value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    match field.r#type.as_str() {
        "number" | "currency" => default
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number),
        "checkbox" | "boolean" => match default.to_ascii_lowercase().as_str() {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        _ => Some(Value::String(default.to_string())),
    }
}

fn apply_field_defaults(fields: &[Field], payload: &Value) -> Value {
    let Some(object) = payload.as_object() else {
        return payload.clone();
    };
    let mut next = object.clone();
    for field in fields {
        if field.r#type == "computed" || next.contains_key(&field.name) {
            continue;
        }
        if let Some(default) = field_default_payload(field) {
            next.insert(field.name.clone(), default);
        }
    }
    Value::Object(next)
}

async fn allocate_auto_numbers(
    pool: &SqlitePool,
    entity_id: &str,
    payload: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    let fields = list_fields(pool, entity_id).await?;
    let numbered: Vec<(String, Option<String>, Option<i64>)> = fields
        .iter()
        .filter(|f| {
            f.r#type == "text" && (f.auto_number_prefix.is_some() || f.auto_number_width.is_some())
        })
        .filter(|f| !payload.contains_key(&f.name))
        .map(|f| {
            (
                f.name.clone(),
                f.auto_number_prefix.clone(),
                f.auto_number_width,
            )
        })
        .collect();
    for (name, prefix, width) in numbered {
        // Single-connection in-memory pools deadlock if a write tx opens
        // while this read is outstanding; allocate outside any tx.
        let next_value: i64 = sqlx::query_scalar(
            "INSERT INTO _number_counter (entity_id, field_name, next_value) VALUES (?, ?, 2) \
             ON CONFLICT(entity_id, field_name) DO UPDATE SET next_value = next_value + 1 \
             RETURNING next_value - 1",
        )
        .bind(entity_id)
        .bind(&name)
        .fetch_one(pool)
        .await?;
        let prefix = prefix.as_deref().unwrap_or("");
        let formatted = match width {
            Some(width) if width > 0 => {
                format!("{prefix}{next_value:0>width$}", width = width as usize)
            }
            _ => format!("{prefix}{next_value}"),
        };
        payload.insert(name, Value::String(formatted));
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
        "text" | "textarea" => value.is_string(),
        "number" | "currency" => value.is_number(),
        "checkbox" | "boolean" => value.is_boolean(),
        "date" => value.is_string(),
        "reference" => value.is_string(),
        "computed" => false,
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
