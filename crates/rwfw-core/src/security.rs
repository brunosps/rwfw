//! Response security headers, layered via `tower-http`'s `SetResponseHeaderLayer`.
//!
//! Each header uses `if_not_present`, so a handler that sets its own value (for
//! example a route that needs a looser CSP) wins. The defaults are conservative
//! but real; the policy is configurable through [`SecurityConfig`].

use crate::app::AppState;
use crate::config::SecurityConfig;
use axum::Router;
use axum::http::{HeaderValue, header};
use tower_http::set_header::SetResponseHeaderLayer;

/// Apply the configured security-header layers to the router. A no-op when
/// `headers_enabled` is false.
pub fn apply_security_headers(mut router: Router<AppState>, config: &SecurityConfig) -> Router<AppState> {
    if !config.headers_enabled {
        return router;
    }

    router = router
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    if let Ok(value) = HeaderValue::from_str(&config.frame_options) {
        router = router.layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            value,
        ));
    }

    if let Some(csp) = &config.content_security_policy {
        if let Ok(value) = HeaderValue::from_str(csp) {
            router = router.layer(SetResponseHeaderLayer::if_not_present(
                header::CONTENT_SECURITY_POLICY,
                value,
            ));
        }
    }

    if config.hsts {
        router = router.layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    router
}
