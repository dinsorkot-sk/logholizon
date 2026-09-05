use logholizon_core::{backup, db, seed};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn seed_is_idempotent() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _meta_entity")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn backup_creates_valid_snapshot() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    let path = std::env::temp_dir().join(format!(
        "logholizon-test-{}.db",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    backup::backup(&pool, &path).await.unwrap();
    let url = format!("sqlite://{}", path.to_str().unwrap().replace('\\', "/"));
    let snapshot = db::connect(&url).await.unwrap();
    assert!(db::integrity_check(&snapshot).await.unwrap());
    snapshot.close().await;
    // ponytail: retry remove when Windows AV/indexer holds the temp file briefly.
    for _ in 0..10 {
        if tokio::fs::remove_file(&path).await.is_ok() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    tokio::fs::remove_file(&path).await.unwrap();
}
