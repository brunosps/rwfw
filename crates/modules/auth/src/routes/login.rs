use crate::config::{AuthConfig, login_providers};
use crate::strategies::local::LocalAuthStrategy;
use crate::use_cases::login_user::{LoginUserInput, LoginUserUseCase};
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /auth/login - Show login page.
async fn get(State(state): State<AppState>, v: View) -> Response {
    v.render(
        "auth/login",
        serde_json::json!({ "ssoProviders": login_providers(&state) }),
    )
}

/// POST /auth/login - Process login.
async fn post(State(state): State<AppState>, v: View, Form(input): Form<LoginUserInput>) -> Response {
    let strategy = LocalAuthStrategy::new();
    let use_case = LoginUserUseCase;
    let email = input.email.clone();

    match use_case.execute(&strategy, &state.db, input).await {
        Ok(output) => {
            let ttl = session_ttl(&state);
            match rwfw_core::auth::create_session(&state.db, output.user_id, ttl).await {
                Ok(token) => {
                    let mut response = Redirect::to("/home").into_response();
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
                "old": { "email": email }
            }),
        ),
        Err(error) => error.into_response(),
    }
}

fn session_ttl(state: &AppState) -> i64 {
    AuthConfig::from_state(state).session_ttl
}
