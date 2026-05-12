use crate::models::post;
use crate::repositories::post_repo::{CreatePost, PostRepository};
use rwfw_core::error::AppError;
use validator::Validate;

#[derive(Debug, serde::Deserialize, Validate)]
pub struct CreatePostInput {
    #[validate(length(min = 1, max = 200, message = "Title is required"))]
    pub title: String,
    #[validate(length(min = 10, message = "Body must be at least 10 characters"))]
    pub body: String,
}

pub struct CreatePostOutput {
    pub post: post::Model,
}

pub struct CreatePostUseCase;

impl CreatePostUseCase {
    pub async fn execute(
        &self,
        repo: &PostRepository,
        author_id: uuid::Uuid,
        input: CreatePostInput,
    ) -> Result<CreatePostOutput, AppError> {
        input
            .validate()
            .map_err(rwfw_core::validation::into_app_error)?;

        let post = repo
            .create(CreatePost {
                title: input.title,
                body: input.body,
                author_id,
            })
            .await
            .map_err(AppError::Internal)?;

        tracing::info!(post_id = %post.id, "Post created");
        Ok(CreatePostOutput { post })
    }
}
