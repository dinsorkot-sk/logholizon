use logholizon_core::db;

#[tokio::test]
async fn migrate_and_check_in_memory() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    assert!(db::integrity_check(&pool).await.unwrap());
    let tables: Vec<String> =
        sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert!(tables.contains(&"_meta_entity".to_string()));
    assert!(tables.contains(&"_meta_field".to_string()));
    assert!(tables.contains(&"_meta_field_option".to_string()));
}
