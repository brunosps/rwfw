use crate::module::Module;
use crate::sql::escape_sql;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Migration ledger table. Table names are uniform across backends (no schemas)
/// so the same name works on Postgres and SQLite.
pub const MIGRATION_LEDGER_TABLE: &str = "rwfw_migrations";

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: &'static str,
    pub name: &'static str,
    /// Postgres (and default) SQL body.
    pub sql: &'static str,
    /// Optional SQLite-specific SQL body; falls back to `sql` when `None`.
    pub sqlite_sql: Option<&'static str>,
}

impl Migration {
    /// A migration whose single SQL body runs on every backend. Generated
    /// scaffolds still call this 3-arg form; it stays source-compatible.
    pub const fn new(version: &'static str, name: &'static str, sql: &'static str) -> Self {
        Self {
            version,
            name,
            sql,
            sqlite_sql: None,
        }
    }

    /// A migration carrying both a Postgres body and a SQLite body.
    pub const fn with_sqlite(
        version: &'static str,
        name: &'static str,
        sql: &'static str,
        sqlite_sql: &'static str,
    ) -> Self {
        Self {
            version,
            name,
            sql,
            sqlite_sql: Some(sqlite_sql),
        }
    }

    /// The SQL body to apply on the active backend.
    pub fn body(&self, backend: DatabaseBackend) -> &'static str {
        match backend {
            DatabaseBackend::Sqlite => self.sqlite_sql.unwrap_or(self.sql),
            _ => self.sql,
        }
    }

    /// Backend-independent checksum (hashes the Postgres body) so a ledger row
    /// stays stable regardless of which backend applied it.
    pub fn checksum(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.sql.trim_end().hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

#[derive(Debug, Default)]
pub struct MigrationReport {
    pub applied: Vec<MigrationRecord>,
    pub skipped: Vec<MigrationRecord>,
}

#[derive(Debug)]
pub struct MigrationRecord {
    pub module: String,
    pub version: String,
    pub name: String,
}

impl MigrationReport {
    fn extend(&mut self, other: MigrationReport) {
        self.applied.extend(other.applied);
        self.skipped.extend(other.skipped);
    }
}

pub async fn run_pending_migrations(
    db: &DatabaseConnection,
    modules: &[Box<dyn Module>],
) -> anyhow::Result<MigrationReport> {
    ensure_ledger(db).await?;

    let mut report = MigrationReport::default();
    for module in modules {
        report.extend(run_module_migrations(db, module.name(), module.migrations()).await?);
    }

    let declared_permissions = modules
        .iter()
        .map(|module| (module.name().to_string(), module.permissions()))
        .filter(|(_, permissions)| !permissions.is_empty())
        .collect::<Vec<_>>();

    if crate::auth::rbac_tables_available(db).await? {
        crate::auth::ensure_role(db, "admin", "Full framework administrator").await?;
        for (module_name, permissions) in declared_permissions {
            crate::auth::ensure_permissions(db, &module_name, &permissions).await?;
        }
    } else if !declared_permissions.is_empty() {
        tracing::warn!(
            "Skipping permission sync because auth RBAC tables are not available; include the auth module to enable RBAC"
        );
    }

    Ok(report)
}

pub async fn run_module_migrations(
    db: &DatabaseConnection,
    module: &str,
    migrations: Vec<Migration>,
) -> anyhow::Result<MigrationReport> {
    ensure_ledger(db).await?;
    let backend = db.get_database_backend();
    let mut report = MigrationReport::default();

    for migration in migrations {
        let checksum = migration.checksum();

        if let Some(applied_checksum) = applied_checksum(db, module, migration.version).await? {
            if applied_checksum != checksum {
                anyhow::bail!(
                    "Migration checksum mismatch for {}:{} ({})",
                    module,
                    migration.version,
                    migration.name
                );
            }

            report.skipped.push(MigrationRecord {
                module: module.to_string(),
                version: migration.version.to_string(),
                name: migration.name.to_string(),
            });
            continue;
        }

        tracing::info!(
            module = %module,
            version = %migration.version,
            name = %migration.name,
            "Applying migration"
        );

        db.execute_unprepared(migration.body(backend)).await?;
        record_applied(db, module, &migration, &checksum).await?;

        report.applied.push(MigrationRecord {
            module: module.to_string(),
            version: migration.version.to_string(),
            name: migration.name.to_string(),
        });
    }

    Ok(report)
}

async fn ensure_ledger(db: &DatabaseConnection) -> anyhow::Result<()> {
    // Uniform, portable DDL: no schema, `applied_at` is TEXT filled from Rust at
    // insert time (see `record_applied`), so the same statement runs on both
    // Postgres and SQLite.
    db.execute_unprepared(&format!(
        "CREATE TABLE IF NOT EXISTS {MIGRATION_LEDGER_TABLE} (
            module_name TEXT NOT NULL,
            version TEXT NOT NULL,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL,
            PRIMARY KEY (module_name, version)
        )"
    ))
    .await?;
    Ok(())
}

async fn applied_checksum(
    db: &DatabaseConnection,
    module: &str,
    version: &str,
) -> anyhow::Result<Option<String>> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT checksum FROM {MIGRATION_LEDGER_TABLE} WHERE module_name = {} AND version = {}",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );
    let row = db
        .query_one(Statement::from_sql_and_values(
            backend,
            sql,
            vec![Value::from(module), Value::from(version)],
        ))
        .await?;

    Ok(row.map(|row| row.try_get("", "checksum")).transpose()?)
}

async fn record_applied(
    db: &DatabaseConnection,
    module: &str,
    migration: &Migration,
    checksum: &str,
) -> anyhow::Result<()> {
    let backend = db.get_database_backend();
    let now = crate::sql::now_iso();
    let sql = format!(
        "INSERT INTO {MIGRATION_LEDGER_TABLE} (module_name, version, name, checksum, applied_at) \
         VALUES ({}, {}, {}, {}, '{}')",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
        crate::sql::placeholder(backend, 3),
        crate::sql::placeholder(backend, 4),
        escape_sql(&now),
    );
    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![
            Value::from(module),
            Value::from(migration.version),
            Value::from(migration.name),
            Value::from(checksum),
        ],
    ))
    .await?;
    Ok(())
}
