pub mod config;
pub mod middleware;
pub mod migrations;
pub mod models;
pub mod queries;
pub mod repositories;
pub mod strategies;
pub mod use_cases;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::auth::Permission;
use rwfw_core::module::{Module, ModuleRegistration};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct AuthRoutes;

pub struct AuthModule;

impl AuthModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Module for AuthModule {
    fn name(&self) -> &str {
        "auth"
    }

    fn routes(&self) -> Router<AppState> {
        AuthRoutes::generated_routes()
    }

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {
        migrations::migrations()
    }

    fn permissions(&self) -> Vec<Permission> {
        vec![
            Permission::new("auth.users.manage", "Manage users"),
            Permission::new("auth.roles.manage", "Manage roles"),
            Permission::new("auth.permissions.manage", "Manage permissions"),
            Permission::new("auth.sessions.manage", "Manage sessions"),
        ]
    }

    fn web_root(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web"))
    }
}

inventory::submit! {
    ModuleRegistration::new("auth", || Box::new(AuthModule::new()))
}
