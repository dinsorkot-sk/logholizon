use anyhow::{bail, Result};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub label: String,
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
