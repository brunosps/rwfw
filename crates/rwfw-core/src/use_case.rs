use crate::error::AppError;

#[async_trait::async_trait]
pub trait UseCase<I: Send, O: Send>: Send + Sync {
    async fn execute(&self, input: I) -> Result<O, AppError>;
}
