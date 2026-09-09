use anyhow::{bail, Result};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: String,
    pub system: bool,
    pub created_at: String,
}

pub async fn list_roles(pool: &SqlitePool) -> Result<Vec<Role>> {
    let rows = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        "SELECT id,name,label,description,system,created_at FROM _role ORDER BY system DESC,label,name",
    ).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, label, description, system, created_at)| Role {
            id,
            name,
            label,
            description,
            system: system != 0,
            created_at,
        })
        .collect())
}

pub async fn create_role(
    pool: &SqlitePool,
    name: &str,
    label: &str,
    description: &str,
) -> Result<Role> {
    let name = name.trim();
    let label = label.trim();
    if name.is_empty() || label.is_empty() {
        bail!("role name and label are required");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        bail!("role name must contain lowercase letters, digits, '_' or '-'");
    }
    let id = format!("role_{name}");
    sqlx::query("INSERT INTO _role (id,name,label,description,system) VALUES (?,?,?,?,0)")
        .bind(&id)
        .bind(name)
        .bind(label)
        .bind(description.trim())
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO _entity_permission (entity_id, role) SELECT id, ? FROM _meta_entity",
    )
    .bind(name)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT OR IGNORE INTO _field_permission (field_id, role) SELECT id, ? FROM _meta_field",
    )
    .bind(name)
    .execute(pool)
    .await?;
    sqlx::query("INSERT OR IGNORE INTO _record_permission (entity_id, role, scope) SELECT id, ?, ''all'' FROM _meta_entity")
        .bind(name).execute(pool).await?;
    get_role(pool, &id).await
}

pub async fn get_role(pool: &SqlitePool, id: &str) -> Result<Role> {
    let row = sqlx::query_as::<_, (String, String, String, String, i64, String)>(
        "SELECT id,name,label,description,system,created_at FROM _role WHERE id=?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let Some((id, name, label, description, system, created_at)) = row else {
        return Err(AppError::NotFound(format!("role not found: {id}")).into());
    };
    Ok(Role {
        id,
        name,
        label,
        description,
        system: system != 0,
        created_at,
    })
}

pub async fn update_role(
    pool: &SqlitePool,
    id: &str,
    label: &str,
    description: &str,
) -> Result<Role> {
    let role = get_role(pool, id).await?;
    if role.system {
        bail!("system role cannot be modified");
    }
    sqlx::query("UPDATE _role SET label=?, description=? WHERE id=?")
        .bind(label.trim())
        .bind(description.trim())
        .bind(id)
        .execute(pool)
        .await?;
    get_role(pool, id).await
}

pub async fn delete_role(pool: &SqlitePool, id: &str) -> Result<()> {
    let role = get_role(pool, id).await?;
    if role.system {
        bail!("system role cannot be deleted");
    }
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _user_role WHERE role_id=?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    if users > 0 {
        return Err(AppError::Conflict("role is assigned to users".into()).into());
    }
    sqlx::query("DELETE FROM _role WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn assign_role(pool: &SqlitePool, user_id: &str, role_id: &str) -> Result<()> {
    get_role(pool, role_id).await?;
    let result = sqlx::query("INSERT INTO _user_role(user_id,role_id) VALUES(?,?) ON CONFLICT(user_id) DO UPDATE SET role_id=excluded.role_id")
        .bind(user_id).bind(role_id).execute(pool).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("user not found: {user_id}")).into());
    }
    Ok(())
}

pub async fn effective_role_name(pool: &SqlitePool, user_id: &str) -> Result<String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT r.name FROM _user_role ur JOIN _role r ON r.id=ur.role_id WHERE ur.user_id=?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    if let Some((name,)) = row {
        return Ok(name);
    }
    let row = sqlx::query_as::<_, (String,)>("SELECT role FROM _user WHERE id=?")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::NotFound(format!("user not found: {user_id}")).into())
}
