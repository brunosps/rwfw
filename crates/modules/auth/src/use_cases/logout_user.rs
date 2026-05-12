use rwfw_core::error::AppError;

pub struct LogoutUserUseCase;

impl LogoutUserUseCase {
    pub async fn execute(&self) -> Result<(), AppError> {
        tracing::info!("User logged out");
        Ok(())
    }
}
