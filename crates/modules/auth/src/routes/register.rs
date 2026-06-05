use crate::config::AuthConfig;
use crate::repositories::user_repo::UserRepository;
use crate::use_cases::register_user::{RegisterUserInput, RegisterUserUseCase};
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

/// GET /auth/register - Show registration page.
async fn get(v: View) -> Response {
    v.render("auth/register", serde_json::json!({}))
}

/// POST /auth/register - Process registration.
async fn post(
    State(state): State<AppState>,
    v: View,
    Form(input): Form<RegisterUserInput>,
) -> Response {
    let repo = UserRepository::new(state.db.clone());
    let use_case = RegisterUserUseCase;
    let should_assign_admin = rwfw_core::auth::user_count(&state.db)
        .await
        .map(|count| count == 0)
        .unwrap_or(false);

    let name = input.name.clone();
    let email = input.email.clone();

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
            "auth/register",
            serde_json::json!({
                "errors": rwfw_core::validation::first_messages(errors),
                "old": { "name": name, "email": email }
            }),
        ),
        Err(error) => error.into_response(),
    }
}

fn session_ttl(state: &AppState) -> i64 {
    AuthConfig::from_state(state).session_ttl
}
