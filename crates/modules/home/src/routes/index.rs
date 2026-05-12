use axum::response::IntoResponse;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(i: Inertia) -> impl IntoResponse {
    i.render_with_ssr(
        "home/Index",
        serde_json::json!({
            "title": "Welcome to RWFW",
            "description": "A modular Rust web framework with React + SSR"
        }),
    )
    .await
}
