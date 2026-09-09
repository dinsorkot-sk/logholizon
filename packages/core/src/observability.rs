use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize)]
pub struct ObservabilityLog {
    pub id: String,
    pub occurred_at: String,
    pub level: String,
    pub category: String,
    pub action: String,
    pub actor: Option<String>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub status_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub message: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ObservabilityFilter {
    pub category: Option<String>,
    pub level: Option<String>,
    pub actor: Option<String>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn record(
    pool: &SqlitePool,
    level: &str,
    category: &str,
    action: &str,
    actor: Option<&str>,
    request_id: Option<&str>,
    correlation_id: Option<&str>,
    target_type: Option<&str>,
    target_id: Option<&str>,
    status_code: Option<i64>,
    duration_ms: Option<i64>,
    message: &str,
    metadata: &Value,
) -> Result<()> {
    let id = format!("obs_{}", crate::repository::chrono_nanos_public());
    sqlx::query("INSERT INTO _observability_log (id,level,category,action,actor,request_id,correlation_id,target_type,target_id,status_code,duration_ms,message,metadata) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(id).bind(level).bind(category).bind(action).bind(actor).bind(request_id)
        .bind(correlation_id).bind(target_type).bind(target_id).bind(status_code)
        .bind(duration_ms).bind(message).bind(metadata.to_string()).execute(pool).await?;
    Ok(())
}

pub async fn list(
    pool: &SqlitePool,
    filter: &ObservabilityFilter,
    limit: i64,
    offset: i64,
) -> Result<(i64, Vec<ObservabilityLog>)> {
    let limit = limit.clamp(1, 200);
    let offset = offset.max(0);
    let mut where_sql = vec!["1=1".to_string()];
    let mut binds: Vec<String> = Vec::new();
    if let Some(v) = &filter.category {
        where_sql.push("category = ?".into());
        binds.push(v.clone());
    }
    if let Some(v) = &filter.level {
        where_sql.push("level = ?".into());
        binds.push(v.clone());
    }
    if let Some(v) = &filter.actor {
        where_sql.push("actor = ?".into());
        binds.push(v.clone());
    }
    if let Some(v) = &filter.request_id {
        where_sql.push("request_id = ?".into());
        binds.push(v.clone());
    }
    if let Some(v) = &filter.correlation_id {
        where_sql.push("correlation_id = ?".into());
        binds.push(v.clone());
    }
    let where_clause = where_sql.join(" AND ");
    let count_sql = format!("SELECT COUNT(*) FROM _observability_log WHERE {where_clause}");
    let mut q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        q = q.bind(b);
    }
    let total = q.fetch_one(pool).await?;
    let list_sql = format!("SELECT id,occurred_at,level,category,action,actor,request_id,correlation_id,target_type,target_id,status_code,duration_ms,message,metadata FROM _observability_log WHERE {where_clause} ORDER BY occurred_at DESC,id DESC LIMIT ? OFFSET ?");
    let mut q = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            String,
            String,
        ),
    >(&list_sql);
    for b in &binds {
        q = q.bind(b);
    }
    let rows = q.bind(limit).bind(offset).fetch_all(pool).await?;
    let items = rows
        .into_iter()
        .map(|r| ObservabilityLog {
            id: r.0,
            occurred_at: r.1,
            level: r.2,
            category: r.3,
            action: r.4,
            actor: r.5,
            request_id: r.6,
            correlation_id: r.7,
            target_type: r.8,
            target_id: r.9,
            status_code: r.10,
            duration_ms: r.11,
            message: r.12,
            metadata: serde_json::from_str(&r.13).unwrap_or(Value::Object(Default::default())),
        })
        .collect();
    Ok((total, items))
}
