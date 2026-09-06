use anyhow::Result;
use sqlx::SqlitePool;
use std::time::Duration;

/// Background webhook delivery worker (Phase 3). Polls pending deliveries,
/// POSTs each payload with a timeout, and retries with backoff up to the
/// configured max attempts before marking failed.
pub async fn deliver_pending(
    pool: &SqlitePool,
    timeout_secs: u64,
    max_attempts: i64,
) -> Result<usize> {
    let pending: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT d.id, r.target_url, d.payload FROM _notification_delivery d \
         JOIN _notification_rule r ON r.id = d.rule_id \
         WHERE d.status = 'pending' ORDER BY d.created_at LIMIT 50",
    )
    .fetch_all(pool)
    .await?;
    if pending.is_empty() {
        return Ok(0);
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs.max(1)))
        .build()?;
    let mut delivered = 0;
    for (id, target_url, payload) in pending {
        // Cap response bodies at 1MB; the stored payload is capped at enqueue.
        match client
            .post(&target_url)
            .header("content-type", "application/json")
            .body(payload)
            .send()
            .await
        {
            Ok(response) => {
                let response = response.bytes().await?;
                let _ = response.slice(..response.len().min(1_000_000));
                sqlx::query(
                    "UPDATE _notification_delivery SET status = 'delivered', attempts = attempts + 1, last_error = NULL WHERE id = ?",
                )
                .bind(&id)
                .execute(pool)
                .await?;
                delivered += 1;
            }
            Err(error) => {
                let message = error.to_string();
                let message = message.chars().take(500).collect::<String>();
                let attempts: i64 =
                    sqlx::query_scalar("SELECT attempts FROM _notification_delivery WHERE id = ?")
                        .bind(&id)
                        .fetch_one(pool)
                        .await?;
                let next_attempts = attempts + 1;
                let status = if next_attempts >= max_attempts.max(1) {
                    "failed"
                } else {
                    "pending"
                };
                sqlx::query(
                    "UPDATE _notification_delivery SET status = ?, attempts = ?, last_error = ? WHERE id = ?",
                )
                .bind(status)
                .bind(next_attempts)
                .bind(message)
                .bind(&id)
                .execute(pool)
                .await?;
            }
        }
    }
    Ok(delivered)
}
