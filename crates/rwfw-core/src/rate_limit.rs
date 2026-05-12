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
        let mut state = self.state.lock().unwrap();
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
            // Use client IP as identifier (from X-Forwarded-For or peer addr)
            let client_id = request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();

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
