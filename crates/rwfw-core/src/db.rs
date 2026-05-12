use crate::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};

pub async fn connect(config: &AppConfig) -> anyhow::Result<DatabaseConnection> {
    let database = config.database()?;
    Ok(Database::connect(&database.url).await?)
}
