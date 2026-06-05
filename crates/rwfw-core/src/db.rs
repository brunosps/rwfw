use crate::config::AppConfig;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use std::path::Path;
use std::time::Duration;

/// Connect using the configured database URL. The driver is chosen by URL
/// scheme: `postgres://`/`postgresql://` -> Postgres, `sqlite:`/a bare path ->
/// SQLite. Both drivers are compiled in; selection happens at runtime.
pub async fn connect(config: &AppConfig) -> anyhow::Result<DatabaseConnection> {
    let database = config.database()?;
    connect_url(&database.url).await
}

/// Connect to an explicit URL (used by tests, the CLI, and the Tauri host).
pub async fn connect_url(url: &str) -> anyhow::Result<DatabaseConnection> {
    let url = normalize_url(url);
    let mut opts = ConnectOptions::new(url.clone());
    // A single-writer pool for SQLite avoids SQLITE_BUSY storms and keeps the
    // per-connection PRAGMAs below effective for the only pooled connection.
    opts.max_connections(if is_sqlite(&url) { 1 } else { 16 })
        .acquire_timeout(Duration::from_secs(8))
        .sqlx_logging(false);

    let db = Database::connect(opts).await?;

    if db.get_database_backend() == DatabaseBackend::Sqlite {
        // WAL for concurrent reads; foreign_keys ON so ON DELETE CASCADE is
        // actually enforced (SQLite ignores FK actions unless this is set).
        for pragma in [
            "PRAGMA journal_mode = WAL",
            "PRAGMA foreign_keys = ON",
            "PRAGMA busy_timeout = 5000",
        ] {
            db.execute(Statement::from_string(
                DatabaseBackend::Sqlite,
                pragma.to_string(),
            ))
            .await?;
        }
    }

    Ok(db)
}

/// True if the (normalized) URL targets the SQLite driver.
pub fn is_sqlite(url: &str) -> bool {
    url.starts_with("sqlite:")
}

/// Accept a bare filesystem path (`./data/app.db`, `/abs/app.db`) as a SQLite
/// URL, and ensure SQLite is always allowed to create the file.
fn normalize_url(url: &str) -> String {
    if url.starts_with("postgres://")
        || url.starts_with("postgresql://")
        || url.starts_with("mysql://")
    {
        return url.to_string();
    }
    if url.starts_with("sqlite::memory:") {
        return url.to_string();
    }
    if url.starts_with("sqlite:") {
        // already a sqlite URL; make sure it can create the file.
        return ensure_sqlite_create(url);
    }
    // bare filesystem path -> sqlite URL
    sqlite_url_from_path(url)
}

/// Build a `sqlite://<path>?mode=rwc` URL from a filesystem path. Public so the
/// CLI/Tauri host can derive a DB URL from an app-data file. Absolute paths
/// yield `sqlite:///abs/...`.
pub fn sqlite_url_from_path(path: impl AsRef<Path>) -> String {
    let display = path.as_ref().to_string_lossy();
    ensure_sqlite_create(&format!("sqlite://{display}"))
}

fn ensure_sqlite_create(url: &str) -> String {
    if url.contains('?') {
        if url.contains("mode=") {
            url.to_string()
        } else {
            format!("{url}&mode=rwc")
        }
    } else {
        format!("{url}?mode=rwc")
    }
}
