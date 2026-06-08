use crate::config::AuthConfig;
use crate::strategies::oidc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;
use std::time::Duration;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::post(post)
}

/// POST /auth/logout — Log out.
///
/// Always clears the local session. For SSO sessions it then performs
/// RP-initiated logout (OIDC `end_session_endpoint`) so the IdP session is
/// terminated too — otherwise "Sign in with <provider>" would silently
/// re-authenticate. Any failure (provider removed, no `end_session_endpoint`,
/// network/timeout) falls back to a local-only logout: logout must never hang
/// on IdP reachability.
async fn post(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let mut oidc_logout: Option<(String, String)> = None;
    if let Some(token) = rwfw_core::auth::extract_session_token(&headers) {
        oidc_logout = rwfw_core::auth::session_oidc(&state.db, &token)
            .await
            .ok()
            .flatten();
        if let Err(error) = rwfw_core::auth::delete_session(&state.db, &token).await {
            tracing::warn!(error = %error, "Failed to delete auth session");
        }
    }

    if let Some((provider, id_token)) = oidc_logout {
        let config = AuthConfig::from_state(&state);
        if let Some(provider_config) = config.oidc.providers.get(&provider) {
            let base = config
                .oidc
                .redirect_base_url
                .as_deref()
                .unwrap_or("http://localhost:3000")
                .trim_end_matches('/');
            let post_logout = format!("{base}/auth/login");
            let lookup = tokio::time::timeout(
                Duration::from_secs(5),
                oidc::end_session_url(provider_config, &id_token, &post_logout),
            )
            .await;
            if let Ok(Ok(Some(url))) = lookup {
                let mut response = Inertia::redirect(&url);
                rwfw_core::auth::append_set_cookie(
                    &mut response,
                    rwfw_core::auth::expired_session_cookie(),
                );
                return response;
            }
            tracing::warn!(provider = %provider, "RP-initiated logout unavailable; local logout only");
        }
    }

    let mut response = Inertia::redirect("/auth/login");
    rwfw_core::auth::append_set_cookie(&mut response, rwfw_core::auth::expired_session_cookie());
    response
}
