use axum::Router;
use axum::response::Redirect;
use rwfw_core::config::AppConfig;
use rwfw_core::module::{Module, ModuleRegistration};

// Force linker to include module crates so inventory can discover them.
extern crate mod_auth;
extern crate mod_blog;
extern crate mod_home;

pub fn registered_modules() -> Vec<Box<dyn Module>> {
    inventory::iter::<ModuleRegistration>
        .into_iter()
        .map(|reg| {
            tracing::info!(module = %reg.name, "Discovered module");
            reg.create()
        })
        .collect()
}

pub async fn build_router(config: AppConfig) -> anyhow::Result<Router> {
    init_ssr_if_available(&config);

    let mut app_builder = rwfw_core::app::RwfwApp::new(config)
        .web_root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web"));

    for module in registered_modules() {
        app_builder = app_builder.module(module);
    }

    Ok(app_builder
        .build()
        .await?
        .route("/", axum::routing::get(root_redirect)))
}

async fn root_redirect() -> Redirect {
    Redirect::to("/home")
}

pub async fn run_migrations(
    config: &AppConfig,
) -> anyhow::Result<rwfw_core::migration::MigrationReport> {
    let db = rwfw_core::db::connect(config).await?;
    let modules = registered_modules();
    rwfw_core::migration::run_pending_migrations(&db, &modules).await
}

fn init_ssr_if_available(config: &AppConfig) {
    if config.is_development() {
        return;
    }

    let ssr_bundle_path = config
        .vite()
        .ok()
        .and_then(|vite| vite.ssr_bundle_path)
        .unwrap_or_else(|| "dist/server/ssr.js".to_string());

    match std::fs::read_to_string(&ssr_bundle_path) {
        Ok(bundle) => rwfw_core::ssr::init(&bundle),
        Err(error) => {
            tracing::warn!(
                path = %ssr_bundle_path,
                error = %error,
                "SSR bundle not available; falling back to CSR shell"
            );
        }
    }
}
