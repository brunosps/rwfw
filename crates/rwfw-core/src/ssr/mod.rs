pub mod pool;

use pool::SsrPool;
use std::sync::OnceLock;

static SSR_POOL: OnceLock<SsrPool> = OnceLock::new();

/// Initialize the SSR engine with a JavaScript bundle.
/// Must be called once before any rendering.
pub fn init(js_bundle: &str) {
    ssr_rs::Ssr::create_platform();
    let pool = SsrPool::new(js_bundle);
    SSR_POOL.set(pool).ok();
    tracing::info!("SSR engine initialized");
}

/// Render a page to HTML string using the V8 SSR engine.
/// This runs on a blocking thread to avoid blocking the async runtime.
/// Returns None if SSR is not initialized (dev mode).
pub async fn render(page_json: &str) -> Option<String> {
    let pool = SSR_POOL.get()?;
    let json = page_json.to_string();

    let result = tokio::task::spawn_blocking(move || pool.render(&json))
        .await
        .ok()?;

    result
}

/// Check if SSR is available (initialized)
pub fn is_available() -> bool {
    SSR_POOL.get().is_some()
}
