use crate::config::{AuthConfig, login_providers};
use crate::strategies::local::LocalAuthStrategy;
use crate::use_cases::login_user::{LoginUserInput, LoginUserUseCase};
use axum::extract::{Form, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;
use std::collections::HashMap;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /auth/login - Show login page.
async fn get(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    v: View,
) -> Response {
    let return_to = query.get("return_to").and_then(|value| safe_return_to(value));
    v.render(
        "auth/login",
        serde_json::json!({ "ssoProviders": login_providers(&state), "return_to": return_to }),
    )
}

/// POST /auth/login - Process login.
async fn post(State(state): State<AppState>, v: View, Form(input): Form<LoginUserInput>) -> Response {
    let strategy = LocalAuthStrategy::new();
    let use_case = LoginUserUseCase;
    let email = input.email.clone();
    let return_to = input.return_to.as_deref().and_then(safe_return_to);
    let target = return_to.as_deref().unwrap_or("/home").to_string();

    match use_case.execute(&strategy, &state.db, input).await {
        Ok(output) => {
            let ttl = session_ttl(&state);
            match rwfw_core::auth::create_session(&state.db, output.user_id, ttl, None, None).await {
                Ok(token) => {
                    let mut response = Redirect::to(&target).into_response();
                    rwfw_core::auth::append_set_cookie(
                        &mut response,
                        rwfw_core::auth::session_cookie(&token, ttl, !state.config.is_development()),
                    );
                    response
                }
                Err(error) => AppError::Internal(error).into_response(),
            }
        }
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "auth/login",
            serde_json::json!({
                "ssoProviders": login_providers(&state),
                "errors": rwfw_core::validation::first_messages(errors),
                "old": { "email": email },
                "return_to": return_to
            }),
        ),
        Err(error) => error.into_response(),
    }
}

fn session_ttl(state: &AppState) -> i64 {
    AuthConfig::from_state(state).session_ttl
}

fn safe_return_to(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with('/') && !value.starts_with("//") && !value.contains("://") {
        Some(value.to_string())
    } else {
        None
    }
}
