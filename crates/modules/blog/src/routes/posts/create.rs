use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

/// GET /blog/posts/create - New post form (requires authentication).
async fn get(_user: CurrentUser, v: View) -> Response {
    v.render("blog/create", serde_json::json!({}))
}
