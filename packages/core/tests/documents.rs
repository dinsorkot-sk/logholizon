use logholizon_core::{db, repository};
use serde_json::json;

async fn seeded_pool() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO _meta_entity (id, name, label) VALUES ('ticket', 'ticket', 'Ticket')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO _meta_field (id, entity_id, name, type, required) VALUES ('f1', 'ticket', 'title', 'text', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO _meta_field (id, entity_id, name, type, required) VALUES ('f2', 'ticket', 'priority', 'number', 0)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn document_crud_validates_payload() {
    let pool = seeded_pool().await;
    let doc = repository::create_document(&pool, "d1", "ticket", &json!({"title": "Fix pump"}))
        .await
        .unwrap();
    assert_eq!(doc.payload["title"], "Fix pump");

    let err = repository::create_document(&pool, "d2", "ticket", &json!({"priority": 1}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("missing required field"));

    let err = repository::create_document(&pool, "d3", "ticket", &json!({"title": 1}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid value"));

    let err = repository::create_document(&pool, "d4", "ticket", &json!({"title": "x", "nope": 1}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown field"));

    let updated = repository::update_document(&pool, "d1", &json!({"title": "Fixed"}))
        .await
        .unwrap();
    assert_eq!(updated.payload["title"], "Fixed");

    let list = repository::list_documents(&pool, "ticket", 10, 0)
        .await
        .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items.len(), 1);

    repository::delete_document(&pool, "d1").await.unwrap();
    assert!(repository::get_document(&pool, "d1").await.is_err());
}
