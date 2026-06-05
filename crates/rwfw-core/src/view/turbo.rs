//! Turbo Stream fragment builder (Hotwire). A `TurboStream` serializes to a
//! `<turbo-stream>` element and sets the `text/vnd.turbo-stream.html` content
//! type, for use in form responses and SSE/real-time broadcasts.

use axum::response::{Html, IntoResponse, Response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurboAction {
    Append,
    Prepend,
    Replace,
    Update,
    Remove,
    Before,
    After,
}

impl TurboAction {
    pub fn as_str(self) -> &'static str {
        match self {
            TurboAction::Append => "append",
            TurboAction::Prepend => "prepend",
            TurboAction::Replace => "replace",
            TurboAction::Update => "update",
            TurboAction::Remove => "remove",
            TurboAction::Before => "before",
            TurboAction::After => "after",
        }
    }
}

pub const TURBO_STREAM_CONTENT_TYPE: &str = "text/vnd.turbo-stream.html; charset=utf-8";

#[derive(Clone, Debug)]
pub struct TurboStream {
    action: TurboAction,
    target: String,
    html: String,
}

impl TurboStream {
    pub fn new(action: TurboAction, target: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            action,
            target: target.into(),
            html: html.into(),
        }
    }

    pub fn render(&self) -> String {
        if self.action == TurboAction::Remove {
            format!(
                r#"<turbo-stream action="remove" target="{}"></turbo-stream>"#,
                self.target
            )
        } else {
            format!(
                r#"<turbo-stream action="{}" target="{}"><template>{}</template></turbo-stream>"#,
                self.action.as_str(),
                self.target,
                self.html
            )
        }
    }
}

impl IntoResponse for TurboStream {
    fn into_response(self) -> Response {
        let mut response = Html(self.render()).into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            TURBO_STREAM_CONTENT_TYPE.parse().unwrap(),
        );
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_append_stream() {
        let stream = TurboStream::new(TurboAction::Append, "posts", "<li>x</li>");
        assert_eq!(
            stream.render(),
            r#"<turbo-stream action="append" target="posts"><template><li>x</li></template></turbo-stream>"#
        );
    }

    #[test]
    fn renders_remove_without_template() {
        let stream = TurboStream::new(TurboAction::Remove, "post_1", "");
        assert_eq!(
            stream.render(),
            r#"<turbo-stream action="remove" target="post_1"></turbo-stream>"#
        );
    }
}
