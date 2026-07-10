use axum::extract::{FromRequestParts, OriginalUri};
use axum::http::header::ACCEPT;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use serde_json::{Map, Value};
use std::convert::Infallible;

use super::renderer::ViewRenderer;
use crate::app::AppState;
use crate::inertia::shared::InertiaSharedProps;

/// Axum extractor for rendering server-side HTML views.
///
/// Mirrors the ergonomics of the legacy `Inertia` extractor, but produces HTML
/// from MiniJinja templates instead of JSON for React. Shared props
/// (`auth`/`flash`/`errors`/`csrf_token`/`modules`) are injected automatically.
#[derive(Clone)]
pub struct View {
    renderer: ViewRenderer,
    shared: Value,
    reactive_secret_key: Option<Vec<u8>>,
    pub url: String,
    pub is_turbo_stream: bool,
    pub headers: HeaderMap,
}

impl View {
    /// Render `template` (e.g. `"blog/index"`) with `props` merged over shared
    /// props, returning a `200 OK` HTML response.
    pub fn render(&self, template: &str, props: Value) -> Response {
        self.render_status(StatusCode::OK, template, props)
    }

    /// Render with an explicit status (e.g. `422` to re-render a form with errors).
    pub fn render_status(&self, status: StatusCode, template: &str, props: Value) -> Response {
        let ctx = self.merge(props);
        match self
            .renderer
            .render_to_string(template, minijinja::Value::from_serialize(&ctx))
        {
            Ok(html) => (status, Html(html)).into_response(),
            Err(error) => {
                tracing::error!(template = %template, error = %error, "view render failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!(
                        "<h1>Template error</h1><pre>{}</pre>",
                        escape_html(&format!("{error:#}"))
                    )),
                )
                    .into_response()
            }
        }
    }

    /// Render a template fragment (no surrounding layout) to a raw HTML string,
    /// e.g. for Turbo Stream payloads.
    pub fn render_fragment(&self, template: &str, props: Value) -> String {
        let ctx = self.merge(props);
        self.renderer
            .render_to_string(template, minijinja::Value::from_serialize(&ctx))
            .unwrap_or_else(|error| {
                tracing::error!(template = %template, error = %error, "view fragment render failed");
                String::new()
            })
    }

    /// Render a stateless reactive component and return its signed root HTML.
    ///
    /// Handlers pass the returned string into a normal page render and mark it
    /// safe in MiniJinja:
    ///
    /// ```ignore
    /// v.render("page", context! { counter => v.reactive(&Counter { count: 0 }) })
    /// ```
    ///
    /// The component template can use `{{ on("increment") }}` to emit the
    /// phase-1 trigger attributes.
    pub fn reactive<C>(&self, component: &C) -> String
    where
        C: crate::reactive::ReactiveComponent,
    {
        let Some(key) = self.reactive_secret_key.as_deref() else {
            tracing::error!("reactive component rendered without a reactive secret key");
            return String::new();
        };
        crate::reactive::render_component(component, &self.renderer, key).unwrap_or_else(|error| {
            tracing::error!(error = %error, "reactive component render failed");
            String::new()
        })
    }

    fn merge(&self, props: Value) -> Value {
        match (&self.shared, props) {
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

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl FromRequestParts<AppState> for View {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let shared = parts
            .extensions
            .get::<InertiaSharedProps>()
            .map(|props| props.0.clone())
            .unwrap_or_else(|| Value::Object(Map::new()));

        let url = parts
            .extensions
            .get::<OriginalUri>()
            .map(|uri| uri.0.to_string())
            .unwrap_or_else(|| parts.uri.to_string());

        let is_turbo_stream = parts
            .headers
            .get(ACCEPT)
            .and_then(|value| value.to_str().ok())
            .map(|accept| accept.contains("text/vnd.turbo-stream.html"))
            .unwrap_or(false);

        Ok(View {
            renderer: state.view.clone(),
            shared,
            reactive_secret_key: state.reactive_secret_key.clone(),
            url,
            is_turbo_stream,
            headers: parts.headers.clone(),
        })
    }
}
