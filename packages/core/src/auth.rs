use anyhow::{bail, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct Session {
    pub token: String,
    pub user: User,
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| anyhow::anyhow!("password hashing failed: {error}"))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn new_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub async fn register(pool: &SqlitePool, username: &str, password: &str) -> Result<User> {
    let username = username.trim();
    if username.is_empty() || password.len() < 8 {
        bail!("username is required and password must be at least 8 characters");
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _user")
        .fetch_one(pool)
        .await?;
    let role = if count == 0 { "admin" } else { "user" };
    let id = format!("user_{username}");
    let hash = hash_password(password)?;
    sqlx::query("INSERT INTO _user (id, username, password_hash, role) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(username)
        .bind(&hash)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(User {
        id,
        username: username.to_string(),
        role: role.to_string(),
    })
}

pub async fn login(pool: &SqlitePool, username: &str, password: &str) -> Result<Session> {
    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, username, password_hash, role FROM _user WHERE username = ?",
    )
    .bind(username.trim())
    .fetch_optional(pool)
    .await?;
    let Some((id, username, hash, role)) = row else {
        return Err(AppError::Unauthorized("invalid username or password".into()).into());
    };
    if !verify_password(password, &hash) {
        return Err(AppError::Unauthorized("invalid username or password".into()).into());
    }
    let token = new_token();
    let expires = chrono_like_now_plus_days(7);
    sqlx::query("INSERT INTO _session (token, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(&id)
        .bind(&expires)
        .execute(pool)
        .await?;
    Ok(Session {
        token,
        user: User { id, username, role },
    })
}

pub async fn logout(pool: &SqlitePool, token: &str) -> Result<()> {
    sqlx::query("DELETE FROM _session WHERE token = ?")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn user_for_token(pool: &SqlitePool, token: &str) -> Result<User> {
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT u.id, u.username, u.role FROM _session s JOIN _user u ON u.id = s.user_id WHERE s.token = ? AND s.expires_at > datetime('now')",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;
    let Some((id, username, role)) = row else {
        return Err(AppError::Unauthorized("invalid or expired session".into()).into());
    };
    Ok(User { id, username, role })
}

/// UTC timestamp `YYYY-MM-DD HH:MM:SS` (SQLite format) `days` from now.
fn chrono_like_now_plus_days(days: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let target = now + days * 86_400;
    let days_since_epoch = target / 86_400;
    let secs_of_day = target % 86_400;
    let (year, month, day) = civil_from_days(days_since_epoch);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

/// Howard Hinnant's civil-from-days algorithm (days since 1970-01-01).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}
