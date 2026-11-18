//! Response security headers, layered via `tower-http`'s `SetResponseHeaderLayer`.
//!
//! Each header uses `if_not_present`, so a handler that sets its own value (for
//! example a route that needs a looser CSP) wins. The defaults are conservative
//! but real; the policy is configurable through [`SecurityConfig`].

use crate::config::{DEFAULT_REFERRER_POLICY, SecurityConfig};
use axum::Router;
use axum::http::{HeaderName, HeaderValue, header};
use tower_http::set_header::SetResponseHeaderLayer;

/// Apply the configured security-header layers to the router. A no-op when
/// `headers_enabled` is false.
///
/// Apply it after every layer that can answer on its own (rate limiting, CSRF),
/// so their responses carry the headers too.
pub fn apply_security_headers<S>(mut router: Router<S>, config: &SecurityConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    if !config.headers_enabled {
        return router;
    }

    router = router.layer(SetResponseHeaderLayer::if_not_present(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ));

    if !config.referrer_policy.is_empty() {
        let value = header_value("referrer_policy", &config.referrer_policy)
            .unwrap_or_else(|| HeaderValue::from_static(DEFAULT_REFERRER_POLICY));
        router = with_header(router, header::REFERRER_POLICY, value);
    }

    if let Some(value) = header_value("frame_options", &config.frame_options) {
        router = with_header(router, header::X_FRAME_OPTIONS, value);
    }

    if let Some(csp) = &config.content_security_policy
        && let Some(value) = header_value("content_security_policy", csp)
    {
        router = with_header(router, header::CONTENT_SECURITY_POLICY, value);
    }

    if config.hsts {
        router = with_header(
            router,
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    router
}

fn with_header<S>(router: Router<S>, name: HeaderName, value: HeaderValue) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(SetResponseHeaderLayer::if_not_present(name, value))
}

/// Parse a configured header value, logging (instead of silently dropping) one
/// that is not a valid HTTP header value.
fn header_value(field: &str, raw: &str) -> Option<HeaderValue> {
    match HeaderValue::from_str(raw) {
        Ok(value) => Some(value),
        Err(_) => {
            tracing::warn!(
                field = %format!("security.{field}"),
                "invalid header value in config; ignoring it"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    async fn headers_for(config: &SecurityConfig) -> axum::http::HeaderMap {
        let router: Router = Router::new().route("/", get(|| async { "ok" }));
        let router = apply_security_headers(router, config);
        let response = router
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response.headers().clone()
    }

    #[tokio::test]
    async fn referrer_policy_defaults_to_strict_origin_when_cross_origin() {
        let headers = headers_for(&SecurityConfig::default()).await;
        assert_eq!(
            headers.get(header::REFERRER_POLICY).unwrap(),
            "strict-origin-when-cross-origin"
        );
    }

    #[tokio::test]
    async fn referrer_policy_is_configurable() {
        let mut config = SecurityConfig::default();
        config.referrer_policy = "no-referrer".to_string();
        let headers = headers_for(&config).await;
        assert_eq!(headers.get(header::REFERRER_POLICY).unwrap(), "no-referrer");
    }

    #[tokio::test]
    async fn empty_referrer_policy_omits_the_header() {
        let mut config = SecurityConfig::default();
        config.referrer_policy = String::new();
        let headers = headers_for(&config).await;
        assert!(headers.get(header::REFERRER_POLICY).is_none());
        // The other headers are unaffected.
        assert_eq!(
            headers.get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );
    }

    #[tokio::test]
    async fn invalid_referrer_policy_falls_back_to_the_default() {
        let mut config = SecurityConfig::default();
        config.referrer_policy = "bad\nvalue".to_string();
        let headers = headers_for(&config).await;
        assert_eq!(
            headers.get(header::REFERRER_POLICY).unwrap(),
            DEFAULT_REFERRER_POLICY
        );
    }
}
