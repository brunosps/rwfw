use crate::config::AuthConfig;
use crate::repositories::oauth_repo::OauthRepository;
use crate::strategies::oidc;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;
use serde::Deserialize;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn get(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if let Some(error) = query.error {
        tracing::warn!(
            provider = %provider,
            error = %error,
            description = ?query.error_description,
            "OIDC provider returned an error"
        );
        return Inertia::redirect_with_error("/auth/login", "SSO login was cancelled or denied");
    }

    let Some(code) = query.code else {
        return Inertia::redirect_with_error("/auth/login", "Missing SSO authorization code");
    };
    let Some(returned_state) = query.state else {
        return Inertia::redirect_with_error("/auth/login", "Missing SSO state");
    };

    let config = AuthConfig::from_state(&state);
    let Some(provider_config) = config.oidc.providers.get(&provider) else {
        return Inertia::redirect_with_error("/auth/login", "Unknown SSO provider");
    };

    let repo = OauthRepository::new(state.db.clone());
    let Some(saved_state) = (match repo.take_login_state(&provider, &returned_state).await {
        Ok(saved_state) => saved_state,
        Err(error) => {
            tracing::warn!(error = %error, provider = %provider, "Failed to load OIDC state");
            return Inertia::redirect_with_error("/auth/login", "Invalid SSO login state");
        }
    }) else {
        return Inertia::redirect_with_error("/auth/login", "Expired or invalid SSO login state");
    };

    let redirect_url = callback_url(config.oidc.redirect_base_url.as_deref(), &provider);
    let identity = match oidc::exchange_code(
        &provider,
        provider_config,
        &redirect_url,
        &code,
        &saved_state.nonce,
        &saved_state.pkce_verifier,
    )
    .await
    {
        Ok(identity) => identity,
        Err(error) => {
            tracing::warn!(error = %error, provider = %provider, "OIDC callback failed");
            return Inertia::redirect_with_error("/auth/login", "SSO login failed");
        }
    };

    let user_id = match repo
        .resolve_or_create_user(identity, provider_config.require_verified_email)
        .await
    {
        Ok(user_id) => user_id,
        Err(error) => {
            tracing::warn!(error = %error, provider = %provider, "Failed to resolve OIDC user");
            return Inertia::redirect_with_error("/auth/login", "SSO user is not allowed");
        }
    };

    let ttl = config.session_ttl;
    let token = match rwfw_core::auth::create_session(&state.db, user_id, ttl).await {
        Ok(token) => token,
        Err(error) => {
            tracing::warn!(error = %error, provider = %provider, "Failed to create RWFW session");
            return Inertia::redirect_with_error("/auth/login", "Failed to create session");
        }
    };

    let target = saved_state.return_to.as_deref().unwrap_or("/home");
    let mut response = Inertia::redirect_with_success(target, "Signed in with SSO");
    rwfw_core::auth::append_set_cookie(
        &mut response,
        rwfw_core::auth::session_cookie(&token, ttl, secure_cookie(&state, &headers)),
    );
    response
}

fn callback_url(base_url: Option<&str>, provider: &str) -> String {
    let base_url = base_url.unwrap_or("http://localhost:3000").trim_end_matches('/');
    format!("{base_url}/auth/sso/{provider}/callback")
}

fn secure_cookie(state: &AppState, headers: &HeaderMap) -> bool {
    !state.config.is_development()
        || headers
            .get("x-forwarded-proto")
            .and_then(|value| value.to_str().ok())
            .map(|proto| proto.eq_ignore_ascii_case("https"))
            .unwrap_or(false)
}
