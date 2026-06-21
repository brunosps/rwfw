use crate::config::OidcProviderConfig;
use crate::repositories::oauth_repo::ExternalIdentity;
use anyhow::Context;
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::reqwest;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

pub struct OidcAuthorization {
    pub authorization_url: String,
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

pub async fn authorization_url(
    provider: &OidcProviderConfig,
    redirect_url: &str,
) -> anyhow::Result<OidcAuthorization> {
    let (client_id, client_secret) = client_credentials(provider)?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata(provider).await?,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut request = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge);

    for scope in effective_scopes(provider) {
        // The authorization-code flow already requests `openid`; adding it again
        // would emit a duplicate `scope=openid openid`.
        if scope == "openid" {
            continue;
        }
        request = request.add_scope(Scope::new(scope));
    }

    let (authorization_url, state, nonce) = request.url();

    Ok(OidcAuthorization {
        authorization_url: authorization_url.to_string(),
        state: state.secret().to_string(),
        nonce: nonce.secret().to_string(),
        pkce_verifier: pkce_verifier.secret().to_string(),
    })
}

pub async fn exchange_code(
    provider_name: &str,
    provider: &OidcProviderConfig,
    redirect_url: &str,
    code: &str,
    nonce: &str,
    pkce_verifier: &str,
) -> anyhow::Result<(ExternalIdentity, String)> {
    let (client_id, client_secret) = client_credentials(provider)?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata(provider).await?,
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?);
    let http_client = http_client()?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))?
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
        .request_async(&http_client)
        .await
        .context("failed to exchange OIDC authorization code")?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow::anyhow!("OIDC provider did not return an id_token"))?;
    // Raw compact JWT, kept so logout can send it back as `id_token_hint`.
    let id_token_raw = id_token.to_string();
    let id_token_verifier = client.id_token_verifier();
    let nonce = Nonce::new(nonce.to_string());
    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .context("failed to verify OIDC id_token")?;

    if let Some(expected_access_token_hash) = claims.access_token_hash() {
        let actual_access_token_hash = AccessTokenHash::from_token(
            token_response.access_token(),
            id_token.signing_alg()?,
            id_token.signing_key(&id_token_verifier)?,
        )?;
        if actual_access_token_hash != *expected_access_token_hash {
            anyhow::bail!("OIDC access token hash verification failed");
        }
    }

    let subject = claims.subject().as_str().to_string();
    let email = claims
        .email()
        .map(|email| email.as_str().to_string())
        .ok_or_else(|| anyhow::anyhow!("OIDC provider did not return an email claim"))?;
    let email_verified = claims.email_verified().unwrap_or(false);
    let name = claims
        .name()
        .and_then(|name| name.get(None))
        .map(|name| name.as_str().to_string())
        .or_else(|| {
            claims
                .preferred_username()
                .map(|username| username.as_str().to_string())
        })
        .unwrap_or_else(|| email.clone());
    let claims = serde_json::to_value(claims).context("failed to serialize OIDC claims")?;

    Ok((
        ExternalIdentity {
            provider: provider_name.to_string(),
            issuer: provider.issuer_url.clone(),
            subject,
            email,
            email_verified,
            name,
            claims,
        },
        id_token_raw,
    ))
}

/// Build the RP-initiated logout URL (OpenID Connect Session Management
/// `end_session_endpoint`) so logging out of the app also terminates the IdP
/// session. Returns `None` if the provider's discovery document has no
/// `end_session_endpoint`. Sends `id_token_hint` (so the IdP knows which session
/// to end), `post_logout_redirect_uri`, and `client_id`.
pub async fn end_session_url(
    provider: &OidcProviderConfig,
    id_token: &str,
    post_logout_redirect: &str,
) -> anyhow::Result<Option<String>> {
    let discovery_url = format!(
        "{}/.well-known/openid-configuration",
        provider.issuer_url.trim_end_matches('/')
    );
    let http_client = http_client()?;
    let doc: serde_json::Value = http_client
        .get(&discovery_url)
        .send()
        .await
        .context("failed to fetch OIDC discovery document")?
        .json()
        .await
        .context("failed to parse OIDC discovery document")?;

    let Some(endpoint) = doc.get("end_session_endpoint").and_then(|v| v.as_str()) else {
        return Ok(None);
    };

    let (client_id, _) = client_credentials(provider)?;
    Ok(Some(build_end_session_url(
        endpoint,
        &client_id,
        id_token,
        post_logout_redirect,
    )?))
}

/// Build the RP-initiated logout URL (pure; no network) from a discovered
/// `end_session_endpoint` plus the hint/redirect/client params.
fn build_end_session_url(
    endpoint: &str,
    client_id: &str,
    id_token: &str,
    post_logout_redirect: &str,
) -> anyhow::Result<String> {
    let mut url =
        openidconnect::url::Url::parse(endpoint).context("invalid end_session_endpoint URL")?;
    url.query_pairs_mut()
        .append_pair("id_token_hint", id_token)
        .append_pair("post_logout_redirect_uri", post_logout_redirect)
        .append_pair("client_id", client_id);
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::build_end_session_url;

    #[test]
    fn end_session_url_carries_hint_redirect_and_client() {
        let url = build_end_session_url(
            "https://idp.example/realms/x/protocol/openid-connect/logout",
            "my-client",
            "the.id.token",
            "https://app.example/auth/login",
        )
        .unwrap();
        assert!(url.starts_with("https://idp.example/realms/x/protocol/openid-connect/logout?"));
        assert!(url.contains("id_token_hint=the.id.token"));
        assert!(url.contains("client_id=my-client"));
        assert!(url.contains("post_logout_redirect_uri=https%3A%2F%2Fapp.example%2Fauth%2Flogin"));
    }
}

fn client_credentials(provider: &OidcProviderConfig) -> anyhow::Result<(String, String)> {
    let client_id = std::env::var(&provider.client_id_env).with_context(|| {
        format!(
            "missing OIDC client id env var `{}`",
            provider.client_id_env
        )
    })?;
    let client_secret = std::env::var(&provider.client_secret_env).with_context(|| {
        format!(
            "missing OIDC client secret env var `{}`",
            provider.client_secret_env
        )
    })?;

    Ok((client_id, client_secret))
}

async fn provider_metadata(provider: &OidcProviderConfig) -> anyhow::Result<CoreProviderMetadata> {
    let http_client = http_client()?;
    CoreProviderMetadata::discover_async(IssuerUrl::new(provider.issuer_url.clone())?, &http_client)
        .await
        .context("failed to discover OIDC provider metadata")
}

fn http_client() -> anyhow::Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}

fn effective_scopes(provider: &OidcProviderConfig) -> Vec<String> {
    let mut scopes = if provider.scopes.is_empty() {
        vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ]
    } else {
        provider.scopes.clone()
    };

    if !scopes.iter().any(|scope| scope == "openid") {
        scopes.insert(0, "openid".to_string());
    }

    scopes
}
