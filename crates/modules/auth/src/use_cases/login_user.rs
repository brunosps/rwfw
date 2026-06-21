use crate::strategies::{AuthRequest, AuthStrategy};
use rwfw_core::error::AppError;
use validator::Validate;

#[derive(Debug, serde::Deserialize, Validate)]
pub struct LoginUserInput {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
    pub return_to: Option<String>,
}

pub struct LoginUserOutput {
    pub user_id: sea_orm::prelude::Uuid,
    pub name: String,
    pub email: String,
}

pub struct LoginUserUseCase;

impl LoginUserUseCase {
    pub async fn execute(
        &self,
        strategy: &dyn AuthStrategy,
        db: &sea_orm::DatabaseConnection,
        input: LoginUserInput,
    ) -> Result<LoginUserOutput, AppError> {
        input
            .validate()
            .map_err(rwfw_core::validation::into_app_error)?;

        let req = AuthRequest {
            email: input.email,
            password: input.password,
        };

        let result = strategy.authenticate(db, &req).await.map_err(|_| {
            AppError::Validation(
                [("email".into(), vec!["Invalid email or password".into()])]
                    .into_iter()
                    .collect(),
            )
        })?;

        tracing::info!(user_id = %result.user_id, "User logged in");
        Ok(LoginUserOutput {
            user_id: result.user_id,
            name: result.name,
            email: result.email,
        })
    }
}
