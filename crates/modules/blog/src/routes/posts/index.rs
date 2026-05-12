use crate::repositories::post_repo::PostRepository;
use crate::use_cases::create_post::{CreatePostInput, CreatePostUseCase};
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

/// GET /blog/posts - List all posts
async fn get(State(state): State<AppState>, i: Inertia) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let (posts, total) = match repo.find_all(1, 20).await {
        Ok(result) => result,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "blog/posts/Index",
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
    .await
}

/// POST /blog/posts - Create a new post
async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Json(input): Json<CreatePostInput>,
) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let use_case = CreatePostUseCase;

    match use_case.execute(&repo, user.id, input).await {
        Ok(output) => Inertia::redirect(&format!("/blog/posts/{}", output.post.id)),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
