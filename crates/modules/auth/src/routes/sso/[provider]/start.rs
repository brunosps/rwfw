use crate::config::AuthConfig;
use crate::repositories::oauth_repo::{CreateLoginState, OauthRepository};
use crate::strategies::oidc;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;
use std::collections::HashMap;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let config = AuthConfig::from_state(&state);
    let Some(provider_config) = config.oidc.providers.get(&provider) else {
        return Inertia::redirect_with_error("/auth/login", "Unknown SSO provider");
    };

    let redirect_url = callback_url(config.oidc.redirect_base_url.as_deref(), &provider);
    let auth = match oidc::authorization_url(provider_config, &redirect_url).await {
        Ok(auth) => auth,
        Err(error) => {
            tracing::warn!(error = %error, provider = %provider, "Failed to start OIDC login");
            return Inertia::redirect_with_error("/auth/login", "Failed to start SSO login");
        }
    };

    let repo = OauthRepository::new(state.db.clone());
    let return_to = query.get("return_to").and_then(|value| safe_return_to(value));
    if let Err(error) = repo
        .save_login_state(CreateLoginState {
            provider: provider.clone(),
            state: auth.state,
            nonce: auth.nonce,
            pkce_verifier: auth.pkce_verifier,
            return_to,
            expires_at: (chrono::Utc::now() + chrono::Duration::minutes(10)).fixed_offset(),
        })
        .await
    {
        tracing::warn!(error = %error, provider = %provider, "Failed to persist OIDC state");
        return Inertia::redirect_with_error("/auth/login", "Failed to start SSO login");
    }

    axum::response::Redirect::to(&auth.authorization_url).into_response()
}

fn callback_url(base_url: Option<&str>, provider: &str) -> String {
    let base_url = base_url.unwrap_or("http://localhost:3000").trim_end_matches('/');
    format!("{base_url}/auth/sso/{provider}/callback")
}

fn safe_return_to(value: &str) -> Option<String> {
    if value.starts_with('/') && !value.starts_with("//") {
        Some(value.to_string())
    } else {
        None
    }
}
