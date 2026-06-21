//! CSRF protection via the double-submit cookie pattern.
//!
//! A random token is issued in the `rwfw_csrf` cookie (HttpOnly) and rendered
//! into pages via shared props (`csrf_token`) as a `<meta name="csrf-token">`
//! tag plus a hidden `_csrf` form field. Turbo automatically echoes the meta
//! value back as the `X-CSRF-Token` header on form submissions; the middleware
//! verifies that header against the cookie for mutating, form-encoded requests.
//!
//! JSON bodies, `/api/*`, and SSO callbacks are exempt (they rely on
//! SameSite cookies, not a double-submit token); browser-driven form posts are
//! always verified.

use axum::extract::Request;
use axum::http::header::{CONTENT_TYPE, COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub const CSRF_COOKIE: &str = "rwfw_csrf";
pub const CSRF_HEADER: &str = "x-csrf-token";

/// The current request's CSRF token, inserted into request extensions by
/// [`csrf_middleware`] and read by the shared-props middleware.
#[derive(Clone, Debug)]
pub struct CsrfToken(pub String);

/// Generate a CSRF token (random hex string).
pub fn generate_token() -> String {
    let first = uuid::Uuid::new_v4().simple();
    let second = uuid::Uuid::new_v4().simple();
    format!("{first}{second}")
}

/// Constant-time token comparison.
pub fn verify_token(request_token: &str, session_token: &str) -> bool {
    if request_token.len() != session_token.len() {
        return false;
    }
    request_token
        .bytes()
        .zip(session_token.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// CSRF protection middleware (outermost layer).
///
/// - Issues/propagates the token (cookie + request extension) for every
///   non-static request, so pages can render it.
/// - Verifies the double-submit on mutating, form-encoded, non-exempt requests.
pub async fn csrf_middleware(mut request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    if is_static(&path) {
        return next.run(request).await;
    }

    let cookie_token = cookie_value(request.headers(), CSRF_COOKIE).map(str::to_string);
    let (token, fresh) = match cookie_token.clone() {
        Some(existing) if !existing.is_empty() => (existing, false),
        _ => (generate_token(), true),
    };
    request.extensions_mut().insert(CsrfToken(token.clone()));

    let method = request.method().clone();
    let mutating = method != Method::GET
        && method != Method::HEAD
        && method != Method::OPTIONS
        && method != Method::TRACE;
    if mutating && requires_csrf(&path, request.headers()) {
        let submitted = request
            .headers()
            .get(CSRF_HEADER)
            .and_then(|value| value.to_str().ok());
        let valid = matches!(
            (cookie_token.as_deref(), submitted),
            (Some(cookie), Some(sent)) if verify_token(sent, cookie)
        );
        if !valid {
            return (StatusCode::FORBIDDEN, "CSRF token missing or invalid").into_response();
        }
    }

    let mut response = next.run(request).await;
    if fresh {
        // `Secure` outside development so the token cookie is HTTPS-only.
        // `SameSite=Lax` (not Strict) is required so the cookie survives the
        // top-level redirect back from an SSO provider.
        let secure = if is_secure_env() { "; Secure" } else { "" };
        let cookie = format!("{CSRF_COOKIE}={token}; Path=/; SameSite=Lax; HttpOnly{secure}");
        if let Ok(value) = cookie.parse() {
            response.headers_mut().append(SET_COOKIE, value);
        }
    }
    response
}

/// True when running outside the `development` environment (mirrors
/// `AppConfig::is_development`), so cookies can be marked `Secure`. Read from the
/// env directly because the CSRF layer has no `AppState` handle.
fn is_secure_env() -> bool {
    std::env::var("RWFW_ENV")
        .map(|env| env != "development")
        .unwrap_or(false)
}

fn is_static(path: &str) -> bool {
    path.starts_with("/assets/")
        || path.starts_with("/vendor/")
        || path == "/health"
        || path == "/favicon.ico"
}

/// Verification applies to browser-driven mutations (HTML form posts and
/// Turbo `data-turbo-method` PUT/PATCH/DELETE, which send `X-CSRF-Token`).
/// Exempt: JSON bodies, `/api/*`, and SSO callbacks.
fn requires_csrf(path: &str, headers: &HeaderMap) -> bool {
    if path.starts_with("/api/") || path.contains("/sso/") {
        return false;
    }
    let is_json = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);
    !is_json
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let header = headers.get(COOKIE)?.to_str().ok()?;
    header.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_token_is_constant_time_equal() {
        assert!(verify_token("abc", "abc"));
        assert!(!verify_token("abc", "abd"));
        assert!(!verify_token("abc", "abcd"));
    }

    #[test]
    fn json_and_sso_are_exempt() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        assert!(!requires_csrf("/blog/posts", &headers));

        let mut form = HeaderMap::new();
        form.insert(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        assert!(requires_csrf("/blog/posts", &form));
        assert!(!requires_csrf("/auth/sso/google/callback", &form));
        assert!(!requires_csrf("/api/blog/posts", &form));
    }
}
