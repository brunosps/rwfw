use axum::extract::OriginalUri;
use axum::http::header::{HeaderMap, REFERER, SET_COOKIE};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
use serde_json::{Map, Value};

/// The Inertia extractor for Axum handlers.
/// Detects X-Inertia header and provides render/redirect methods.
#[derive(Clone)]
pub struct Inertia {
    pub is_inertia: bool,
    pub version: Option<String>,
    pub url: String,
    pub headers: HeaderMap,
    pub shared_props: Value,
}

impl Inertia {
    pub fn from_request(headers: &HeaderMap, uri: &str) -> Self {
        Self::from_request_with_shared(headers, uri, Value::Object(Map::new()))
    }

    pub fn from_request_with_shared(headers: &HeaderMap, uri: &str, shared_props: Value) -> Self {
        let is_inertia = headers
            .get("X-Inertia")
            .map(|v| v.to_str().unwrap_or("") == "true")
            .unwrap_or(false);

        let version = headers
            .get("X-Inertia-Version")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            is_inertia,
            version,
            url: uri.to_string(),
            headers: headers.clone(),
            shared_props,
        }
    }

    /// Render an Inertia page response.
    /// - XHR with X-Inertia header → JSON response
    /// - Initial page load + SSR available → HTML with server-rendered content
    /// - Initial page load + no SSR → HTML shell with data-page for CSR
    pub fn render(&self, component: &str, props: Value) -> Response {
        let page = InertiaPage {
            component: component.to_string(),
            props: self.merge_props(props),
            url: self.url.clone(),
            version: "1.0".to_string(),
        };

        if self.is_inertia {
            // XHR request - return JSON
            let mut response = axum::Json(&page).into_response();
            response
                .headers_mut()
                .insert("X-Inertia", "true".parse().unwrap());
            response
                .headers_mut()
                .insert("Vary", "X-Inertia".parse().unwrap());
            response
        } else {
            // Initial page load - return HTML shell
            let page_json = serde_json::to_string(&page).unwrap_or_default();

            // In production, attempt SSR. In dev, always CSR.
            let html = crate::inertia::response::render_html_shell(&page_json, None);
            Html(html).into_response()
        }
    }

    /// Render with SSR support (async version for use when SSR is desired)
    pub async fn render_with_ssr(&self, component: &str, props: Value) -> Response {
        let page = InertiaPage {
            component: component.to_string(),
            props: self.merge_props(props),
            url: self.url.clone(),
            version: "1.0".to_string(),
        };

        if self.is_inertia {
            let mut response = axum::Json(&page).into_response();
            response
                .headers_mut()
                .insert("X-Inertia", "true".parse().unwrap());
            response
                .headers_mut()
                .insert("Vary", "X-Inertia".parse().unwrap());
            response
        } else {
            let page_json = serde_json::to_string(&page).unwrap_or_default();

            // Try SSR if available
            let ssr_html = crate::ssr::render(&page_json).await;
            let html = crate::inertia::response::render_html_shell(&page_json, ssr_html.as_deref());
            Html(html).into_response()
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

    fn merge_props(&self, props: Value) -> Value {
        match (&self.shared_props, props) {
            (Value::Object(shared), Value::Object(page)) => {
                let mut merged = shared.clone();
                for (key, value) in page {
                    merged.insert(key, value);
                }
                Value::Object(merged)
            }
            (_, page) => page,
        }
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

#[derive(Debug, Serialize)]
pub struct InertiaPage {
    pub component: String,
    pub props: Value,
    pub url: String,
    pub version: String,
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
