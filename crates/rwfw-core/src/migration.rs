use crate::module::Module;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const MIGRATION_LEDGER_SCHEMA: &str = "rwfw";
pub const MIGRATION_LEDGER_TABLE: &str = "migrations";

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: &'static str,
    pub name: &'static str,
    pub sql: &'static str,
}

impl Migration {
    pub const fn new(version: &'static str, name: &'static str, sql: &'static str) -> Self {
        Self { version, name, sql }
    }

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

        db.execute_unprepared(migration.sql).await?;
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
    db.execute_unprepared(&format!(
        "CREATE SCHEMA IF NOT EXISTS {MIGRATION_LEDGER_SCHEMA}"
    ))
    .await?;
    db.execute_unprepared(&format!(
        "CREATE TABLE IF NOT EXISTS {MIGRATION_LEDGER_SCHEMA}.{MIGRATION_LEDGER_TABLE} (
            module_name TEXT NOT NULL,
            version TEXT NOT NULL,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
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
    let sql = format!(
        "SELECT checksum FROM {MIGRATION_LEDGER_SCHEMA}.{MIGRATION_LEDGER_TABLE} WHERE module_name = '{}' AND version = '{}'",
        escape_sql(module),
        escape_sql(version)
    );
    let row = db
        .query_one(Statement::from_string(db.get_database_backend(), sql))
        .await?;

    Ok(row.map(|row| row.try_get("", "checksum")).transpose()?)
}

async fn record_applied(
    db: &DatabaseConnection,
    module: &str,
    migration: &Migration,
    checksum: &str,
) -> anyhow::Result<()> {
    let sql = format!(
        "INSERT INTO {MIGRATION_LEDGER_SCHEMA}.{MIGRATION_LEDGER_TABLE} (module_name, version, name, checksum) \
         VALUES ('{}', '{}', '{}', '{}')",
        escape_sql(module),
        escape_sql(migration.version),
        escape_sql(migration.name),
        escape_sql(checksum)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

fn escape_sql(value: &str) -> String {
    value.replace('\'', "''")
}
