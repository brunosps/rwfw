use crate::app::AppState;
use crate::auth::Permission;
use crate::migration::Migration;
use axum::Router;
use serde::Serialize;
use std::any::Any;

/// Check a module's [`Module::api_prefix`]: `/api/` plus at least one static
/// segment, no trailing slash, no route parameters.
pub fn validate_api_prefix(module: &str, prefix: &str) -> anyhow::Result<()> {
    let rest = prefix.strip_prefix("/api/").ok_or_else(|| {
        anyhow::anyhow!(
            "module `{module}`: api_prefix `{prefix}` must start with `/api/` \
             (that prefix is what exempts API routes from CSRF)"
        )
    })?;
    let valid = !rest.is_empty()
        && !rest.ends_with('/')
        && rest
            .split('/')
            .all(|segment| !segment.is_empty() && !segment.contains(['{', '}', '*', ':']));
    if !valid {
        anyhow::bail!(
            "module `{module}`: api_prefix `{prefix}` must look like `/api/<segment>[/<segment>...]` \
             with no trailing slash or route parameters"
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    pub icon: Option<String>,
}

#[async_trait::async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn routes(&self) -> Router<AppState>;
    fn api_routes(&self) -> Option<Router<AppState>> {
        None
    }
    /// Where [`Module::api_routes`] is mounted. Defaults to `/api/<name>`.
    ///
    /// Override it to version an API (`/api/v1`) without renaming the module:
    /// the name also sets the HTML prefix, the template namespace, and the
    /// migration and permission keys. It must start with `/api/`, because that
    /// prefix is what exempts API calls from the CSRF check.
    fn api_prefix(&self) -> String {
        format!("/api/{}", self.name())
    }
    fn migrations(&self) -> Vec<Migration> {
        vec![]
    }
    fn permissions(&self) -> Vec<Permission> {
        vec![]
    }
    async fn migrate(&self, db: &sea_orm::DatabaseConnection) -> anyhow::Result<()> {
        crate::migration::run_module_migrations(db, self.name(), self.migrations()).await?;
        Ok(())
    }
    fn services(&self) -> Vec<Box<dyn Any + Send + Sync>> {
        vec![]
    }
    fn nav_items(&self) -> Vec<NavItem> {
        vec![]
    }
    /// Filesystem root for this module's web assets (`web/templates`,
    /// `web/components`). Implemented per-module via `CARGO_MANIFEST_DIR` so the
    /// path is absolute and independent of the process working directory.
    fn web_root(&self) -> Option<std::path::PathBuf> {
        None
    }
}

pub struct ModuleRegistration {
    pub name: &'static str,
    pub factory: fn() -> Box<dyn Module>,
}

impl ModuleRegistration {
    pub const fn new(name: &'static str, factory: fn() -> Box<dyn Module>) -> Self {
        Self { name, factory }
    }

    pub fn create(&self) -> Box<dyn Module> {
        (self.factory)()
    }
}

inventory::collect!(ModuleRegistration);

#[cfg(test)]
mod tests {
    use super::validate_api_prefix;

    #[test]
    fn accepts_api_prefixes() {
        for prefix in ["/api/blog", "/api/v1", "/api/v1/sync"] {
            validate_api_prefix("m", prefix).unwrap();
        }
    }

    #[test]
    fn rejects_prefixes_outside_api_or_malformed() {
        for prefix in [
            "/v1",
            "/apiv1",
            "/api",
            "/api/",
            "/api/v1/",
            "/api//v1",
            "/api/{tenant}",
            "/api/*rest",
        ] {
            assert!(
                validate_api_prefix("m", prefix).is_err(),
                "{prefix} should be rejected"
            );
        }
    }
}
