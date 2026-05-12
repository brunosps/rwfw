use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(user: CurrentUser, i: Inertia) -> Response {
    if !user.can("blog.posts.view") {
        return AppError::Forbidden("Missing permission: blog.posts.view".into()).into_response();
    }

    i.render_with_ssr(
        "blog/Index",
        serde_json::json!({
            "title": "Blog",
            "posts": [],
            "pagination": { "page": 1, "per_page": 20, "total": 0, "total_pages": 1 }
        }),
    )
    .await
}
