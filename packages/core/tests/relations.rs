use logholizon_core::{db, relation, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "vehicle", "vehicle", "Vehicle")
        .await
        .unwrap();
    repository::create_entity(&pool, "driver", "driver", "Driver")
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn relation_crud_and_validation() {
    let pool = setup().await;
    let driver_id = repository::create_field(
        &pool,
        "vehicle",
        "driver",
        "reference",
        false,
        false,
        Some("driver"),
        None,
    )
    .await
    .unwrap();
    let relation = relation::create_relation(
        &pool,
        "vehicle",
        Some(&driver_id.id),
        "driver",
        None,
        "driver",
        "many_to_one",
        "restrict",
    )
    .await
    .unwrap();
    assert_eq!(relation.relation_type, "many_to_one");
    assert_eq!(relation.on_delete, "restrict");
    let listed = relation::list_relations(&pool, "vehicle").await.unwrap();
    assert_eq!(listed.len(), 1);
    let updated = relation::update_relation(
        &pool,
        &relation.id,
        "assigned_driver",
        "one_to_one",
        "set_null",
        Some(&driver_id.id),
        None,
    )
    .await
    .unwrap();
    assert_eq!(updated.name, "assigned_driver");
    assert_eq!(updated.on_delete, "set_null");
    assert!(relation::create_relation(
        &pool,
        "vehicle",
        None,
        "driver",
        None,
        "bad",
        "one_to_many",
        "restrict"
    )
    .await
    .is_err());
    relation::delete_relation(&pool, &relation.id)
        .await
        .unwrap();
    assert!(relation::get_relation(&pool, &relation.id).await.is_err());
}

#[tokio::test]
async fn many_to_many_links_roundtrip() {
    let pool = setup().await;
    let relation = relation::create_relation(
        &pool,
        "vehicle",
        None,
        "driver",
        None,
        "drivers",
        "many_to_many",
        "restrict",
    )
    .await
    .unwrap();
    repository::create_document(&pool, "v1", "vehicle", &serde_json::json!({}), None)
        .await
        .unwrap();
    repository::create_document(&pool, "d1", "driver", &serde_json::json!({}), None)
        .await
        .unwrap();
    repository::create_document(&pool, "d2", "driver", &serde_json::json!({}), None)
        .await
        .unwrap();
    let links = relation::set_links(&pool, &relation.id, "v1", &["d1".into(), "d2".into()])
        .await
        .unwrap();
    assert_eq!(links.len(), 2);
    let related = relation::related_documents(&pool, &relation.id, "v1")
        .await
        .unwrap();
    assert_eq!(related.len(), 2);
}
