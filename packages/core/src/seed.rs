use anyhow::Result;
use sqlx::SqlitePool;

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
