use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::post(post)
}

/// POST /auth/logout - Log out
async fn post(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = rwfw_core::auth::extract_session_token(&headers) {
        if let Err(error) = rwfw_core::auth::delete_session(&state.db, &token).await {
            tracing::warn!(error = %error, "Failed to delete auth session");
        }
    }

    let mut response = Inertia::redirect("/auth/login");
    rwfw_core::auth::append_set_cookie(
        &mut response,
        rwfw_core::auth::expired_session_cookie(),
    );
    response
}
