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

pub async fn export_documents_csv(pool: &SqlitePool, entity_id: &str) -> Result<String> {
    let fields = list_fields(pool, entity_id).await?;
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
    let fields = list_fields(pool, entity_id).await?;
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
        if let Err(error) = validate_payload(pool, entity_id, &value).await {
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
) -> Result<ImportResult> {
    let preview = preview_documents_csv(pool, entity_id, input).await?;
    if !preview.errors.is_empty() {
        return Err(AppError::BadRequest(preview.errors.join("; ")).into());
    }
    let mut transaction = pool.begin().await?;
    let mut created = 0;
    let mut updated = 0;
    for row in preview.rows {
        let id = row["id"].as_str().unwrap();
        let payload = &row["payload"];
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id = ?)")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await?;
        if exists {
            let owner: String = sqlx::query_scalar("SELECT entity_id FROM _doc WHERE id = ?")
                .bind(id)
                .fetch_one(&mut *transaction)
                .await?;
            if owner != entity_id {
                return Err(
                    AppError::Conflict(format!("id belongs to another entity: {id}")).into(),
                );
            }
            sqlx::query("UPDATE _doc SET payload = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(payload.to_string())
                .bind(id)
                .execute(&mut *transaction)
                .await?;
            updated += 1;
        } else {
            sqlx::query("INSERT INTO _doc (id, entity_id, payload) VALUES (?, ?, ?)")
                .bind(id)
                .bind(entity_id)
                .bind(payload.to_string())
                .execute(&mut *transaction)
                .await?;
            created += 1;
        }
        sqlx::query(
            "INSERT INTO _audit_log (id, entity_id, doc_id, action, payload) VALUES (?, ?, ?, 'import', ?)",
        )
        .bind(audit_id(id, "import"))
        .bind(entity_id)
        .bind(id)
        .bind(payload.to_string())
        .execute(&mut *transaction)
        .await?;
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
}

pub async fn list_documents(
    pool: &SqlitePool,
    entity_id: &str,
    limit: i64,
    offset: i64,
    filter: &ListDocumentsFilter,
) -> Result<DocumentList> {
    use sqlx::Row;
    let fields = list_fields(pool, entity_id).await?;
    let mut where_sql = String::from("entity_id = ?");
    let mut params: Vec<String> = vec![entity_id.to_string()];

    if let Some(status) = filter.status.as_deref().filter(|s| !s.trim().is_empty()) {
        if let Some(status_field) = fields.iter().find(|f| f.is_status) {
            where_sql.push_str(&format!(
                " AND json_extract(payload, '$.{}') = ?",
                status_field.name
            ));
            params.push(status.to_string());
        }
    }
    if let Some(search) = filter.search.as_deref().filter(|s| !s.trim().is_empty()) {
        let like = format!("%{}%", search.trim());
        let mut clauses = vec!["id LIKE ?".to_string()];
        params.push(like.clone());
        for field in fields.iter().filter(|f| f.r#type == "text") {
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
        Some(name) if fields.iter().any(|f| f.name == name) => {
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
    let mut next = existing.payload;
    next[status_field.name.as_str()] = Value::String(target);
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
    let fields = list_fields(pool, entity_id).await?;
    let Some(status_field) = fields.iter().find(|f| f.is_status) else {
        return Ok(Vec::new());
    };
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
