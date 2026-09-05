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
            database_url: env::var("CORE_DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://.data/core.db".to_string()),
        }
    }
}
