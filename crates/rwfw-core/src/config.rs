use config::{Config, File};
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    inner: Config,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let env = std::env::var("RWFW_ENV").unwrap_or_else(|_| "development".into());

        tracing::info!(environment = %env, "Loading configuration");

        let config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{env}")).required(false))
            .add_source(config::Environment::with_prefix("RWFW").separator("__"))
            .build()?;

        Ok(Self { inner: config })
    }

    pub fn server(&self) -> anyhow::Result<ServerConfig> {
        Ok(self.inner.get::<ServerConfig>("server")?)
    }

    pub fn database(&self) -> anyhow::Result<DatabaseConfig> {
        Ok(self.inner.get::<DatabaseConfig>("database")?)
    }

    pub fn logging(&self) -> anyhow::Result<LoggingConfig> {
        Ok(self.inner.get::<LoggingConfig>("logging")?)
    }

    pub fn module_config<T: DeserializeOwned>(&self, module: &str) -> anyhow::Result<T> {
        Ok(self.inner.get::<T>(module)?)
    }

    pub fn is_development(&self) -> bool {
        std::env::var("RWFW_ENV").unwrap_or_else(|_| "development".into()) == "development"
    }

    /// Build a minimal configuration for tests, overriding only the database URL.
    /// Avoids mutating process-global environment variables across parallel tests.
    #[doc(hidden)]
    pub fn for_test(database_url: &str) -> anyhow::Result<Self> {
        let config = Config::builder()
            .set_override("database.url", database_url)?
            .build()?;
        Ok(Self { inner: config })
    }
}
