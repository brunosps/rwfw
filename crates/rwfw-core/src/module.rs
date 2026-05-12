use crate::app::AppState;
use crate::auth::Permission;
use crate::migration::Migration;
use axum::Router;
use serde::Serialize;
use std::any::Any;

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
    fn event_handlers(&self) -> Vec<crate::events::EventSubscription> {
        vec![]
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
