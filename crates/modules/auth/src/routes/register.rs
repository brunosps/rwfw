use crate::config::AuthConfig;
use crate::repositories::user_repo::UserRepository;
use crate::use_cases::register_user::{RegisterUserInput, RegisterUserUseCase};
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /auth/register - Show registration page
async fn get(i: Inertia) -> impl IntoResponse {
    i.render_with_ssr("auth/Register", serde_json::json!({})).await
}

/// POST /auth/register - Process registration
async fn post(
    State(state): State<AppState>,
    i: Inertia,
    Json(input): Json<RegisterUserInput>,
) -> Response {
    let repo = UserRepository::new(state.db.clone());
    let use_case = RegisterUserUseCase;
    let should_assign_admin = rwfw_core::auth::user_count(&state.db)
        .await
        .map(|count| count == 0)
        .unwrap_or(false);

    match use_case.execute(&repo, input).await {
        Ok(output) => {
            if should_assign_admin {
                if let Err(error) =
                    rwfw_core::auth::assign_role(&state.db, output.user.id, "admin").await
                {
                    return AppError::Internal(error).into_response();
                }
            }

            let ttl = session_ttl(&state);
            match rwfw_core::auth::create_session(&state.db, output.user.id, ttl).await {
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
