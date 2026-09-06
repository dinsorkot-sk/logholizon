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
    seed_inventory_module(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

/// Inventory sample module (proves the module builder end-to-end):
/// `product` + `warehouse` entities linked from `stock_move` via reference
/// fields, grouped under the "Stock" module in the sidebar.
async fn seed_inventory_module(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
    for (id, name, label, module) in [
        ("product", "product", "Product", "Stock"),
        ("warehouse", "warehouse", "Warehouse", "Stock"),
        ("stock_move", "stock_move", "Stock Move", "Stock"),
    ] {
        sqlx::query(
            "INSERT INTO _meta_entity (id, name, label, module) VALUES (?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET module = excluded.module",
        )
        .bind(id)
        .bind(name)
        .bind(label)
        .bind(module)
        .execute(&mut **tx)
        .await?;
        for role in ["admin", "user"] {
            sqlx::query(
                "INSERT OR IGNORE INTO _entity_permission (entity_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
            )
            .bind(id)
            .bind(role)
            .execute(&mut **tx)
            .await?;
        }
    }
    // (field_id, entity_id, name, type, required, is_status, ref_entity, computed_expr)
    for (id, entity_id, name, field_type, required, is_status, ref_entity, computed_expr) in [
        (
            "product_title",
            "product",
            "title",
            "text",
            1,
            0,
            None,
            None,
        ),
        ("product_sku", "product", "sku", "text", 1, 0, None, None),
        (
            "product_unit",
            "product",
            "unit",
            "select",
            0,
            0,
            None,
            None,
        ),
        (
            "product_reorder_level",
            "product",
            "reorder_level",
            "number",
            0,
            0,
            None,
            None,
        ),
        (
            "product_status",
            "product",
            "status",
            "select",
            1,
            1,
            None,
            None,
        ),
        (
            "product_summary",
            "product",
            "summary",
            "computed",
            0,
            0,
            None,
            Some("{title} [{sku}]"),
        ),
        (
            "warehouse_title",
            "warehouse",
            "title",
            "text",
            1,
            0,
            None,
            None,
        ),
        (
            "warehouse_location",
            "warehouse",
            "location",
            "text",
            0,
            0,
            None,
            None,
        ),
        (
            "warehouse_status",
            "warehouse",
            "status",
            "select",
            1,
            1,
            None,
            None,
        ),
        (
            "stock_move_product",
            "stock_move",
            "product",
            "reference",
            1,
            0,
            Some("product"),
            None,
        ),
        (
            "stock_move_warehouse",
            "stock_move",
            "warehouse",
            "reference",
            1,
            0,
            Some("warehouse"),
            None,
        ),
        (
            "stock_move_qty",
            "stock_move",
            "qty",
            "number",
            1,
            0,
            None,
            None,
        ),
        (
            "stock_move_move_type",
            "stock_move",
            "move_type",
            "select",
            1,
            0,
            None,
            None,
        ),
        (
            "stock_move_status",
            "stock_move",
            "status",
            "select",
            1,
            1,
            None,
            None,
        ),
    ] {
        let id: &str = id;
        let entity_id: &str = entity_id;
        let name: &str = name;
        let field_type: &str = field_type;
        let ref_entity: Option<&str> = ref_entity;
        let computed_expr: Option<&str> = computed_expr;
        sqlx::query(
            "INSERT INTO _meta_field (id, entity_id, name, type, required, is_status, ref_entity, computed_expr) VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET ref_entity = excluded.ref_entity, computed_expr = excluded.computed_expr",
        )
        .bind(id)
        .bind(entity_id)
        .bind(name)
        .bind(field_type)
        .bind(required)
        .bind(is_status)
        .bind(ref_entity)
        .bind(computed_expr)
        .execute(&mut **tx)
        .await?;
        for role in ["admin", "user"] {
            sqlx::query(
                "INSERT OR IGNORE INTO _field_permission (field_id, role, can_view, can_edit) VALUES (?, ?, 1, 1)",
            )
            .bind(id)
            .bind(role)
            .execute(&mut **tx)
            .await?;
        }
    }
    for (id, field_id, value, label) in [
        ("product_unit_pcs", "product_unit", "pcs", "Pieces"),
        ("product_unit_box", "product_unit", "box", "Box"),
        ("product_unit_kg", "product_unit", "kg", "Kilograms"),
        (
            "product_status_active",
            "product_status",
            "active",
            "Active",
        ),
        (
            "product_status_discontinued",
            "product_status",
            "discontinued",
            "Discontinued",
        ),
        (
            "warehouse_status_active",
            "warehouse_status",
            "active",
            "Active",
        ),
        (
            "warehouse_status_closed",
            "warehouse_status",
            "closed",
            "Closed",
        ),
        (
            "stock_move_move_type_in",
            "stock_move_move_type",
            "in",
            "Stock In",
        ),
        (
            "stock_move_move_type_out",
            "stock_move_move_type",
            "out",
            "Stock Out",
        ),
        (
            "stock_move_status_draft",
            "stock_move_status",
            "draft",
            "Draft",
        ),
        (
            "stock_move_status_confirmed",
            "stock_move_status",
            "confirmed",
            "Confirmed",
        ),
        (
            "stock_move_status_done",
            "stock_move_status",
            "done",
            "Done",
        ),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _meta_field_option (id, field_id, value, label) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(field_id)
            .bind(value)
            .bind(label)
            .execute(&mut **tx)
            .await?;
    }
    for (id, entity_id, name, label, position) in [
        ("product_active", "product", "active", "Active", 0),
        (
            "product_discontinued",
            "product",
            "discontinued",
            "Discontinued",
            1,
        ),
        ("warehouse_active", "warehouse", "active", "Active", 0),
        ("warehouse_closed", "warehouse", "closed", "Closed", 1),
        ("stock_move_draft", "stock_move", "draft", "Draft", 0),
        (
            "stock_move_confirmed",
            "stock_move",
            "confirmed",
            "Confirmed",
            1,
        ),
        ("stock_move_done", "stock_move", "done", "Done", 2),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_state (id, entity_id, name, label, position) VALUES (?, ?, ?, ?, ?)")
            .bind(id).bind(entity_id).bind(name).bind(label).bind(position).execute(&mut **tx).await?;
    }
    for (id, entity_id, from_state, to_state, action) in [
        (
            "stock_move_confirm",
            "stock_move",
            "draft",
            "confirmed",
            "confirm",
        ),
        (
            "stock_move_receive",
            "stock_move",
            "confirmed",
            "done",
            "done",
        ),
    ] {
        sqlx::query("INSERT OR IGNORE INTO _workflow_transition (id, entity_id, from_state, to_state, action) VALUES (?, ?, ?, ?, ?)")
            .bind(id).bind(entity_id).bind(from_state).bind(to_state).bind(action).execute(&mut **tx).await?;
    }
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
