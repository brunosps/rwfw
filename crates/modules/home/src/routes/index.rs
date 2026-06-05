use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(v: View) -> Response {
    v.render(
        "home/index",
        serde_json::json!({
            "title": "Welcome to RWFW",
            "description": "A modular Rust web framework — Hotwire + MiniJinja, zero npm"
        }),
    )
}
