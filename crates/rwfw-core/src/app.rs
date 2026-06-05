use crate::config::AppConfig;
use crate::events::EventBus;
use crate::inertia::SharedData;
use crate::module::{Module, NavItem};
use axum::Router;
use axum::middleware;
use crate::view::{TemplateRoot, ViewRenderer};
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub events: Arc<EventBus>,
    pub shared_data: Arc<SharedData>,
    pub view: ViewRenderer,
    /// Broadcast channel for Turbo Stream fragments delivered over SSE.
    pub broadcaster: tokio::sync::broadcast::Sender<String>,
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
    app_web_root: Option<PathBuf>,
}

impl RwfwApp {
    pub fn new(config: AppConfig) -> Self {
        Self {
            modules: Vec::new(),
            config,
            app_web_root: None,
        }
    }

    /// Set the app-level web root (containing `templates/`, `vendor/`, `assets/`).
    /// Callers pass an absolute path (e.g. via `CARGO_MANIFEST_DIR`) so template
    /// resolution is independent of the process working directory.
    pub fn web_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.app_web_root = Some(path.into());
        self
    }

    pub fn module(mut self, module: Box<dyn Module>) -> Self {
        self.modules.push(module);
        self
    }

    pub async fn build(self) -> anyhow::Result<Router> {
        let db = crate::db::connect(&self.config).await?;
        let events = Arc::new(EventBus::new());

        let mut modules_nav = Vec::new();
        let mut template_roots: Vec<TemplateRoot> = Vec::new();
        let mut router = Router::new();

        for module in &self.modules {
            let name = module.name().to_string();
            let prefix = format!("/{}", name);

            // Register this module's template root (namespaced by module name).
            if let Some(web) = module.web_root() {
                template_roots.push(TemplateRoot::module(&name, web.join("templates")));
            }

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

        // App-level templates (layouts, shared partials) are the fallback root.
        if let Some(app_web) = &self.app_web_root {
            template_roots.push(TemplateRoot::app(app_web.join("templates")));
        }
        let view = ViewRenderer::new(template_roots);

        let shared_data = Arc::new(SharedData::default());
        let (broadcaster, _rx) = tokio::sync::broadcast::channel::<String>(256);

        let state = AppState {
            config: Arc::new(self.config),
            db,
            events,
            shared_data,
            view,
            broadcaster,
            modules_nav: Arc::new(modules_nav),
        };

        // Add health check
        router = router.route("/health", axum::routing::get(health_handler));
        router = router.route("/favicon.ico", axum::routing::get(favicon_handler));

        // Static assets (vendored JS, compiled CSS) served from the app web root
        // in both dev and prod — no npm bundler involved.
        if let Some(app_web) = &self.app_web_root {
            router = router
                .nest_service("/vendor", ServeDir::new(app_web.join("vendor")))
                .nest_service("/assets", ServeDir::new(app_web.join("assets")));
        }

        router = router.layer(middleware::from_fn_with_state(
            state.clone(),
            crate::inertia::shared::inertia_shared_middleware,
        ));
        // CSRF runs outermost: it issues the token (cookie + request extension)
        // before shared props read it, and verifies double-submit on form posts.
        router = router.layer(middleware::from_fn(crate::csrf::csrf_middleware));

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
