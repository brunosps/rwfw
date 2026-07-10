use axum::Router;
use axum::response::Response;
use rwfw_core::app::AppState;
use rwfw_core::module::{Module, ModuleRegistration, NavItem};
use rwfw_core::reactive::Ctx;
use rwfw_core::view::View;
use rwfw_macros::{ReactiveComponent, reactive_actions};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ReactiveComponent)]
#[reactive(name = "counter", template = "demo/reactive/counter")]
pub struct Counter {
    pub count: i64,
}

#[reactive_actions]
impl Counter {
    #[action]
    pub fn increment(&mut self, _ctx: &Ctx<'_>) {
        self.count += 1;
    }

    #[action]
    pub fn decrement(&mut self, _ctx: &Ctx<'_>) {
        self.count -= 1;
    }

    #[action]
    pub fn set(&mut self, _ctx: &Ctx<'_>, value: i64) {
        self.count = value;
    }
}

pub struct DemoModule;

impl DemoModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DemoModule {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Module for DemoModule {
    fn name(&self) -> &str {
        "demo"
    }

    fn routes(&self) -> Router<AppState> {
        Router::new().route("/counter", axum::routing::get(counter))
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![NavItem {
            label: "Counter".to_string(),
            href: "/demo/counter".to_string(),
            icon: Some("plus".to_string()),
        }]
    }

    fn web_root(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web"))
    }
}

async fn counter(v: View) -> Response {
    let counter = v.reactive(&Counter { count: 0 });
    v.render("demo/counter", serde_json::json!({ "counter": counter }))
}

inventory::submit! {
    ModuleRegistration::new("demo", || Box::new(DemoModule::new()))
}
