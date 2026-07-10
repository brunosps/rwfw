use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::{Ctx, ReactiveError, Reply};
use crate::app::AppState;
use crate::view::turbo::{TurboAction, TurboStream};

#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub token: String,
    pub act: String,
    #[serde(default = "empty_params")]
    pub params: serde_json::Value,
}

pub async fn actions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ActionRequest>,
) -> Response {
    match handle_action(&state, &headers, request).await {
        Ok(response) => response,
        Err(error) => error_response(&state, error),
    }
}

async fn handle_action(
    state: &AppState,
    headers: &HeaderMap,
    request: ActionRequest,
) -> Result<Response, EndpointError> {
    let key = state
        .reactive_secret_key
        .as_deref()
        .ok_or_else(|| EndpointError::bad_request("reactive components are not configured"))?;
    let payload = super::token::verify(&request.token, key)
        .map_err(|error| EndpointError::bad_request(error.to_string()))?;
    let registration = super::find(&payload.c)
        .ok_or_else(|| EndpointError::bad_request("unknown reactive component"))?;

    let current_user = current_user(state, headers).await;
    let ctx = Ctx {
        state,
        user: current_user.as_ref(),
    };
    let output = (registration.dispatch)(payload.s, &request.act, request.params, &ctx)
        .map_err(EndpointError::from)?;

    match output.reply {
        Reply::Nothing => Ok(StatusCode::NO_CONTENT.into_response()),
        Reply::Replace { morph } => {
            let html = super::render_registered_output(&output, &state.view, key)
                .map_err(EndpointError::from)?;
            let stream = if morph {
                TurboStream::new(TurboAction::Replace, output.dom_id, html).method("morph")
            } else {
                TurboStream::new(TurboAction::Replace, output.dom_id, html)
            };
            Ok(stream.into_response())
        }
    }
}

async fn current_user(state: &AppState, headers: &HeaderMap) -> Option<crate::auth::CurrentUser> {
    let token = crate::auth::extract_session_token(headers)?;
    match crate::auth::find_current_user(&state.db, &token).await {
        Ok(user) => user,
        Err(error) => {
            tracing::warn!(error = %error, "failed to load current user for reactive action");
            None
        }
    }
}

fn error_response(state: &AppState, error: EndpointError) -> Response {
    if state.config.is_development() {
        (error.status, error.message).into_response()
    } else {
        tracing::warn!(
            status = error.status.as_u16(),
            error = %error.message,
            "reactive action failed"
        );
        error.status.into_response()
    }
}

fn empty_params() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

struct EndpointError {
    status: StatusCode,
    message: String,
}

impl EndpointError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl From<ReactiveError> for EndpointError {
    fn from(error: ReactiveError) -> Self {
        let status = match error {
            ReactiveError::Unauthorized | ReactiveError::UnknownAction { .. } => {
                StatusCode::FORBIDDEN
            }
            ReactiveError::NotFound => StatusCode::NOT_FOUND,
            ReactiveError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ReactiveError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            status,
            message: error.to_string(),
        }
    }
}
