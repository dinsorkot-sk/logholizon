use anyhow::Result;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::SqlitePool;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub subject: String,
    pub body: String,
    pub variables: Value,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct NotificationItem {
    pub id: String,
    pub user_id: String,
    pub channel: String,
    pub subject: String,
    pub body: String,
    pub data: Value,
    pub status: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub created_at: String,
    pub read_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub headers: Value,
    pub active: bool,
    pub timeout_secs: i64,
    pub max_attempts: i64,
}

#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub template_id: String,
    #[serde(default)]
    pub user_ids: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub data: Value,
    pub document_id: Option<String>,
}

fn render(input: &str, data: &Value) -> String {
    let mut out = input.to_string();
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let rendered = value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            out = out.replace(&format!("{{{{{key}}}}}"), &rendered);
        }
    }
    out
}

fn validate_template(channel: &str, subject: &str, body: &str) -> Result<()> {
    anyhow::ensure!(
        matches!(channel, "in_app" | "email"),
        "invalid notification channel"
    );
    anyhow::ensure!(body.len() <= 1_000_000, "notification body is too large");
    anyhow::ensure!(subject.len() <= 10_000, "notification subject is too large");
    Ok(())
}

pub async fn create_template(
    pool: &SqlitePool,
    name: &str,
    channel: &str,
    subject: &str,
    body: &str,
    variables: &Value,
    active: bool,
) -> Result<NotificationTemplate> {
    validate_template(channel, subject, body)?;
    let id = format!(
        "notification_template_{}",
        crate::repository::chrono_nanos_public()
    );
    sqlx::query("INSERT INTO _notification_template (id,name,channel,subject,body,variables,active) VALUES (?,?,?,?,?,?,?)")
        .bind(&id).bind(name.trim()).bind(channel).bind(subject).bind(body).bind(variables.to_string()).bind(i64::from(active)).execute(pool).await?;
    get_template(pool, &id).await
}

pub async fn get_template(pool: &SqlitePool, id: &str) -> Result<NotificationTemplate> {
    let r = sqlx::query_as::<_, (String,String,String,String,String,String,i64)>("SELECT id,name,channel,subject,body,variables,active FROM _notification_template WHERE id=?")
        .bind(id).fetch_one(pool).await?;
    Ok(NotificationTemplate {
        id: r.0,
        name: r.1,
        channel: r.2,
        subject: r.3,
        body: r.4,
        variables: serde_json::from_str(&r.5)?,
        active: r.6 != 0,
    })
}

pub async fn list_templates(pool: &SqlitePool) -> Result<Vec<NotificationTemplate>> {
    let ids =
        sqlx::query_scalar::<_, String>("SELECT id FROM _notification_template ORDER BY name")
            .fetch_all(pool)
            .await?;
    let mut out = Vec::new();
    for id in ids {
        out.push(get_template(pool, &id).await?);
    }
    Ok(out)
}

pub async fn update_template(
    pool: &SqlitePool,
    id: &str,
    channel: Option<&str>,
    subject: Option<&str>,
    body: Option<&str>,
    variables: Option<&Value>,
    active: Option<bool>,
) -> Result<NotificationTemplate> {
    let old = get_template(pool, id).await?;
    let c = channel.unwrap_or(&old.channel);
    let s = subject.unwrap_or(&old.subject);
    let b = body.unwrap_or(&old.body);
    let v = variables.unwrap_or(&old.variables);
    let a = active.unwrap_or(old.active);
    validate_template(c, s, b)?;
    sqlx::query("UPDATE _notification_template SET channel=?,subject=?,body=?,variables=?,active=?,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(c).bind(s).bind(b).bind(v.to_string()).bind(i64::from(a)).bind(id).execute(pool).await?;
    get_template(pool, id).await
}

pub async fn send(pool: &SqlitePool, req: &SendRequest) -> Result<usize> {
    let t = get_template(pool, &req.template_id).await?;
    anyhow::ensure!(t.active, "notification template is inactive");
    let mut users = req.user_ids.clone();
    if !req.roles.is_empty() {
        let marks = std::iter::repeat_n("?", req.roles.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql=format!("SELECT DISTINCT u.id FROM _user u LEFT JOIN _user_role ur ON ur.user_id=u.id LEFT JOIN _role r ON r.id=ur.role_id WHERE r.name IN ({marks})");
        let mut q = sqlx::query_scalar::<_, String>(&sql);
        for role in &req.roles {
            q = q.bind(role);
        }
        users.extend(q.fetch_all(pool).await?);
    }
    users.sort();
    users.dedup();
    anyhow::ensure!(!users.is_empty(), "notification has no recipients");
    let subject = render(&t.subject, &req.data);
    let body = render(&t.body, &req.data);
    let mut count = 0;
    for user in users {
        sqlx::query("INSERT INTO _notification (id,template_id,user_id,channel,subject,body,data,status) VALUES (?,?,?,?,?,?,?,'pending')").bind(format!("notification_{}",crate::repository::chrono_nanos_public())).bind(&t.id).bind(user).bind(&t.channel).bind(&subject).bind(&body).bind(req.data.to_string()).execute(pool).await?;
        count += 1;
    }
    Ok(count)
}

pub async fn list_for_user(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    unread: bool,
) -> Result<Vec<NotificationItem>> {
    let sql = if unread {
        "SELECT id,user_id,channel,subject,body,data,status,attempts,last_error,created_at,read_at FROM _notification WHERE user_id=? AND status!='read' ORDER BY created_at DESC LIMIT ?"
    } else {
        "SELECT id,user_id,channel,subject,body,data,status,attempts,last_error,created_at,read_at FROM _notification WHERE user_id=? ORDER BY created_at DESC LIMIT ?"
    };
    let rows = sqlx::query_as::<
        _,
        (
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
            Option<String>,
        ),
    >(sql)
    .bind(user_id)
    .bind(limit.clamp(1, 100))
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(NotificationItem {
                id: r.0,
                user_id: r.1,
                channel: r.2,
                subject: r.3,
                body: r.4,
                data: serde_json::from_str(&r.5)?,
                status: r.6,
                attempts: r.7,
                last_error: r.8,
                created_at: r.9,
                read_at: r.10,
            })
        })
        .collect()
}

pub async fn mark_read(pool: &SqlitePool, id: &str, user_id: &str) -> Result<()> {
    let r = sqlx::query(
        "UPDATE _notification SET status='read',read_at=CURRENT_TIMESTAMP WHERE id=? AND user_id=?",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    anyhow::ensure!(r.rows_affected() == 1, "notification not found");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_webhook(
    pool: &SqlitePool,
    name: &str,
    url: &str,
    secret: &str,
    headers: &Value,
    timeout: i64,
    max_attempts: i64,
    active: bool,
) -> Result<WebhookEndpoint> {
    anyhow::ensure!(
        url.starts_with("https://") || url.starts_with("http://"),
        "webhook url must use http or https"
    );
    anyhow::ensure!(timeout > 0 && timeout <= 300, "invalid webhook timeout");
    anyhow::ensure!(
        (1..=20).contains(&max_attempts),
        "invalid webhook max_attempts"
    );
    let id = format!("webhook_{}", crate::repository::chrono_nanos_public());
    sqlx::query("INSERT INTO _webhook_endpoint (id,name,url,secret,headers,active,timeout_secs,max_attempts) VALUES (?,?,?,?,?,?,?,?)").bind(&id).bind(name.trim()).bind(url).bind(secret).bind(headers.to_string()).bind(i64::from(active)).bind(timeout).bind(max_attempts).execute(pool).await?;
    get_webhook(pool, &id).await
}

pub async fn get_webhook(pool: &SqlitePool, id: &str) -> Result<WebhookEndpoint> {
    let r=sqlx::query_as::<_,(String,String,String,String,i64,i64,i64)>("SELECT id,name,url,headers,active,timeout_secs,max_attempts FROM _webhook_endpoint WHERE id=?").bind(id).fetch_one(pool).await?;
    Ok(WebhookEndpoint {
        id: r.0,
        name: r.1,
        url: r.2,
        headers: serde_json::from_str(&r.3)?,
        active: r.4 != 0,
        timeout_secs: r.5,
        max_attempts: r.6,
    })
}

pub async fn list_webhooks(pool: &SqlitePool) -> Result<Vec<WebhookEndpoint>> {
    let ids = sqlx::query_scalar::<_, String>("SELECT id FROM _webhook_endpoint ORDER BY name")
        .fetch_all(pool)
        .await?;
    let mut out = Vec::new();
    for id in ids {
        out.push(get_webhook(pool, &id).await?);
    }
    Ok(out)
}

pub async fn enqueue_webhook(
    pool: &SqlitePool,
    endpoint_id: &str,
    event_type: &str,
    document_id: Option<&str>,
    payload: &Value,
) -> Result<String> {
    let e = get_webhook(pool, endpoint_id).await?;
    anyhow::ensure!(e.active, "webhook endpoint is inactive");
    anyhow::ensure!(
        payload.to_string().len() <= 1_000_000,
        "webhook payload is too large"
    );
    let id = format!(
        "webhook_delivery_{}",
        crate::repository::chrono_nanos_public()
    );
    sqlx::query("INSERT INTO _webhook_delivery (id,endpoint_id,event_type,document_id,payload) VALUES (?,?,?,?,?)").bind(&id).bind(endpoint_id).bind(event_type).bind(document_id).bind(payload.to_string()).execute(pool).await?;
    Ok(id)
}

fn signature(secret: &str, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(payload.as_bytes());
    format!(
        "sha256={}",
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

pub async fn deliver_pending(pool: &SqlitePool) -> Result<usize> {
    let rows=sqlx::query_as::<_,(String,String,String,String,String,i64,i64,i64,String)>("SELECT d.id,e.url,e.secret,e.headers,d.payload,d.attempts,e.timeout_secs,e.max_attempts,d.event_type FROM _webhook_delivery d JOIN _webhook_endpoint e ON e.id=d.endpoint_id WHERE d.status='pending' ORDER BY d.created_at LIMIT 50").fetch_all(pool).await?;
    let mut sent = 0;
    for (id, url, secret, headers, payload, attempts, timeout, max_attempts, event_type) in rows {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout.max(1) as u64))
            .build()?;
        let mut req = client
            .post(&url)
            .header("content-type", "application/json")
            .header("x-logholizon-event", &event_type);
        if !secret.is_empty() {
            req = req.header("x-logholizon-signature", signature(&secret, &payload));
        }
        if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&headers)
        {
            for (k, v) in map {
                if let (Ok(n), Ok(v)) = (
                    reqwest::header::HeaderName::try_from(k),
                    reqwest::header::HeaderValue::try_from(v),
                ) {
                    req = req.header(n, v);
                }
            }
        }
        match req.body(payload).send().await {
            Ok(r) => {
                let code = r.status().as_u16() as i64;
                let ok = r.status().is_success();
                sqlx::query("UPDATE _webhook_delivery SET status=?,attempts=attempts+1,response_status=?,last_error=? ,delivered_at=CASE WHEN ? THEN CURRENT_TIMESTAMP ELSE delivered_at END WHERE id=?").bind(if ok{"delivered"}else{"pending"}).bind(code).bind(if ok{None::<String>}else{Some(format!("HTTP {code}"))}).bind(ok).bind(&id).execute(pool).await?;
                if ok {
                    sent += 1;
                } else if attempts + 1 >= max_attempts {
                    sqlx::query("UPDATE _webhook_delivery SET status='failed' WHERE id=?")
                        .bind(&id)
                        .execute(pool)
                        .await?;
                }
            }
            Err(e) => {
                let next = attempts + 1;
                sqlx::query(
                    "UPDATE _webhook_delivery SET attempts=?,last_error=?,status=? WHERE id=?",
                )
                .bind(next)
                .bind(e.to_string().chars().take(500).collect::<String>())
                .bind(if next >= max_attempts {
                    "failed"
                } else {
                    "pending"
                })
                .bind(&id)
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(sent)
}

pub async fn deliver_notifications(pool: &SqlitePool) -> Result<usize> {
    let rows=sqlx::query_as::<_,(String,String,String,String,String,i64,i64)>("SELECT n.id,u.username,n.subject,n.body,n.status,n.attempts,3 FROM _notification n JOIN _user u ON u.id=n.user_id WHERE n.channel='email' AND n.status='pending' ORDER BY n.created_at LIMIT 50").fetch_all(pool).await?;
    let mut sent = 0;
    for (id, to, subject, body, _, attempts, max_attempts) in rows {
        let url = std::env::var("CORE_EMAIL_WEBHOOK_URL").ok();
        if let Some(url) = url {
            let client = reqwest::Client::new();
            let result = client
                .post(url)
                .json(&json!({"to":to,"subject":subject,"body":body}))
                .send()
                .await;
            match result {
                Ok(r) if r.status().is_success() => {
                    sqlx::query("UPDATE _notification SET status='delivered',attempts=attempts+1,delivered_at=CURRENT_TIMESTAMP,last_error=NULL WHERE id=?").bind(&id).execute(pool).await?;
                    sent += 1;
                }
                _ => {
                    let next = attempts + 1;
                    sqlx::query(
                        "UPDATE _notification SET attempts=?,last_error=?,status=? WHERE id=?",
                    )
                    .bind(next)
                    .bind("email delivery failed")
                    .bind(if next >= max_attempts {
                        "failed"
                    } else {
                        "pending"
                    })
                    .bind(&id)
                    .execute(pool)
                    .await?;
                }
            }
        } else {
            break;
        }
    }
    Ok(sent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_variables() {
        let data = json!({"name":"Ada", "count":3});
        assert_eq!(render("Hello {{name}} ({{count}})", &data), "Hello Ada (3)");
    }

    #[test]
    fn validates_channels_and_limits() {
        assert!(validate_template("in_app", "", "ok").is_ok());
        assert!(validate_template("sms", "", "ok").is_err());
        assert!(validate_template("email", "", &"x".repeat(1_000_001)).is_err());
    }

    #[test]
    fn signs_payload_with_hmac() {
        let a = signature("secret", "{}");
        let b = signature("secret", "{}");
        assert_eq!(a, b);
        assert_ne!(a, signature("other", "{}"));
    }
}
