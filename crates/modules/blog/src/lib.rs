pub mod migrations;
pub mod models;
pub mod queries;
pub mod repositories;
pub mod use_cases;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::auth::Permission;
use rwfw_core::module::{Module, ModuleRegistration, NavItem};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct BlogRoutes;

pub struct BlogModule;

impl BlogModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Module for BlogModule {
    fn name(&self) -> &str {
        "blog"
    }

    fn routes(&self) -> Router<AppState> {
        BlogRoutes::generated_routes()
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![NavItem {
            label: "Posts".to_string(),
            href: "/blog/posts".to_string(),
            icon: Some("file-text".to_string()),
        }]
    }

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {
        migrations::migrations()
    }

    fn permissions(&self) -> Vec<Permission> {
        vec![
            Permission::new("blog.posts.view", "View blog posts"),
            Permission::new("blog.posts.create", "Create blog posts"),
            Permission::new("blog.posts.update", "Update blog posts"),
            Permission::new("blog.posts.delete", "Delete blog posts"),
        ]
    }
}

inventory::submit! {
    ModuleRegistration::new("blog", || Box::new(BlogModule::new()))
}
