use crate::error::AppError;

#[async_trait::async_trait]
pub trait Repository<E: Send, C: Send, U: Send>: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<E>, AppError>;
    async fn create(&self, data: C) -> Result<E, AppError>;
    async fn update(&self, id: i32, data: U) -> Result<E, AppError>;
    async fn delete(&self, id: i32) -> Result<(), AppError>;
}
