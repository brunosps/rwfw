use anyhow::Context;
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SsoProviderKind {
    Oidc,
    Keycloak,
    AzureB2c,
}

pub struct SsoAddInput {
    pub name: String,
    pub provider: SsoProviderKind,
    pub issuer_url: String,
    pub client_id_env: String,
    pub client_secret_env: String,
    pub display_name: Option<String>,
    pub redirect_base_url: Option<String>,
    pub scopes: String,
    pub force: bool,
}

#[derive(Debug, Serialize)]
struct SsoProviderConfig {
    provider: String,
    display_name: String,
    issuer_url: String,
    client_id_env: String,
    client_secret_env: String,
    scopes: Vec<String>,
    require_verified_email: bool,
}

pub async fn create_admin(
    name: &str,
    email: &str,
    password: Option<&str>,
    update_password: bool,
) -> anyhow::Result<()> {
    let password = resolve_password(password)?;
    let db = connect()
        .await
        .context("failed to connect to the application database")?;

    let report = rwfw_core::auth::ensure_admin_user(
        &db,
        rwfw_core::auth::EnsureAdminUserInput {
            name,
            email,
            password: &password,
            update_password,
        },
    )
    .await
    .context("failed to create admin user; run `rwfw migrate` first if auth tables are missing")?;

    if report.created {
        println!("Created admin user `{}`.", report.email);
    } else {
        println!("Admin user `{}` already exists.", report.email);
    }

    if report.role_assigned {
        println!("Assigned role `admin`.");
    } else {
        println!("Role `admin` was already assigned.");
    }

    if !report.created && report.password_updated {
        println!("Updated admin password.");
    } else if !report.created && !report.password_updated {
        println!("Password unchanged. Use `--update-password` to replace it.");
    }

    Ok(())
}

pub fn sso_add(input: SsoAddInput) -> anyhow::Result<()> {
    let project_root = find_project_root(&std::env::current_dir()?).ok_or_else(|| {
        anyhow::anyhow!("could not find rwfw.toml in this directory or its parents")
    })?;
    let provider_name = normalize_provider_name(&input.name)?;
    let display_name = input
        .display_name
        .clone()
        .unwrap_or_else(|| default_display_name(input.provider, &provider_name));
    let redirect_base_url = input
        .redirect_base_url
        .clone()
        .unwrap_or_else(|| "http://localhost:3000".to_string());
    let scopes = parse_scopes(&input.scopes);

    let provider_config = SsoProviderConfig {
        provider: provider_kind_label(input.provider).to_string(),
        display_name,
        issuer_url: input.issuer_url,
        client_id_env: input.client_id_env,
        client_secret_env: input.client_secret_env,
        scopes,
        require_verified_email: true,
    };

    for relative_path in ["config/development.yaml", "config/production.yaml"] {
        let path = project_root.join(relative_path);
        upsert_sso_provider(
            &path,
            &provider_name,
            &redirect_base_url,
            &provider_config,
            input.force,
        )
        .with_context(|| format!("updating {}", path.display()))?;
    }

    update_env_example(&project_root, &provider_config)?;

    println!("Configured SSO provider `{provider_name}`.");
    println!(
        "Redirect URI: {}/auth/sso/{provider_name}/callback",
        redirect_base_url.trim_end_matches('/')
    );
    println!("Set these variables before starting the app:");
    println!("  {}=<client id>", provider_config.client_id_env);
    println!("  {}=<client secret>", provider_config.client_secret_env);

    Ok(())
}

fn resolve_password(password: Option<&str>) -> anyhow::Result<String> {
    let password = password
        .map(str::to_string)
        .or_else(|| std::env::var("RWFW_ADMIN_PASSWORD").ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing admin password; pass `--password` or set `RWFW_ADMIN_PASSWORD`"
            )
        })?;

    if password.len() < 8 {
        anyhow::bail!("Admin password must be at least 8 characters");
    }

    Ok(password)
}

async fn connect() -> anyhow::Result<sea_orm::DatabaseConnection> {
    let config = rwfw_core::config::AppConfig::load()?;
    rwfw_core::db::connect(&config).await
}

fn find_project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join("rwfw.toml").exists())
        .map(Path::to_path_buf)
}

fn normalize_provider_name(name: &str) -> anyhow::Result<String> {
    let normalized = name
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if normalized.is_empty() {
        anyhow::bail!("SSO provider name must contain at least one ASCII letter or number");
    }

    Ok(normalized)
}

fn parse_scopes(scopes: &str) -> Vec<String> {
    let mut parsed = scopes
        .split([' ', ','])
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    if !parsed.iter().any(|scope| scope == "openid") {
        parsed.insert(0, "openid".to_string());
    }

    parsed
}

fn provider_kind_label(kind: SsoProviderKind) -> &'static str {
    match kind {
        SsoProviderKind::Oidc => "oidc",
        SsoProviderKind::Keycloak => "keycloak",
        SsoProviderKind::AzureB2c => "azure-b2c",
    }
}

fn default_display_name(kind: SsoProviderKind, provider_name: &str) -> String {
    match kind {
        SsoProviderKind::Oidc => titleize(provider_name),
        SsoProviderKind::Keycloak => "Keycloak".to_string(),
        SsoProviderKind::AzureB2c => "Azure AD B2C".to_string(),
    }
}

fn titleize(value: &str) -> String {
    value
        .split(['-', '_'])
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

fn upsert_sso_provider(
    path: &Path,
    name: &str,
    redirect_base_url: &str,
    provider: &SsoProviderConfig,
    force: bool,
) -> anyhow::Result<()> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut document = if existing.trim().is_empty() {
        Value::Mapping(Mapping::new())
    } else {
        serde_yaml::from_str::<Value>(&existing)?
    };

    let root = document
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("YAML root must be a mapping"))?;
    let auth = ensure_mapping(root, "auth");
    if !auth.contains_key(Value::String("session_ttl".to_string())) {
        auth.insert(
            Value::String("session_ttl".to_string()),
            Value::Number(86_400.into()),
        );
    }

    let oidc = ensure_mapping(auth, "oidc");
    if !oidc.contains_key(Value::String("redirect_base_url".to_string())) {
        oidc.insert(
            Value::String("redirect_base_url".to_string()),
            Value::String(redirect_base_url.to_string()),
        );
    }

    let providers = ensure_mapping(oidc, "providers");
    let provider_key = Value::String(name.to_string());
    if providers.contains_key(&provider_key) && !force {
        anyhow::bail!("SSO provider `{name}` already exists; pass --force to replace it");
    }
    providers.insert(provider_key, serde_yaml::to_value(provider)?);

    fs::write(path, serde_yaml::to_string(&document)?)?;
    Ok(())
}

fn ensure_mapping<'a>(mapping: &'a mut Mapping, key: &str) -> &'a mut Mapping {
    let value = mapping
        .entry(Value::String(key.to_string()))
        .or_insert_with(|| Value::Mapping(Mapping::new()));

    if !matches!(value, Value::Mapping(_)) {
        *value = Value::Mapping(Mapping::new());
    }

    value.as_mapping_mut().expect("mapping just ensured")
}

fn update_env_example(project_root: &Path, provider: &SsoProviderConfig) -> anyhow::Result<()> {
    let path = project_root.join(".env.example");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;

    if !content.contains(&provider.client_id_env) {
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&format!("{}=\n", provider.client_id_env));
        changed = true;
    }

    if !content.contains(&provider.client_secret_env) {
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&format!("{}=\n", provider.client_secret_env));
        changed = true;
    }

    if changed {
        fs::write(path, content)?;
    }

    Ok(())
}
