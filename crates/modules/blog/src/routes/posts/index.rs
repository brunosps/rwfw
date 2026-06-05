use crate::repositories::post_repo::PostRepository;
use crate::use_cases::create_post::{CreatePostInput, CreatePostUseCase};
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /blog/posts - public list of posts.
async fn get(State(state): State<AppState>, v: View) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let (posts, total) = match repo.find_all(1, 20).await {
        Ok(result) => result,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "blog/posts/index",
        serde_json::json!({
            "posts": posts,
            "pagination": {
                "page": 1,
                "per_page": 20,
                "total": total,
                "total_pages": if total == 0 { 1 } else { total.div_ceil(20) }
            }
        }),
    )
}

/// POST /blog/posts - create a new post (requires authentication).
async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Form(input): Form<CreatePostInput>,
) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let use_case = CreatePostUseCase;

    let title = input.title.clone();
    let body = input.body.clone();

    match use_case.execute(&repo, user.id, input).await {
        Ok(output) => Redirect::to(&format!("/blog/posts/{}", output.post.id)).into_response(),
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blog/create",
            serde_json::json!({
                "errors": rwfw_core::validation::first_messages(errors),
                "old": { "title": title, "body": body }
            }),
        ),
        Err(error) => error.into_response(),
    }
}
