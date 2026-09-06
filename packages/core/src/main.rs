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
    let app = http::router(&config, pool).layer(CorsLayer::permissive());
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("logholizon-core listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
