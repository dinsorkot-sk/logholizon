use logholizon_core::{db, repository};

#[tokio::test]
async fn create_and_list_entities() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    let created = repository::create_entity(&pool, "asset", "asset", "Asset")
        .await
        .unwrap();
    assert_eq!(created.name, "asset");
    let list = repository::list_entities(&pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "asset");
}
