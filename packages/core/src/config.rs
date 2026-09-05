use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("CORE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("CORE_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8787),
            database_url: env::var("CORE_DATABASE_URL").unwrap_or_else(|_| default_database_url()),
        }
    }
}

/// Single shared dev DB at the workspace root (`<root>/.data/core.db`).
/// `cargo run -p logholizon-core` executes with cwd = `packages/core`,
/// so a bare `sqlite://.data/core.db` would land in `packages/core/.data`.
/// Walk up from the current exe until a `Cargo.toml` workspace root is found.
fn default_database_url() -> String {
    let mut dir = std::env::current_dir().ok();
    while let Some(candidate) = dir.clone() {
        if candidate.join("Cargo.toml").is_file() && candidate.join("pnpm-workspace.yaml").is_file()
        {
            let path = candidate.join(".data").join("core.db");
            return format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
        }
        dir = candidate.parent().map(std::path::Path::to_path_buf);
    }
    "sqlite://.data/core.db".to_string()
}
