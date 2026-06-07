use crate::config::AppConfig;
use crate::inertia::SharedData;
use crate::module::{Module, NavItem};
use axum::Router;
use axum::middleware;
use crate::view::{TemplateRoot, ViewRenderer};
use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub shared_data: Arc<SharedData>,
    pub view: ViewRenderer,
    /// Broadcast channel for Turbo Stream fragments delivered over SSE.
    pub broadcaster: tokio::sync::broadcast::Sender<String>,
    /// Disk-overlay root (`<exe>/web`) for runtime asset/template overrides.
    pub overlay_root: Option<PathBuf>,
    /// App web root (`CARGO_MANIFEST_DIR/web`) for disk asset serving in dev.
    pub app_web_root: Option<PathBuf>,
    /// Embedded static assets (vendor + assets); packaged fallback.
    pub asset_embed: Option<crate::view::AssetEmbed>,
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
    template_embed: Option<crate::view::TemplateEmbed>,
    asset_embed: Option<crate::view::AssetEmbed>,
}

impl RwfwApp {
    pub fn new(config: AppConfig) -> Self {
        Self {
            modules: Vec::new(),
            config,
            app_web_root: None,
            template_embed: None,
            asset_embed: None,
        }
    }

    /// Provide the embedded, canonical-keyed template baseline (from the app
    /// crate's `rust-embed`). Used as the fallback source in a packaged binary.
    pub fn template_embed(mut self, embed: crate::view::TemplateEmbed) -> Self {
        self.template_embed = Some(embed);
        self
    }

    /// Provide embedded static assets (vendor + assets); packaged fallback when
    /// the on-disk web root is absent.
    pub fn asset_embed(mut self, embed: crate::view::AssetEmbed) -> Self {
        self.asset_embed = Some(embed);
        self
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
        let security = self.config.security();

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
        }

        // App-level templates (layouts, shared partials) are the fallback root.
        if let Some(app_web) = &self.app_web_root {
            template_roots.push(TemplateRoot::app(app_web.join("templates")));
        }
        let overlay_root = crate::view::exe_overlay_root();
        let view = ViewRenderer::layered(crate::view::Layers {
            overlay_root: overlay_root.clone(),
            roots: template_roots,
            embed: self.template_embed.clone(),
        });

        let shared_data = Arc::new(SharedData::default());
        let (broadcaster, _rx) = tokio::sync::broadcast::channel::<String>(256);

        let state = AppState {
            config: Arc::new(self.config),
            db,
            shared_data,
            view,
            broadcaster,
            overlay_root,
            app_web_root: self.app_web_root.clone(),
            asset_embed: self.asset_embed.clone(),
            modules_nav: Arc::new(modules_nav),
        };

        // Add health check
        router = router.route("/health", axum::routing::get(health_handler));
        router = router.route("/favicon.ico", axum::routing::get(favicon_handler));
        router = router.route("/components", axum::routing::get(components_catalog));

        // Static assets, available to every app built via RwfwApp. `/assets`
        // resolves disk overlay -> app web root -> embed; `/vendor` is
        // disk-or-embed only (locked). Registered before `with_state` so the
        // handlers can read `AppState`.
        router = router
            .route("/vendor/{*path}", axum::routing::get(vendor_handler))
            .route("/assets/{*path}", axum::routing::get(asset_handler));

        router = router.layer(middleware::from_fn_with_state(
            state.clone(),
            crate::inertia::shared::inertia_shared_middleware,
        ));
        // CSRF runs outermost: it issues the token (cookie + request extension)
        // before shared props read it, and verifies double-submit on form posts.
        router = router.layer(middleware::from_fn(crate::csrf::csrf_middleware));

        // Security response headers (opt-out via `security.headers_enabled`).
        router = crate::security::apply_security_headers(router, &security);

        // Optional per-process rate limiting on `/auth/*` (opt-in; default off).
        if security.rate_limit.enabled {
            let limiter = crate::rate_limit::RateLimiter::new(
                security.rate_limit.max_requests,
                std::time::Duration::from_secs(security.rate_limit.window_secs),
            );
            router = router.layer(middleware::from_fn(
                crate::rate_limit::rate_limit_middleware(limiter),
            ));
        }

        let router = router.with_state(state);

        Ok(router)
    }
}

/// Dev-only component catalog: lists discovered `<x-...>` components + props.
async fn components_catalog(
    state: axum::extract::State<AppState>,
    v: crate::view::View,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !state.config.is_development() {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    let components = state.view.components();
    v.render(
        "components_catalog",
        serde_json::json!({ "components": components }),
    )
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

/// `/vendor/*` — locked JS. Served from the app web root (dev) or the embedded
/// baseline (packaged); never from the disk overlay.
async fn vendor_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(root) = &state.app_web_root {
        if let Some(fp) = crate::view::safe_join(&root.join("vendor"), &path) {
            if let Ok(bytes) = std::fs::read(&fp) {
                return bytes_response(&path, bytes.into());
            }
        }
    }
    if let Some(embed) = &state.asset_embed {
        if let Some(bytes) = embed.vendor(&path) {
            return bytes_response(&path, bytes);
        }
    }
    axum::http::StatusCode::NOT_FOUND.into_response()
}

/// `/assets/*` — disk overlay (`<exe>/web/assets`) → app web root (dev) → embed.
async fn asset_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(overlay) = &state.overlay_root {
        if let Some(fp) = crate::view::safe_join(&overlay.join("assets"), &path) {
            if let Ok(bytes) = std::fs::read(&fp) {
                return bytes_response(&path, bytes.into());
            }
        }
    }
    if let Some(root) = &state.app_web_root {
        if let Some(fp) = crate::view::safe_join(&root.join("assets"), &path) {
            if let Ok(bytes) = std::fs::read(&fp) {
                return bytes_response(&path, bytes.into());
            }
        }
    }
    if let Some(embed) = &state.asset_embed {
        if let Some(bytes) = embed.asset(&path) {
            return bytes_response(&path, bytes);
        }
    }
    axum::http::StatusCode::NOT_FOUND.into_response()
}

fn bytes_response(rel: &str, bytes: std::borrow::Cow<'static, [u8]>) -> axum::response::Response {
    use axum::response::IntoResponse;
    let mime = mime_guess::from_path(rel).first_or_octet_stream();
    let ctype = axum::http::HeaderValue::from_str(mime.as_ref())
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream"));
    (
        [(axum::http::header::CONTENT_TYPE, ctype)],
        bytes.into_owned(),
    )
        .into_response()
}
