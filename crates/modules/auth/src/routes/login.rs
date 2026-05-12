use crate::config::{login_providers, AuthConfig};
use crate::strategies::local::LocalAuthStrategy;
use crate::use_cases::login_user::{LoginUserInput, LoginUserUseCase};
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /auth/login - Show login page
async fn get(State(state): State<AppState>, i: Inertia) -> impl IntoResponse {
    i.render_with_ssr(
        "auth/Login",
        serde_json::json!({
            "ssoProviders": login_providers(&state),
        }),
    )
    .await
}

/// POST /auth/login - Process login
async fn post(
    State(state): State<AppState>,
    i: Inertia,
    Json(input): Json<LoginUserInput>,
) -> Response {
    let strategy = LocalAuthStrategy::new();
    let use_case = LoginUserUseCase;

    match use_case.execute(&strategy, &state.db, input).await {
        Ok(output) => {
            let ttl = session_ttl(&state);
            match rwfw_core::auth::create_session(&state.db, output.user_id, ttl).await {
                Ok(token) => {
                    let mut response = Inertia::redirect("/home");
                    rwfw_core::auth::append_set_cookie(
                        &mut response,
                        rwfw_core::auth::session_cookie(
                            &token,
                            ttl,
                            !state.config.is_development(),
                        ),
                    );
                    response
                }
                Err(error) => AppError::Internal(error).into_response(),
            }
        }
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}

fn session_ttl(state: &AppState) -> i64 {
    AuthConfig::from_state(state).session_ttl
}
