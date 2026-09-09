use anyhow::{bail, Result};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct AutomationExecution {
    pub id: String,
    pub automation_id: String,
    pub event_id: Option<String>,
    pub document_id: Option<String>,
    pub status: String,
    pub attempt: i64,
    pub error: Option<String>,
    pub result: Value,
    pub scheduled_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
}
pub async fn enqueue_scheduled(pool: &SqlitePool) -> Result<usize> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT id, schedule FROM _automation WHERE active != 0 AND trigger = 'schedule' AND schedule != '' AND schedule <= CURRENT_TIMESTAMP")
        .fetch_all(pool).await?;
    let mut n = 0;
    for (id, schedule) in rows {
        let execution_id = format!(
            "{id}-schedule-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        );
        let inserted=sqlx::query("INSERT INTO _automation_execution (id, automation_id, document_id, result) SELECT ?, id, NULL, '{}' FROM _automation WHERE id=? AND active!=0 AND schedule=?")
            .bind(&execution_id).bind(&id).bind(&schedule).execute(pool).await?.rows_affected();
        if inserted > 0 {
            sqlx::query(
                "UPDATE _automation SET schedule='', updated_at=CURRENT_TIMESTAMP WHERE id=?",
            )
            .bind(id)
            .execute(pool)
            .await?;
            n += 1;
        }
    }
    Ok(n)
}

pub async fn enqueue_events(pool: &SqlitePool) -> Result<usize> {
    let events: Vec<(String,String,Option<String>,String,String)> = sqlx::query_as("SELECT id, entity_id, document_id, event_type, payload FROM _event WHERE created_at >= datetime('now','-1 minute') ORDER BY created_at")
        .fetch_all(pool).await?;
    let mut n = 0;
    for (event_id, entity_id, document_id, event_type, payload) in events {
        let trigger = match event_type.as_str() {
            "record.created" => "create",
            "record.updated" => "update",
            "record.deleted" => "delete",
            _ => continue,
        };
        let automations: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, condition FROM _automation WHERE entity_id=? AND trigger=? AND active!=0",
        )
        .bind(&entity_id)
        .bind(trigger)
        .fetch_all(pool)
        .await?;
        for (automation_id, condition) in automations {
            if !condition.trim().is_empty() {
                let vars: HashMap<String, Value> = serde_json::from_str::<Value>(&payload)?
                    .as_object()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                if crate::formula::evaluate(&condition, &vars)? != Value::Bool(true) {
                    continue;
                }
            }
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM _automation_execution WHERE automation_id=? AND event_id=?",
            )
            .bind(&automation_id)
            .bind(&event_id)
            .fetch_one(pool)
            .await?;
            if exists == 0 {
                let id = format!("{automation_id}-{event_id}");
                sqlx::query("INSERT INTO _automation_execution (id,automation_id,event_id,document_id,result) VALUES (?,?,?,?,?)").bind(id).bind(&automation_id).bind(&event_id).bind(&document_id) .bind(&payload).execute(pool).await?;
                n += 1;
            }
        }
    }
    Ok(n)
}

pub async fn process_pending(pool: &SqlitePool) -> Result<usize> {
    let rows:Vec<(String,String,Option<String>,i64)>=sqlx::query_as("SELECT id,automation_id,document_id,attempt FROM _automation_execution WHERE status='pending' AND scheduled_at<=CURRENT_TIMESTAMP ORDER BY created_at LIMIT 20").fetch_all(pool).await?;
    let mut n = 0;
    for (id, automation_id, document_id, attempt) in rows {
        sqlx::query("UPDATE _automation_execution SET status='running', attempt=attempt+1, started_at=CURRENT_TIMESTAMP WHERE id=? AND status='pending'").bind(&id).execute(pool).await?;
        let a: (String, String, String, i64) = sqlx::query_as(
            "SELECT action,target_url,actions,max_attempts FROM _automation WHERE id=?",
        )
        .bind(&automation_id)
        .fetch_one(pool)
        .await?;
        let (action, target_url, actions_json, max_attempts) = a;
        let mut result = Value::Array(vec![]);
        let mut error = None;
        if action == "actions" {
            let list: Vec<Value> = serde_json::from_str(&actions_json).unwrap_or_default();
            if list.is_empty() {
                error = Some("automation action chain is empty".to_string());
            }
            for item in list {
                if error.is_some() {
                    break;
                }
                let aid = item
                    .get("action_id")
                    .and_then(Value::as_str)
                    .or_else(|| item.as_str());
                let Some(aid) = aid else {
                    error = Some("invalid action chain item".to_string());
                    break;
                };
                let default = Value::Object(Default::default());
                let payload = item.get("payload").unwrap_or(&default);
                match crate::repository::execute_module_action(
                    pool,
                    &a_entity(pool, &automation_id).await?,
                    aid,
                    document_id.as_deref(),
                    Some(payload),
                    Some("automation"),
                    None,
                    "admin",
                )
                .await
                {
                    Ok(r) => result
                        .as_array_mut()
                        .unwrap()
                        .push(serde_json::to_value(r)?),
                    Err(e) => error = Some(e.to_string()),
                };
            }
        } else if action == "webhook" || action == "notify" {
            result = serde_json::json!({"action":action,"target_url":target_url});
        } else {
            error = Some(format!("unsupported automation action: {action}"));
        }
        if let Some(e) = error {
            let next = attempt + 1;
            let status = if next >= max_attempts.max(1) {
                "failed"
            } else {
                "pending"
            };
            sqlx::query("UPDATE _automation_execution SET status=?,attempt=?,error=?,result=?,finished_at=CURRENT_TIMESTAMP WHERE id=?").bind(status).bind(next).bind(e).bind(result.to_string()).bind(id).execute(pool).await?;
        } else {
            sqlx::query("UPDATE _automation_execution SET status='succeeded',result=?,error=NULL,finished_at=CURRENT_TIMESTAMP WHERE id=?").bind(result.to_string()).bind(id).execute(pool).await?;
        }
        n += 1;
    }
    Ok(n)
}
async fn a_entity(pool: &SqlitePool, id: &str) -> Result<String> {
    Ok(
        sqlx::query_scalar("SELECT entity_id FROM _automation WHERE id=?")
            .bind(id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn update(
    pool: &SqlitePool,
    id: &str,
    condition: Option<&str>,
    schedule: Option<&str>,
    actions: Option<&Value>,
    max_attempts: Option<i64>,
    active: Option<bool>,
) -> Result<crate::repository::Automation> {
    let current = crate::repository::get_automation(pool, id).await?;
    let condition = condition.unwrap_or("").trim();
    if condition.len() > 2048 {
        bail!("automation condition is too long");
    }
    if !condition.is_empty() {
        crate::formula::validate_syntax(condition)?;
    }
    let schedule = schedule.unwrap_or("").trim();
    let actions = actions.cloned().unwrap_or(Value::Array(vec![])).to_string();
    let max = max_attempts.unwrap_or(3).clamp(1, 20);
    let active = active.unwrap_or(current.active);
    sqlx::query("UPDATE _automation SET condition=?,schedule=?,actions=?,max_attempts=?,active=?,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(condition).bind(schedule).bind(actions).bind(max).bind(active as i64).bind(id).execute(pool).await?;
    crate::repository::get_automation(pool, id).await
}
