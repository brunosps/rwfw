use crate::app::AppState;
use crate::auth::CurrentUser;
use axum::extract::{Request, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;

pub(crate) const VALIDATION_ERRORS_COOKIE: &str = "rwfw_errors";
pub(crate) const FLASH_COOKIE: &str = "rwfw_flash";

#[derive(Debug, Clone, Default, Serialize)]
pub struct SharedData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash: Option<FlashData>,
    pub errors: HashMap<String, String>,
    pub csrf_token: String,
    pub modules: Vec<ModuleNavData>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthData {
    pub user: SharedAuthUser,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SharedAuthUser {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FlashData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleNavData {
    pub name: String,
    pub nav_items: Vec<crate::module::NavItem>,
}

#[derive(Clone, Debug)]
pub struct InertiaSharedProps(pub serde_json::Value);

pub async fn inertia_shared_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if path.starts_with("/assets/")
        || path.starts_with("/vendor/")
        || path.starts_with("/__rwfw/")
        || path == "/health"
    {
        return next.run(request).await;
    }

    let current_user = if let Some(token) = crate::auth::extract_session_token(request.headers()) {
        match crate::auth::find_current_user(&state.db, &token).await {
            Ok(user) => user,
            Err(error) => {
                tracing::warn!(error = %error, "Failed to load current user for Inertia props");
                None
            }
        }
    } else {
        None
    };

    if let Some(user) = &current_user {
        request.extensions_mut().insert(user.clone());
    }

    let validation_errors_cookie =
        cookie_value(request.headers(), VALIDATION_ERRORS_COOKIE).map(str::to_string);
    let validation_errors = validation_errors_cookie
        .as_deref()
        .and_then(validation_errors_from_cookie)
        .unwrap_or_default();
    let flash_cookie = cookie_value(request.headers(), FLASH_COOKIE).map(str::to_string);
    let flash = flash_cookie.as_deref().and_then(flash_from_cookie);

    let csrf_token = request
        .extensions()
        .get::<crate::csrf::CsrfToken>()
        .map(|token| token.0.clone())
        .unwrap_or_default();
    let shared = build_shared_props(&state, current_user, flash, validation_errors, csrf_token);
    request.extensions_mut().insert(InertiaSharedProps(shared));

    let mut response = next.run(request).await;
    if validation_errors_cookie.is_some() {
        if let Ok(value) = expired_validation_errors_cookie().parse() {
            response.headers_mut().append(SET_COOKIE, value);
        }
    }
    if flash_cookie.is_some() {
        if let Ok(value) = expired_flash_cookie().parse() {
            response.headers_mut().append(SET_COOKIE, value);
        }
    }
    response
}

fn build_shared_props(
    state: &AppState,
    current_user: Option<CurrentUser>,
    flash: Option<FlashData>,
    errors: HashMap<String, String>,
    csrf_token: String,
) -> serde_json::Value {
    let auth = current_user.map(|user| AuthData {
        user: SharedAuthUser {
            id: user.id,
            name: user.name,
            email: user.email,
        },
        roles: user.roles,
        permissions: user.permissions,
    });

    serde_json::json!({
        "auth": auth,
        "flash": flash,
        "errors": errors,
        "csrf_token": csrf_token,
        "modules": state.modules_nav(),
        "app_title": state.config.app_title(),
        "is_development": state.config.is_development(),
    })
}

pub(crate) fn validation_errors_cookie(errors: &HashMap<String, String>) -> Option<String> {
    let json = serde_json::to_string(errors).ok()?;
    Some(format!(
        "{VALIDATION_ERRORS_COOKIE}={}; Path=/; Max-Age=60; SameSite=Lax",
        hex_encode(&json)
    ))
}

pub(crate) fn flash_cookie(flash: &FlashData) -> Option<String> {
    let json = serde_json::to_string(flash).ok()?;
    Some(format!(
        "{FLASH_COOKIE}={}; Path=/; Max-Age=60; SameSite=Lax",
        hex_encode(&json)
    ))
}

fn expired_validation_errors_cookie() -> String {
    format!("{VALIDATION_ERRORS_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax")
}

fn expired_flash_cookie() -> String {
    format!("{FLASH_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax")
}

fn validation_errors_from_cookie(value: &str) -> Option<HashMap<String, String>> {
    let json = hex_decode(value)?;
    serde_json::from_str(&json).ok()
}

fn flash_from_cookie(value: &str) -> Option<FlashData> {
    let json = hex_decode(value)?;
    serde_json::from_str(&json).ok()
}

fn cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Option<&'a str> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

fn hex_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for byte in input.as_bytes() {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn hex_decode(input: &str) -> Option<String> {
    if input.len() % 2 != 0 {
        return None;
    }

    let bytes = input
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let hex = std::str::from_utf8(chunk).ok()?;
            u8::from_str_radix(hex, 16).ok()
        })
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}
