use config::{Config, File};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

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

/// Opt-in web security knobs (response headers + auth rate limiting), read from
/// the `security:` config section. Every field has a safe default so apps that
/// omit the section still get sensible protection (headers on, HSTS/rate-limit
/// off — both are deployment-specific).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Emit X-Content-Type-Options / X-Frame-Options / Referrer-Policy / CSP.
    pub headers_enabled: bool,
    /// Emit Strict-Transport-Security (only safe behind TLS; default off).
    pub hsts: bool,
    /// Content-Security-Policy value; `None` skips the header.
    pub content_security_policy: Option<String>,
    /// X-Frame-Options value.
    pub frame_options: String,
    pub rate_limit: RateLimitConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            headers_enabled: true,
            hsts: false,
            content_security_policy: Some(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; \
                 font-src 'self' data:; object-src 'none'; base-uri 'self'; \
                 frame-ancestors 'self'"
                    .to_string(),
            ),
            frame_options: "SAMEORIGIN".to_string(),
            rate_limit: RateLimitConfig::default(),
        }
    }
}

/// In-memory rate limiting for auth routes. Disabled by default (in-memory state
/// does not survive restarts or scale across processes — opt in per deployment).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests: u32,
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests: 10,
            window_secs: 60,
        }
    }
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

    pub fn app_title(&self) -> String {
        self.inner
            .get::<String>("app.title")
            .unwrap_or_else(|_| "RWFW".to_string())
    }

    pub fn module_config<T: DeserializeOwned>(&self, module: &str) -> anyhow::Result<T> {
        Ok(self.inner.get::<T>(module)?)
    }

    /// Security knobs from the `security:` section, falling back to defaults
    /// (headers on, HSTS/rate-limit off) when the section is absent or partial.
    pub fn security(&self) -> SecurityConfig {
        self.inner
            .get::<SecurityConfig>("security")
            .unwrap_or_default()
    }

    pub fn is_development(&self) -> bool {
        std::env::var("RWFW_ENV").unwrap_or_else(|_| "development".into()) == "development"
    }

    pub fn is_test(&self) -> bool {
        std::env::var("RWFW_ENV").unwrap_or_else(|_| "development".into()) == "test" || cfg!(test)
    }

    /// Secret key used to sign stateless reactive component tokens.
    ///
    /// In development and tests, a missing `RWFW_SECRET_KEY` derives a stable
    /// per-project key so local counters work out of the box. In production,
    /// callers pass `required = true` only when a reactive component is
    /// registered, preserving boot behavior for apps that never opt in.
    pub fn reactive_secret_key(&self, required: bool) -> anyhow::Result<Option<Vec<u8>>> {
        if let Ok(secret) = self.inner.get::<String>("secret_key")
            && !secret.is_empty()
        {
            return Ok(Some(secret.into_bytes()));
        }

        if self.is_development() || self.is_test() {
            let root = std::env::current_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "rwfw-development".to_string());
            tracing::warn!(
                "RWFW_SECRET_KEY is unset; deriving a development reactive token key from the project path"
            );
            let digest = Sha256::digest(format!("rwfw-reactive:{root}").as_bytes());
            return Ok(Some(digest.to_vec()));
        }

        if required {
            anyhow::bail!(
                "RWFW_SECRET_KEY must be set when reactive components are registered in production"
            );
        }

        Ok(None)
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

    /// Build a test config backed by a SQLite file (or `:memory:`). Used by the
    /// SQLite test harness so the suite runs with no external Postgres.
    #[doc(hidden)]
    pub fn for_test_sqlite(path: &str) -> anyhow::Result<Self> {
        let url = if path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            crate::db::sqlite_url_from_path(path)
        };
        Self::for_test(&url)
    }
}
