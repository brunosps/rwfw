use crate::models::post;
use crate::repositories::post_repo::{PostRepository, UpdatePost};
use rwfw_core::error::AppError;
use validator::Validate;

#[derive(Debug, serde::Deserialize, Validate)]
pub struct UpdatePostInput {
    #[validate(length(min = 1, max = 200, message = "Title is required"))]
    pub title: Option<String>,
    #[validate(length(min = 10, message = "Body must be at least 10 characters"))]
    pub body: Option<String>,
    pub published: Option<bool>,
}

pub struct UpdatePostOutput {
    pub post: post::Model,
}

pub struct UpdatePostUseCase;

impl UpdatePostUseCase {
    pub async fn execute(
        &self,
        repo: &PostRepository,
        post_id: i32,
        input: UpdatePostInput,
    ) -> Result<UpdatePostOutput, AppError> {
        input
            .validate()
            .map_err(rwfw_core::validation::into_app_error)?;

        let post = repo
            .update(
                post_id,
                UpdatePost {
                    title: input.title,
                    body: input.body,
                    published: input.published,
                },
            )
            .await
            .map_err(AppError::Internal)?;

        tracing::info!(post_id = %post.id, "Post updated");
        Ok(UpdatePostOutput { post })
    }
}
