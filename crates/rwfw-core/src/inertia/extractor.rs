use axum::extract::OriginalUri;
use axum::http::header::{HeaderMap, REFERER, SET_COOKIE};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value};

/// Server-side redirect helper, retained from the legacy Inertia integration.
///
/// The React/Inertia render path is gone (replaced by [`crate::view::View`] +
/// Hotwire); only the redirect helpers survive because SSO/logout flows still
/// use flash- and error-cookie redirects.
#[derive(Clone)]
pub struct Inertia {
    pub url: String,
    pub headers: HeaderMap,
    pub shared_props: Value,
}

impl Inertia {
    pub fn from_request(headers: &HeaderMap, uri: &str) -> Self {
        Self::from_request_with_shared(headers, uri, Value::Object(Map::new()))
    }

    pub fn from_request_with_shared(headers: &HeaderMap, uri: &str, shared_props: Value) -> Self {
        Self {
            url: uri.to_string(),
            headers: headers.clone(),
            shared_props,
        }
    }

    pub fn redirect(to: &str) -> Response {
        axum::response::Redirect::to(to).into_response()
    }

    pub fn redirect_with_success(to: &str, message: impl Into<String>) -> Response {
        Self::redirect_with_flash(
            to,
            crate::inertia::shared::FlashData {
                success: Some(message.into()),
                ..Default::default()
            },
        )
    }

    pub fn redirect_with_error(to: &str, message: impl Into<String>) -> Response {
        Self::redirect_with_flash(
            to,
            crate::inertia::shared::FlashData {
                error: Some(message.into()),
                ..Default::default()
            },
        )
    }

    pub fn redirect_with_info(to: &str, message: impl Into<String>) -> Response {
        Self::redirect_with_flash(
            to,
            crate::inertia::shared::FlashData {
                info: Some(message.into()),
                ..Default::default()
            },
        )
    }

    fn redirect_with_flash(to: &str, flash: crate::inertia::shared::FlashData) -> Response {
        let mut response = axum::response::Redirect::to(to).into_response();
        if let Some(cookie) = crate::inertia::shared::flash_cookie(&flash) {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(SET_COOKIE, value);
            }
        }
        response
    }

    pub fn redirect_back_with_errors(
        &self,
        errors: std::collections::HashMap<String, Vec<String>>,
    ) -> Response {
        let errors = errors
            .into_iter()
            .map(|(field, messages)| {
                (
                    field,
                    messages
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| "Invalid value".to_string()),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();

        let target = self.redirect_back_target();
        let mut response = axum::response::Redirect::to(&target).into_response();
        if let Some(cookie) = crate::inertia::shared::validation_errors_cookie(&errors) {
            if let Ok(value) = cookie.parse() {
                response.headers_mut().append(SET_COOKIE, value);
            }
        }
        response
    }

    fn redirect_back_target(&self) -> String {
        self.headers
            .get(REFERER)
            .and_then(|value| value.to_str().ok())
            .and_then(referer_path)
            .unwrap_or_else(|| self.url.clone())
    }
}

fn referer_path(referer: &str) -> Option<String> {
    if referer.starts_with('/') {
        return Some(referer.to_string());
    }

    let scheme_end = referer.find("://")?;
    let after_host = &referer[scheme_end + 3..];
    let path_start = after_host.find('/')?;
    Some(after_host[path_start..].to_string())
}

impl<S> axum::extract::FromRequestParts<S> for Inertia
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let shared_props = parts
            .extensions
            .get::<crate::inertia::shared::InertiaSharedProps>()
            .map(|props| props.0.clone())
            .unwrap_or_else(|| Value::Object(Map::new()));
        let uri = parts
            .extensions
            .get::<OriginalUri>()
            .map(|uri| uri.0.to_string())
            .unwrap_or_else(|| parts.uri.to_string());

        Ok(Inertia::from_request_with_shared(
            &parts.headers,
            &uri,
            shared_props,
        ))
    }
}
