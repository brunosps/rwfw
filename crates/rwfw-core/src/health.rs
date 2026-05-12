use axum::Json;
use serde_json::Value;

/// Health check handler
/// Returns service status information
pub async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
