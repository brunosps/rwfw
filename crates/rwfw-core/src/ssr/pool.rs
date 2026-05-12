use std::cell::RefCell;

/// Thread-local SSR engine pool using ssr_rs (V8).
/// Each thread gets its own V8 isolate for zero-contention rendering.
pub struct SsrPool {
    js_bundle: String,
}

thread_local! {
    static SSR_ENGINE: RefCell<Option<ssr_rs::Ssr>> = const { RefCell::new(None) };
}

impl SsrPool {
    pub fn new(js_bundle: &str) -> Self {
        Self {
            js_bundle: js_bundle.to_string(),
        }
    }

    /// Render page JSON to HTML using thread-local V8 isolate.
    /// Creates the isolate on first use per thread.
    pub fn render(&self, page_json: &str) -> Option<String> {
        let js_bundle = self.js_bundle.clone();
        let json_owned = page_json.to_string();

        SSR_ENGINE.with(|cell| {
            let mut engine_opt = cell.borrow_mut();

            // Lazy-init the V8 isolate for this thread
            if engine_opt.is_none() {
                tracing::debug!(
                    "Initializing V8 isolate for thread {:?}",
                    std::thread::current().id()
                );
                match ssr_rs::Ssr::from(js_bundle, "RWFWSSR") {
                    Ok(ssr) => *engine_opt = Some(ssr),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to create V8 isolate");
                        return None;
                    }
                }
            }

            let engine = engine_opt.as_mut()?;

            match engine.render_to_string(Some(&json_owned)) {
                Ok(html) => Some(html),
                Err(e) => {
                    tracing::error!(error = %e, "SSR render failed");
                    None
                }
            }
        })
    }
}
