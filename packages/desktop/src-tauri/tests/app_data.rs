//! App-data lifecycle test: directory creation, writability validation,
//! and first-run boot behavior.

use logholizon_desktop::{app_data_dir, boot, ensure_app_data_dir, shutdown};

#[tokio::test]
async fn app_data_dir_is_created_and_writable() {
    let data_dir = std::env::temp_dir().join(format!(
        "logholizon-test-appdata-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Directory does not exist yet.
    assert!(!data_dir.exists());

    // ensure_app_data_dir creates it and the logholizon/ subdirectory.
    ensure_app_data_dir(&data_dir).unwrap();
    assert!(data_dir.exists());
    assert!(data_dir.join("logholizon").is_dir());

    // Idempotent: calling again succeeds.
    ensure_app_data_dir(&data_dir).unwrap();

    // Boot works against the prepared directory.
    let runtime = boot(&data_dir).await.unwrap();
    assert!(runtime.addr.ip().is_loopback());
    shutdown(&runtime.pool).await.unwrap();
}

#[tokio::test]
async fn app_data_dir_rejects_file_path() {
    let data_dir = std::env::temp_dir().join(format!(
        "logholizon-test-appdata-file-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Create a file where the directory should be.
    std::fs::write(&data_dir, b"not a directory").unwrap();

    // ensure_app_data_dir must fail because create_dir_all on an existing
    // file path errors.
    assert!(ensure_app_data_dir(&data_dir).is_err());
}

#[test]
fn app_data_dir_resolves_to_expected_location() {
    // With XDG_DATA_HOME set, the path must include logholizon-desktop.
    let dir = app_data_dir();
    assert!(dir.to_string_lossy().contains("logholizon-desktop"));
}
