use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use rwfw_core::app::AppState;

/// Auth guard middleware — redirects to /auth/login if no session
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let current_user = if let Some(user) = request
        .extensions()
        .get::<rwfw_core::auth::CurrentUser>()
        .cloned()
    {
        Some(user)
    } else if let Some(token) = rwfw_core::auth::extract_session_token(request.headers()) {
        match rwfw_core::auth::find_current_user(&state.db, &token).await {
            Ok(user) => user,
            Err(error) => {
                tracing::warn!(error = %error, "Failed to validate auth session");
                None
            }
        }
    } else {
        None
    };

    let Some(user) = current_user else {
        return Redirect::to("/auth/login").into_response();
    };

    request.extensions_mut().insert(user);
    next.run(request).await
}
