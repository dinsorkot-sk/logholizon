use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::{report, repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layout: Value,
    pub filters: Value,
    pub roles: Vec<String>,
    pub users: Vec<String>,
    pub active: bool,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub entity_id: String,
    pub config: Value,
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

fn list_strings(v: &str) -> Result<Vec<String>> {
    Ok(serde_json::from_str(v).unwrap_or_default())
}

type DashboardRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<String>,
    String,
    String,
);

fn row_to_dashboard(row: DashboardRow) -> Result<Dashboard> {
    let (
        id,
        name,
        description,
        layout,
        filters,
        roles,
        users,
        active,
        created_by,
        created_at,
        updated_at,
    ) = row;
    Ok(Dashboard {
        id,
        name,
        description,
        layout: serde_json::from_str(&layout)?,
        filters: serde_json::from_str(&filters)?,
        roles: list_strings(&roles)?,
        users: list_strings(&users)?,
        active: active != 0,
        created_by,
        created_at,
        updated_at,
    })
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Dashboard>> {
    let rows = sqlx::query_as::<_, (String,String,String,String,String,String,String,i64,Option<String>,String,String)>("SELECT id,name,description,layout,filters,roles,users,active,created_by,created_at,updated_at FROM _dashboard ORDER BY name")
        .fetch_all(pool).await?;
    rows.into_iter().map(row_to_dashboard).collect()
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Dashboard> {
    let row = sqlx::query_as::<_, (String,String,String,String,String,String,String,i64,Option<String>,String,String)>("SELECT id,name,description,layout,filters,roles,users,active,created_by,created_at,updated_at FROM _dashboard WHERE id=?")
        .bind(id).fetch_optional(pool).await?.ok_or_else(|| anyhow::anyhow!("dashboard not found: {id}"))?;
    row_to_dashboard(row)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &SqlitePool,
    name: &str,
    description: &str,
    layout: &Value,
    filters: &Value,
    roles: &[String],
    users: &[String],
    created_by: Option<&str>,
) -> Result<Dashboard> {
    if name.trim().is_empty() {
        anyhow::bail!("dashboard name is required");
    }
    let id = format!("dashboard_{}", repository::chrono_nanos_public());
    sqlx::query("INSERT INTO _dashboard (id,name,description,layout,filters,roles,users,created_by) VALUES (?,?,?,?,?,?,?,?)")
        .bind(&id).bind(name.trim()).bind(description).bind(layout.to_string()).bind(filters.to_string())
        .bind(serde_json::to_string(roles)?).bind(serde_json::to_string(users)?).bind(created_by).execute(pool).await?;
    get(pool, &id).await
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    description: &str,
    layout: &Value,
    filters: &Value,
    roles: &[String],
    users: &[String],
    active: bool,
) -> Result<Dashboard> {
    let result = sqlx::query("UPDATE _dashboard SET name=?,description=?,layout=?,filters=?,roles=?,users=?,active=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(name.trim()).bind(description).bind(layout.to_string()).bind(filters.to_string())
        .bind(serde_json::to_string(roles)?).bind(serde_json::to_string(users)?).bind(active as i64).bind(id).execute(pool).await?;
    if result.rows_affected() == 0 {
        anyhow::bail!("dashboard not found: {id}");
    }
    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM _dashboard WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        anyhow::bail!("dashboard not found: {id}");
    }
    Ok(())
}

pub fn can_view(d: &Dashboard, user_id: Option<&str>, role: &str) -> bool {
    if !d.active {
        return false;
    }
    if role == "admin" || d.roles.is_empty() && d.users.is_empty() {
        return true;
    }
    d.roles.iter().any(|r| r == role) || user_id.is_some_and(|u| d.users.iter().any(|x| x == u))
}

pub async fn run(pool: &SqlitePool, d: &Dashboard, role: &str) -> Result<Value> {
    let widgets = d.layout.as_array().cloned().unwrap_or_default();
    let mut output = Vec::with_capacity(widgets.len());
    for raw in widgets {
        let w: DashboardWidget = serde_json::from_value(raw)?;
        if !matches!(
            w.kind.as_str(),
            "kpi" | "table" | "list" | "bar" | "line" | "pie" | "area"
        ) {
            anyhow::bail!("unsupported dashboard widget kind: {}", w.kind);
        }
        let mut config: report::ReportConfig = serde_json::from_value(w.config.clone())?;
        if let Some(filters) = d.filters.as_object() {
            for (field, value) in filters {
                config.filters.push(report::ReportFilter {
                    field: field.clone(),
                    op: "eq".to_string(),
                    value: value.clone(),
                });
            }
        }
        let result = report::run(pool, &w.entity_id, &config, role).await?;
        output.push(json!({ "widget": w, "result": result }));
    }
    Ok(Value::Array(output))
}
