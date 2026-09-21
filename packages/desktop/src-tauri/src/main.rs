//! Desktop binary: boots the in-process core, then opens the Tauri window.
//!
//! Without the `ui` feature (workspace gates, CI without WebKit) this runs
//! headless: boot + health-gate + shutdown, which still proves the sidecar
//! lifecycle end to end.

use logholizon_desktop::{app_data_dir, boot, ensure_app_data_dir, shutdown, wait_for_health};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let data_dir = app_data_dir();
    ensure_app_data_dir(&data_dir).map_err(|e| {
        tracing::error!("app-data initialization failed: {:#}", e);
        e
    })?;

    let runtime = boot(&data_dir).await?;
    let base_url = runtime.base_url();
    wait_for_health(&base_url, std::time::Duration::from_secs(30)).await?;
    tracing::info!("logholizon-desktop ready at {base_url}");

    // Publish the sidecar URL where the frontend can read it: the Tauri
    // `ui` build forwards it via event, and every build (including plain
    // `cargo run` dev) writes it next to the database so the SPA can pick
    // it up without hardcoding the ephemeral port.
    if let Err(error) = write_core_url_file(&data_dir, &base_url) {
        tracing::warn!("could not write core-url file: {error:#}");
    }

    #[cfg(feature = "ui")]
    {
        run_window(&base_url).await?;
    }
    #[cfg(not(feature = "ui"))]
    {
        tracing::info!("ui feature disabled; headless boot OK at {base_url}");
    }

    shutdown(&runtime.pool).await?;
    Ok(())
}

#[cfg(feature = "ui")]
async fn run_window(base_url: &str) -> anyhow::Result<()> {
    use tauri::Emitter;

    // The window frontend is the Nuxt static SPA built into the bundle.
    // The base URL is exposed to the frontend via a Tauri event/asset so
    // direct `/v1` calls target the ephemeral sidecar port.
    tauri::Builder::default()
        .setup({
            let base_url = base_url.to_string();
            move |app| {
                app.emit("logholizon:core-url", &base_url)?;
                Ok(())
            }
        })
        .run(tauri::generate_context!())
        .map_err(|error| anyhow::anyhow!("tauri runtime failed: {error}"))?;
    Ok(())
}

/// Write the sidecar base URL next to the database so tooling and the
/// dev-mode SPA can discover the ephemeral port without hardcoding it.
fn write_core_url_file(data_dir: &std::path::Path, base_url: &str) -> anyhow::Result<()> {
    let path = data_dir.join("logholizon").join("core-url.txt");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, base_url)?;
    Ok(())
}
