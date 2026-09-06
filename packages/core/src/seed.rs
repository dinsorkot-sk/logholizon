use anyhow::Result;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{auth, repository};

pub async fn seed(pool: &SqlitePool) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT OR IGNORE INTO _meta_entity (id, name, label) VALUES (?, ?, ?)")
        .bind("work_order")
        .bind("work_order")
        .bind("Work Order")
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO _meta_entity (id, name, label) VALUES (?, ?, ?)")
        .bind("pm_schedule")
        .bind("pm_schedule")
        .bind("PM Schedule")
        .execute(&mut *tx)
        .await?;
    for entity_id in ["work_order", "pm_schedule"] {
        for role in ["admin", "user"] {
            sqlx::query(
                "INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
            )
            .bind(entity_id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
        }
    }
    for (id, entity_id, name, field_type, required, is_status) in [
        ("work_order_title", "work_order", "title", "text", 1, 0),
        ("work_order_status", "work_order", "status", "select", 1, 1),
        (
            "work_order_priority",
            "work_order",
            "priority",
            "select",
            0,
            0,
        ),
        (
            "work_order_assignee",
            "work_order",
            "assignee",
            "text",
            0,
            0,
        ),
        (
            "pm_schedule_due_date",
            "pm_schedule",
            "due_date",
            "date",
            1,
            0,
        ),
        ("pm_schedule_title", "pm_schedule", "title", "text", 1, 0),
        (
            "pm_schedule_status",
            "pm_schedule",
            "status",
            "select",
            1,
            1,
        ),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _meta_field (id, entity_id, name, type, required, is_status) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(id)
            .bind(entity_id)
            .bind(name)
            .bind(field_type)
            .bind(required)
            .bind(is_status)
            .execute(&mut *tx)
            .await?;
    }
    for (id, field_id, value, label) in [
        (
            "work_order_status_draft",
            "work_order_status",
            "draft",
            "Draft",
        ),
        (
            "work_order_status_open",
            "work_order_status",
            "open",
            "Open",
        ),
        (
            "work_order_status_done",
            "work_order_status",
            "done",
            "Done",
        ),
        (
            "work_order_priority_low",
            "work_order_priority",
            "low",
            "Low",
        ),
        (
            "work_order_priority_high",
            "work_order_priority",
            "high",
            "High",
        ),
        (
            "work_order_priority_critical",
            "work_order_priority",
            "critical",
            "Critical",
        ),
        (
            "pm_schedule_status_draft",
            "pm_schedule_status",
            "draft",
            "Draft",
        ),
        (
            "pm_schedule_status_scheduled",
            "pm_schedule_status",
            "scheduled",
            "Scheduled",
        ),
        (
            "pm_schedule_status_done",
            "pm_schedule_status",
            "done",
            "Done",
        ),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _meta_field_option (id, field_id, value, label) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(field_id)
            .bind(value)
            .bind(label)
            .execute(&mut *tx)
            .await?;
    }
    for (id, name, label, position) in [
        ("work_order_draft", "draft", "Draft", 0),
        ("work_order_open", "open", "Open", 1),
        ("work_order_done", "done", "Done", 2),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, 'work_order', ?, ?, ?)")
            .bind(id).bind(name).bind(label).bind(position).execute(&mut *tx).await?;
    }
    for (id, from_state, to_state, action) in [
        ("work_order_submit", "draft", "open", "submit"),
        ("work_order_done_transition", "open", "done", "done"),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, 'work_order', ?, ?, ?)")
            .bind(id).bind(from_state).bind(to_state).bind(action).execute(&mut *tx).await?;
    }
    for (id, name, label, position) in [
        ("pm_schedule_draft", "draft", "Draft", 0),
        ("pm_schedule_scheduled", "scheduled", "Scheduled", 1),
        ("pm_schedule_done", "done", "Done", 2),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, 'pm_schedule', ?, ?, ?)")
            .bind(id).bind(name).bind(label).bind(position).execute(&mut *tx).await?;
    }
    for (id, from_state, to_state, action) in [
        ("pm_schedule_schedule", "draft", "scheduled", "schedule"),
        ("pm_schedule_complete", "scheduled", "done", "complete"),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, 'pm_schedule', ?, ?, ?)")
            .bind(id).bind(from_state).bind(to_state).bind(action).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Demo data for trying out the product: users + sample documents.
/// Idempotent: existing users/documents are left untouched.
pub async fn seed_demo(pool: &SqlitePool) -> Result<()> {
    seed(pool).await?;

    for (username, password, role) in [("admin", "admin123", "admin"), ("demo", "demo1234", "user")]
    {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _user WHERE username = ?)")
                .bind(username)
                .fetch_one(pool)
                .await?;
        if !exists {
            auth::create_user(pool, username, password, role).await?;
        }
    }

    let work_orders: &[(&str, &str, &str, &str)] = &[
        ("demo-wo-1", "Fix water pump", "open", "high"),
        ("demo-wo-2", "Replace conveyor belt", "draft", "critical"),
        ("demo-wo-3", "Inspect fire alarm", "done", "low"),
        ("demo-wo-4", "Repair loading dock door", "open", "high"),
        ("demo-wo-5", "Calibrate scale", "draft", "low"),
        ("demo-wo-6", "Service forklift", "open", "critical"),
        ("demo-wo-7", "Clean ventilation ducts", "done", "low"),
        ("demo-wo-8", "Replace office lights", "draft", "low"),
        ("demo-wo-9", "Fix leaking pipe", "open", "high"),
        ("demo-wo-10", "Test backup generator", "done", "critical"),
    ];
    for (id, title, status, priority) in work_orders {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;
        if !exists {
            repository::create_document(
                pool,
                id,
                "work_order",
                &json!({"title": title, "status": status, "priority": priority}),
                None,
            )
            .await?;
        }
    }

    let pm_schedules: &[(&str, &str, &str, &str)] = &[
        (
            "demo-pm-1",
            "Monthly pump inspection",
            "2026-09-10",
            "scheduled",
        ),
        ("demo-pm-2", "Quarterly fire drill", "2026-09-15", "draft"),
        (
            "demo-pm-3",
            "Annual generator service",
            "2026-08-01",
            "done",
        ),
        (
            "demo-pm-4",
            "Weekly forklift check",
            "2026-09-08",
            "scheduled",
        ),
        (
            "demo-pm-5",
            "Monthly filter replacement",
            "2026-09-20",
            "draft",
        ),
    ];
    for (id, title, due_date, status) in pm_schedules {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _doc WHERE id = ?)")
            .bind(id)
            .fetch_one(pool)
            .await?;
        if !exists {
            repository::create_document(
                pool,
                id,
                "pm_schedule",
                &json!({"title": title, "due_date": due_date, "status": status}),
                None,
            )
            .await?;
        }
    }

    Ok(())
}
