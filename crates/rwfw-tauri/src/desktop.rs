//! Tauri glue (feature `desktop`): boot the in-process server and point a
//! webview window at it.
//!
//! `tauri::generate_context!()` is intentionally NOT here — it must live in the
//! final app crate, which owns `tauri.conf.json`. A generated desktop `main.rs`
//! does:
//!
//! ```ignore
//! fn main() -> anyhow::Result<()> {
//!     let config = rwfw_core::config::AppConfig::load()?;
//!     tauri::async_runtime::block_on(my_app::run_migrations(&config))?;
//!     tauri::Builder::default()
//!         .setup(move |app| {
//!             rwfw_tauri::desktop::attach(app, config, my_app::build_router)
//!                 .map_err(|e| e.to_string())?;
//!             Ok(())
//!         })
//!         .run(tauri::generate_context!())
//!         .map_err(|e| anyhow::anyhow!("tauri: {e}"))?;
//!     Ok(())
//! }
//! ```

use std::future::Future;

use axum::Router;
use rwfw_core::config::AppConfig;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::ServerHandle;

/// Boot `router_factory` in-process on a loopback port, open a webview window
/// pointed at it, and cancel the server when that window closes. Call from a
/// Tauri `setup` hook. Migrations are the caller's responsibility (run them
/// before building the window).
pub fn attach<F, Fut>(app: &tauri::App, config: AppConfig, router_factory: F) -> anyhow::Result<()>
where
    F: FnOnce(AppConfig) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<Router>> + Send + 'static,
{
    // Tauri's async runtime is Tokio; block the setup hook on async boot.
    let handle: ServerHandle =
        tauri::async_runtime::block_on(crate::serve_in_background_with(config, router_factory))?;

    let url = format!("{}/", handle.base_url());
    let token = handle.cancellation_token();

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse()?))
        .title("RWFW")
        .inner_size(1200.0, 800.0)
        .build()?;

    window.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
        ) {
            token.cancel();
        }
    });

    // Park the handle in Tauri state so its `Drop` (which cancels the server)
    // does not fire at the end of `setup`; the server lives until app exit.
    app.manage(handle);
    Ok(())
}
