use crate::config::AppConfig;
use crate::events::EventBus;
use crate::inertia::SharedData;
use crate::module::{Module, NavItem};
use axum::Router;
use axum::middleware;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub events: Arc<EventBus>,
    pub shared_data: Arc<SharedData>,
    modules_nav: Arc<Vec<ModuleNav>>,
}

#[derive(Clone, serde::Serialize)]
pub struct ModuleNav {
    pub name: String,
    pub nav_items: Vec<NavItem>,
}

impl AppState {
    pub fn modules_nav(&self) -> &[ModuleNav] {
        &self.modules_nav
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub state: AppState,
}

pub struct RwfwApp {
    modules: Vec<Box<dyn Module>>,
    config: AppConfig,
}

impl RwfwApp {
    pub fn new(config: AppConfig) -> Self {
        Self {
            modules: Vec::new(),
            config,
        }
    }

    pub fn module(mut self, module: Box<dyn Module>) -> Self {
        self.modules.push(module);
        self
    }

    pub async fn build(self) -> anyhow::Result<Router> {
        let is_development = self.config.is_development();
        let db = crate::db::connect(&self.config).await?;
        let events = Arc::new(EventBus::new());

        let mut modules_nav = Vec::new();
        let mut router = Router::new();

        for module in &self.modules {
            let name = module.name().to_string();
            let prefix = format!("/{}", name);

            tracing::info!(module = %name, prefix = %prefix, "Mounting module");

            // Mount module routes under prefix
            let module_routes = module.routes();
            router = router.nest(&prefix, module_routes);

            // Mount API routes if any
            if let Some(api_routes) = module.api_routes() {
                let api_prefix = format!("/api/{}", name);
                router = router.nest(&api_prefix, api_routes);
            }

            // Collect nav items
            let nav = module.nav_items();
            if !nav.is_empty() {
                modules_nav.push(ModuleNav {
                    name: name.clone(),
                    nav_items: nav,
                });
            }

            // Register event handlers
            for subscription in module.event_handlers() {
                events.subscribe(subscription);
            }
        }

        let shared_data = Arc::new(SharedData::default());

        let state = AppState {
            config: Arc::new(self.config),
            db,
            events,
            shared_data,
            modules_nav: Arc::new(modules_nav),
        };

        // Add health check
        router = router.route("/health", axum::routing::get(health_handler));
        router = router.route("/favicon.ico", axum::routing::get(favicon_handler));

        if !is_development {
            router = router.nest_service("/assets", ServeDir::new("dist/client/assets"));
        }

        router = router.layer(middleware::from_fn_with_state(
            state.clone(),
            crate::inertia::shared::inertia_shared_middleware,
        ));

        let router = router.with_state(state);

        Ok(router)
    }
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn favicon_handler() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}
