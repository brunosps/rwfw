use crate::repositories::post_repo::PostRepository;
use crate::use_cases::update_post::{UpdatePostInput, UpdatePostUseCase};
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(put)
}

/// GET /blog/posts/{id}/edit - Edit post form.
async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
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

    v.render("blog/posts/edit", serde_json::json!({ "post": post }))
}

/// POST /blog/posts/{id}/edit - Update a post (HTML forms can't issue PUT).
async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
    Form(input): Form<UpdatePostInput>,
) -> Response {
    if !user.can("blog.posts.update") {
        return AppError::Forbidden("Missing permission: blog.posts.update".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let use_case = UpdatePostUseCase;

    let title = input.title.clone();
    let body = input.body.clone();

    match use_case.execute(&repo, id, input).await {
        Ok(output) => Redirect::to(&format!("/blog/posts/{}", output.post.id)).into_response(),
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blog/posts/edit",
            serde_json::json!({
                "post": { "id": id, "title": title, "body": body },
                "errors": rwfw_core::validation::first_messages(errors)
            }),
        ),
        Err(error) => error.into_response(),
    }
}
