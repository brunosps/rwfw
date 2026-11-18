//! Stand-in for the OIDC strategy when mod-auth is built without the `oidc`
//! feature. Same public API, so the `/auth/sso/*` routes still compile: starting
//! or finishing an SSO login fails with a clear error (the routes turn it into a
//! redirect to the login page), and logout ends only the local session.

use crate::config::OidcProviderConfig;
use crate::repositories::oauth_repo::ExternalIdentity;

const DISABLED: &str = "SSO/OIDC is disabled: build mod-auth with the `oidc` feature";

pub struct OidcAuthorization {
    pub authorization_url: String,
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

pub async fn authorization_url(
    _provider: &OidcProviderConfig,
    _redirect_url: &str,
) -> anyhow::Result<OidcAuthorization> {
    anyhow::bail!(DISABLED)
}

pub async fn exchange_code(
    _provider_name: &str,
    _provider: &OidcProviderConfig,
    _redirect_url: &str,
    _code: &str,
    _nonce: &str,
    _pkce_verifier: &str,
) -> anyhow::Result<(ExternalIdentity, String)> {
    anyhow::bail!(DISABLED)
}

/// No IdP to notify: the caller ends the local session only.
pub async fn end_session_url(
    _provider: &OidcProviderConfig,
    _id_token: &str,
    _post_logout_redirect: &str,
) -> anyhow::Result<Option<String>> {
    Ok(None)
}
