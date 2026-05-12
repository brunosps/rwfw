use rwfw_core::app::AppState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_session_ttl")]
    pub session_ttl: i64,
    #[serde(default)]
    pub oidc: OidcConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct OidcConfig {
    pub redirect_base_url: Option<String>,
    #[serde(default)]
    pub providers: BTreeMap<String, OidcProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OidcProviderConfig {
    #[serde(default = "default_provider_kind")]
    pub provider: String,
    pub display_name: Option<String>,
    pub issuer_url: String,
    pub client_id_env: String,
    pub client_secret_env: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "default_require_verified_email")]
    pub require_verified_email: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginProvider {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "loginUrl")]
    pub login_url: String,
}

impl AuthConfig {
    pub fn from_state(state: &AppState) -> Self {
        state
            .config
            .module_config::<AuthConfig>("auth")
            .unwrap_or_default()
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            session_ttl: default_session_ttl(),
            oidc: OidcConfig::default(),
        }
    }
}

impl OidcProviderConfig {
    pub fn display_name(&self, fallback: &str) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| titleize_provider_name(fallback))
    }
}

pub fn login_providers(state: &AppState) -> Vec<LoginProvider> {
    AuthConfig::from_state(state)
        .oidc
        .providers
        .into_iter()
        .map(|(name, provider)| LoginProvider {
            display_name: provider.display_name(&name),
            login_url: format!("/auth/sso/{name}/start"),
            name,
        })
        .collect()
}

fn titleize_provider_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn default_session_ttl() -> i64 {
    86_400
}

fn default_provider_kind() -> String {
    "oidc".to_string()
}

fn default_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ]
}

fn default_require_verified_email() -> bool {
    true
}
