use crate::config::AppConfig;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
};
use std::path::Path;
use std::time::Duration;

/// Connect using the configured database URL. The driver is chosen by URL
/// scheme: `postgres://`/`postgresql://` -> Postgres, `sqlite:`/a bare path ->
/// SQLite. Each driver is compiled in only with its feature (`postgres`,
/// `sqlite`; both by default); selection happens at runtime.
pub async fn connect(config: &AppConfig) -> anyhow::Result<DatabaseConnection> {
    let database = config.database()?;
    connect_url(&database.url).await
}

/// Connect to an explicit URL (used by tests, the CLI, and the Tauri host).
pub async fn connect_url(url: &str) -> anyhow::Result<DatabaseConnection> {
    let url = normalize_url(url);
    ensure_driver_compiled(&url)?;
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

/// Fail with a clear message when the URL needs a driver this build left out,
/// instead of sea-orm's generic "no supporting driver" error.
fn ensure_driver_compiled(url: &str) -> anyhow::Result<()> {
    if is_sqlite(url) && !cfg!(feature = "sqlite") {
        anyhow::bail!(
            "database URL is SQLite, but rwfw-core was built without the `sqlite` feature"
        );
    }
    if (url.starts_with("postgres://") || url.starts_with("postgresql://"))
        && !cfg!(feature = "postgres")
    {
        anyhow::bail!(
            "database URL is Postgres, but rwfw-core was built without the `postgres` feature"
        );
    }
    Ok(())
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

#[cfg(test)]
mod driver_feature_tests {
    #[cfg(not(feature = "sqlite"))]
    #[tokio::test]
    async fn sqlite_url_without_the_sqlite_feature_fails_clearly() {
        let error = super::connect_url("sqlite::memory:").await.unwrap_err();
        assert!(
            error.to_string().contains("without the `sqlite` feature"),
            "{error}"
        );
    }

    #[cfg(not(feature = "postgres"))]
    #[tokio::test]
    async fn postgres_url_without_the_postgres_feature_fails_clearly() {
        let error = super::connect_url("postgres://u:p@127.0.0.1:1/db")
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("without the `postgres` feature"),
            "{error}"
        );
    }
}
