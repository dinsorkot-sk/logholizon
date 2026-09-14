use logholizon_core::{auth, db, desktop};

#[tokio::test]
async fn desktop_boot_resolves_appdata_db_and_first_run() {
    let data_dir = std::env::temp_dir().join(format!(
        "logholizon-test-desktop-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let url = desktop::desktop_database_url(&data_dir).await.unwrap();
    assert!(url.starts_with("sqlite://"));
    assert!(url.contains("logholizon/core.db"));

    // Desktop config points at the app-data DB, allows Tauri origins,
    // and disables background loops by default.
    let config = desktop::desktop_config(&url);
    assert_eq!(config.database_url, url);
    for origin in desktop::DESKTOP_ALLOWED_ORIGINS {
        assert!(
            config.allowed_origins.iter().any(|o| o == origin),
            "missing desktop origin {origin}"
        );
    }
    assert_eq!(config.backup_interval_hours, 0);
    assert_eq!(config.notify_interval_secs, 0);

    // Boot pool runs the shared migrations; first run creates admin + demo.
    let pool = desktop::boot_desktop_pool(&url).await.unwrap();
    assert!(db::integrity_check(&pool).await.unwrap());
    assert!(desktop::ensure_first_run(&pool).await.unwrap());
    let session = auth::login(&pool, "admin", "admin12345").await.unwrap();
    assert_eq!(session.user.role, "admin");
    // Second call is a no-op (users already exist).
    assert!(!desktop::ensure_first_run(&pool).await.unwrap());
}
