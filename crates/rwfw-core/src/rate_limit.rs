use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple in-memory rate limiter.
/// For production, use Redis-based rate limiting.
#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    max_requests: u32,
    window: Duration,
}

struct RateLimiterState {
    clients: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                clients: HashMap::new(),
            })),
            max_requests,
            window,
        }
    }

    pub fn check(&self, client_id: &str) -> bool {
        // Recover from a poisoned lock instead of cascading panics: the guarded
        // request-window map stays structurally valid after a handler panic.
        let mut state = self.state.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("rate limiter mutex poisoned; recovering");
            poisoned.into_inner()
        });
        let now = Instant::now();

        let requests = state.clients.entry(client_id.to_string()).or_default();

        // Remove expired entries
        requests.retain(|t| now.duration_since(*t) < self.window);

        if requests.len() >= self.max_requests as usize {
            return false;
        }

        requests.push(now);
        true
    }
}

/// Rate limiting middleware factory
pub fn rate_limit_middleware(
    limiter: RateLimiter,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone
+ Send {
    move |request: Request, next: Next| {
        let limiter = limiter.clone();
        Box::pin(async move {
            // Only throttle auth endpoints (login/register/sso); assets, SSE and
            // app routes pass through untouched.
            if !request.uri().path().starts_with("/auth/") {
                return next.run(request).await;
            }

            // First hop of X-Forwarded-For is the client; fall back to "unknown"
            // (fail-open per process) when no proxy sets the header.
            let client_id = request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(',').next())
                .map(|ip| ip.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            if !limiter.check(&client_id) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                )
                    .into_response();
            }

            next.run(request).await
        })
    }
}
