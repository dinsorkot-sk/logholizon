use logholizon_core::{backup, db, repository, seed};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "logholizon-test-{}-{name}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

async fn remove_retry(path: &std::path::Path) {
    for _ in 0..10 {
        if tokio::fs::remove_file(path).await.is_ok() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    tokio::fs::remove_file(path).await.unwrap();
}

#[tokio::test]
async fn seed_demo_is_idempotent() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed_demo(&pool).await.unwrap();
    seed::seed_demo(&pool).await.unwrap();

    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _user")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 2);

    let docs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc WHERE id LIKE 'demo-%'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(docs, 15);

    // Demo users can log in.
    let session = logholizon_core::auth::login(&pool, "demo", "demo1234")
        .await
        .unwrap();
    assert_eq!(session.user.role, "user");
    let admin = logholizon_core::auth::login(&pool, "admin", "admin123")
        .await
        .unwrap();
    assert_eq!(admin.user.role, "admin");
}

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
    assert_eq!(count, 5);
}

#[tokio::test]
async fn prune_backups_keeps_newest() {
    let dir = temp_path("prunedir");
    tokio::fs::create_dir_all(&dir).await.unwrap();
    for name in ["core-1.db", "core-2.db", "core-3.db", "other.txt"] {
        tokio::fs::write(dir.join(name), b"x").await.unwrap();
    }
    let removed = backup::prune_backups(&dir, 2).await.unwrap();
    assert_eq!(removed, 1);
    assert!(!dir.join("core-1.db").exists());
    assert!(dir.join("core-2.db").exists());
    assert!(dir.join("core-3.db").exists());
    // Non-backup files are untouched.
    assert!(dir.join("other.txt").exists());
    // Missing dir is a no-op.
    assert_eq!(
        backup::prune_backups(&dir.join("missing"), 2)
            .await
            .unwrap(),
        0
    );
    tokio::fs::remove_dir_all(&dir).await.ok();
}

#[tokio::test]
async fn backup_creates_valid_snapshot() {
    // File-backed source: VACUUM INTO from a shared-cache in-memory DB
    // silently produces no file.
    let src_dir = temp_path("snapdir");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();
    let src_path = src_dir.join("src.db");
    let src_url = format!("sqlite://{}", src_path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&src_url).await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    let path = src_dir.join("snapshot.db");
    backup::backup(&pool, &path).await.unwrap();
    pool.close().await;
    let url = format!("sqlite://{}", path.to_str().unwrap().replace('\\', "/"));
    let snapshot = db::connect(&url).await.unwrap();
    assert!(db::integrity_check(&snapshot).await.unwrap());
    snapshot.close().await;
    remove_retry(&path).await;
    remove_retry(&src_path).await;
    tokio::fs::remove_dir(&src_dir).await.ok();
}

#[tokio::test]
async fn validate_rejects_non_sqlite_file() {
    let path = temp_path("garbage.db");
    tokio::fs::write(&path, b"this is not a sqlite database")
        .await
        .unwrap();
    assert!(backup::validate(&path).await.is_err());
    remove_retry(&path).await;
}

#[tokio::test]
async fn staged_restore_applies_on_startup() {
    // Source DB with a document (file-backed: VACUUM INTO cannot back up
    // a shared-cache in-memory DB to a file).
    let src_dir = temp_path("srcdir");
    tokio::fs::create_dir_all(&src_dir).await.unwrap();
    let src_path = src_dir.join("src.db");
    let src_url = format!("sqlite://{}", src_path.to_str().unwrap().replace('\\', "/"));
    let pool = db::connect(&src_url).await.unwrap();
    db::migrate(&pool).await.unwrap();
    seed::seed(&pool).await.unwrap();
    repository::create_document(
        &pool,
        "wo-1",
        "work_order",
        &json!({"title": "Fix pump", "status": "draft", "priority": "high"}),
        None,
    )
    .await
    .unwrap();
    let backup_path = src_dir.join("backup.db");
    if let Err(error) = backup::backup(&pool, &backup_path).await {
        eprintln!("backup error: {error:#}");
        panic!("VACUUM INTO backup failed");
    }
    pool.close().await;

    // Destination DB (empty, different file) in its own temp dir so the
    // staging file (`restore-pending.db` next to the destination) is unique
    // per test run — parallel tests share the same system temp dir.
    let dest_dir = temp_path("destdir");
    tokio::fs::create_dir_all(&dest_dir).await.unwrap();
    let dest_path = dest_dir.join("core.db");
    let dest_url = format!(
        "sqlite://{}",
        dest_path.to_str().unwrap().replace('\\', "/")
    );
    let dest_pool = db::connect(&dest_url).await.unwrap();
    db::migrate(&dest_pool).await.unwrap();
    dest_pool.close().await;

    // Stage the backup next to the destination.
    let staging = dest_path.parent().unwrap().join("restore-pending.db");
    tokio::fs::copy(&backup_path, &staging).await.unwrap();

    // "Startup" applies the staged restore.
    assert!(backup::apply_staged_restore(&dest_url).await.unwrap());
    assert!(!staging.exists());

    let restored = db::connect(&dest_url).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc")
        .fetch_one(&restored)
        .await
        .unwrap();
    assert_eq!(count, 1);
    restored.close().await;

    remove_retry(&backup_path).await;
    remove_retry(&dest_path).await;
    tokio::fs::remove_dir(&dest_dir).await.ok();
    remove_retry(&src_path).await;
    tokio::fs::remove_dir(&src_dir).await.ok();
}
