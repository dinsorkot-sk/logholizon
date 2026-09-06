use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub backup_interval_hours: u64,
    pub backup_keep: usize,
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
            backup_interval_hours: env::var("CORE_BACKUP_INTERVAL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            backup_keep: env::var("CORE_BACKUP_KEEP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
        }
    }
}

/// Single shared dev DB at the workspace root (`<root>/.data/core.db`).
/// Turbo runs the core with cwd = `packages/core`, so a bare
/// `sqlite://.data/core.db` would land in `packages/core/.data`.
/// Walk up from the current exe location first (stable regardless of the
/// invoker cwd), then fall back to the current dir.
fn default_database_url() -> String {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    for start in candidates {
        let mut dir = Some(start);
        while let Some(candidate) = dir.clone() {
            if candidate.join("Cargo.toml").is_file()
                && candidate.join("pnpm-workspace.yaml").is_file()
            {
                let path = candidate.join(".data").join("core.db");
                return format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
            }
            dir = candidate.parent().map(std::path::Path::to_path_buf);
        }
    }
    "sqlite://.data/core.db".to_string()
}
