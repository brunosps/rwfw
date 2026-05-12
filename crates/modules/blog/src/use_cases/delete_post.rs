use crate::repositories::post_repo::PostRepository;
use rwfw_core::error::AppError;

pub struct DeletePostUseCase;

impl DeletePostUseCase {
    pub async fn execute(&self, repo: &PostRepository, post_id: i32) -> Result<(), AppError> {
        repo.delete(post_id).await.map_err(AppError::Internal)?;
        tracing::info!(post_id = %post_id, "Post deleted");
        Ok(())
    }
}
