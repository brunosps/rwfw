//! Boot the rwfw axum app in-process on an ephemeral loopback port with graceful
//! shutdown, for embedding in a Tauri desktop window (or any host).
//!
//! The default build is GUI-free: it exposes [`serve_in_background`] and
//! [`ServerHandle`] and is fully testable headless (see `tests/headless.rs`).
//! The `desktop` feature adds the Tauri window glue in [`desktop`].

use std::future::Future;
use std::net::SocketAddr;

use axum::Router;
use rwfw_core::config::AppConfig;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "desktop")]
pub mod desktop;

/// Handle to a running in-process server. Triggers graceful shutdown on an
/// explicit [`ServerHandle::shutdown`] or on `Drop`, so a closing window (which
/// drops the handle out of Tauri state) tears the server down.
pub struct ServerHandle {
    addr: SocketAddr,
    token: CancellationToken,
    join: tokio::task::JoinHandle<()>,
}

impl ServerHandle {
    /// The bound loopback address (port is non-zero after binding).
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Base URL, e.g. `http://127.0.0.1:54321`.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// A clone of the cancellation token (e.g. to cancel from a window event).
    pub fn cancellation_token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Trigger graceful shutdown and await the server task.
    pub async fn shutdown(mut self) {
        self.token.cancel();
        // Await via `&mut` (not by-move) because `ServerHandle` is `Drop`.
        let _ = (&mut self.join).await;
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Best-effort: signal shutdown even if `shutdown()` was never called.
        self.token.cancel();
    }
}

/// Bind `127.0.0.1:0`, spawn `axum::serve` with graceful shutdown on a
/// [`CancellationToken`], and return the bound address + a shutdown handle.
///
/// `router_factory` lets callers supply their own router; [`serve_in_background`]
/// uses `rwfw_app::build_router`.
pub async fn serve_in_background_with<F, Fut>(
    config: AppConfig,
    router_factory: F,
) -> anyhow::Result<ServerHandle>
where
    F: FnOnce(AppConfig) -> Fut,
    Fut: Future<Output = anyhow::Result<Router>>,
{
    let router = router_factory(config).await?;

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let addr = listener.local_addr()?;
    tracing::info!(%addr, "rwfw-tauri server listening");

    let token = CancellationToken::new();
    let shutdown_token = token.clone();
    let join = tokio::spawn(async move {
        let result = axum::serve(listener, router)
            .with_graceful_shutdown(async move { shutdown_token.cancelled().await })
            .await;
        if let Err(error) = result {
            tracing::error!(%error, "rwfw-tauri server exited with error");
        }
    });

    Ok(ServerHandle { addr, token, join })
}

/// Default entry point: boots `rwfw_app::build_router` on a loopback port.
pub async fn serve_in_background(config: AppConfig) -> anyhow::Result<ServerHandle> {
    serve_in_background_with(
        config,
        |cfg| async move { rwfw_app::build_router(cfg).await },
    )
    .await
}
