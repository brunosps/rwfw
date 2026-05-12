use crate::config::LoggingConfig;
use tracing_subscriber::{EnvFilter, fmt};

pub fn init(config: &LoggingConfig) {
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format.as_str() {
        "json" => {
            fmt()
                .json()
                .with_env_filter(filter)
                .with_target(true)
                .with_thread_ids(true)
                .init();
        }
        _ => {
            fmt()
                .pretty()
                .with_env_filter(filter)
                .with_target(true)
                .init();
        }
    }
}

/// Initialize logging with sensible defaults (for use before config is loaded)
pub fn init_default() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .pretty()
        .with_env_filter(filter)
        .with_target(true)
        .init();
}
