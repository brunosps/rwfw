use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Generate a CSRF token (random hex string)
pub fn generate_token() -> String {
    let first = uuid::Uuid::new_v4().simple();
    let second = uuid::Uuid::new_v4().simple();
    format!("{first}{second}")
}

/// Verify a CSRF token from the request against the expected token
pub fn verify_token(request_token: &str, session_token: &str) -> bool {
    // Constant-time comparison to prevent timing attacks
    if request_token.len() != session_token.len() {
        return false;
    }
    request_token
        .bytes()
        .zip(session_token.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// CSRF protection middleware.
/// Checks for a valid CSRF token on mutating requests (POST, PUT, DELETE, PATCH).
/// Skips validation for XHR/Inertia requests (they use CORS + SameSite cookies).
pub async fn csrf_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();

    // Only check mutating methods
    if method == "GET" || method == "HEAD" || method == "OPTIONS" {
        return next.run(request).await;
    }

    // Skip CSRF check for Inertia XHR requests (they have X-Inertia header)
    if request.headers().get("X-Inertia").is_some() {
        return next.run(request).await;
    }

    // TODO: Extract CSRF token from form data or header and verify against session
    // For now, pass through
    next.run(request).await
}
