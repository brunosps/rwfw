use axum::response::Redirect;
use axum::routing;
use rwfw_core::app::AppState;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

/// GET /blog - module root redirects to the (public) post list.
async fn get() -> Redirect {
    Redirect::to("/blog/posts")
}
