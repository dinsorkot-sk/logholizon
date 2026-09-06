use logholizon_core::{backup, http, Config};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    if backup::apply_staged_restore(&config.database_url).await? {
        tracing::info!("staged restore applied");
    }
    let pool = logholizon_core::db::connect(&config.database_url).await?;
    logholizon_core::db::migrate(&pool).await?;

    // Scheduled backups: VACUUM INTO every interval, keep the newest N.
    if config.backup_interval_hours > 0 {
        let task_pool = pool.clone();
        let task_url = config.database_url.clone();
        let interval = std::time::Duration::from_secs(config.backup_interval_hours * 3600);
        let keep = config.backup_keep;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match backup::scheduled_backup(&task_pool, &task_url, keep).await {
                    Ok(path) => tracing::info!("scheduled backup: {}", path.display()),
                    Err(error) => tracing::warn!("scheduled backup failed: {error:#}"),
                }
            }
        });
        tracing::info!(
            "scheduled backups every {}h, keep {}",
            config.backup_interval_hours,
            config.backup_keep
        );
    }

    let app = http::router(&config, pool).layer(CorsLayer::permissive());
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("logholizon-core listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
