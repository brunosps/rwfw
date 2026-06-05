use crate::repositories::post_repo::PostRepository;
use crate::use_cases::delete_post::DeletePostUseCase;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).delete(delete)
}

/// GET /blog/posts/{id} - Show a post.
async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("blog.posts.view") {
        return AppError::Forbidden("Missing permission: blog.posts.view".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let post = match repo.find_by_id(id).await {
        Ok(Some(post)) => post,
        Ok(None) => return AppError::NotFound(format!("Post not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render("blog/posts/show", serde_json::json!({ "post": post }))
}

/// DELETE /blog/posts/{id} - Delete a post (Turbo `data-turbo-method="delete"`).
async fn delete(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("blog.posts.delete") {
        return AppError::Forbidden("Missing permission: blog.posts.delete".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let use_case = DeletePostUseCase;

    match use_case.execute(&repo, id).await {
        Ok(()) => Redirect::to("/blog/posts").into_response(),
        Err(error) => error.into_response(),
    }
}
