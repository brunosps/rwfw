use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

/// GET /blog/posts/create - New post form
async fn get(_user: CurrentUser, i: Inertia) -> Response {
    i.render_with_ssr("blog/Create", serde_json::json!({}))
        .await
}
