use crate::repositories::post_repo::PostRepository;
use crate::use_cases::update_post::{UpdatePostInput, UpdatePostUseCase};
use axum::extract::Path;
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).put(put)
}

/// GET /blog/posts/{id}/edit - Edit post form
async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("blog.posts.update") {
        return AppError::Forbidden("Missing permission: blog.posts.update".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let post = match repo.find_by_id(id).await {
        Ok(Some(post)) => post,
        Ok(None) => return AppError::NotFound(format!("Post not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "blog/posts/Edit",
        serde_json::json!({
            "post": post
        }),
    )
    .await
}

/// PUT /blog/posts/{id}/edit - Update a post
async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
    Json(input): Json<UpdatePostInput>,
) -> Response {
    if !user.can("blog.posts.update") {
        return AppError::Forbidden("Missing permission: blog.posts.update".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let use_case = UpdatePostUseCase;

    match use_case.execute(&repo, id, input).await {
        Ok(output) => Inertia::redirect(&format!("/blog/posts/{}", output.post.id)),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
