use crate::models::user;
use crate::repositories::user_repo::{CreateUser, UserRepository};
use rwfw_core::error::AppError;
use validator::Validate;

#[derive(Debug, serde::Deserialize, Validate)]
pub struct RegisterUserInput {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: String,
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub password_confirmation: String,
}

pub struct RegisterUserOutput {
    pub user: user::Model,
}

pub struct RegisterUserUseCase;

impl RegisterUserUseCase {
    pub async fn execute(
        &self,
        repo: &UserRepository,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, AppError> {
        // Validate input
        input
            .validate()
            .map_err(rwfw_core::validation::into_app_error)?;

        // Check password confirmation
        if input.password != input.password_confirmation {
            return Err(AppError::Validation(
                [(
                    "password_confirmation".into(),
                    vec!["Passwords do not match".into()],
                )]
                .into_iter()
                .collect(),
            ));
        }

        // Check if email already exists
        if repo
            .find_by_email(&input.email)
            .await
            .map_err(|e| AppError::Internal(e))?
            .is_some()
        {
            return Err(AppError::Validation(
                [("email".into(), vec!["Email already taken".into()])]
                    .into_iter()
                    .collect(),
            ));
        }

        // Create user
        let user = repo
            .create(CreateUser {
                name: input.name,
                email: input.email,
                password: input.password,
            })
            .await
            .map_err(AppError::Internal)?;

        tracing::info!(user_id = %user.id, "User registered");
        Ok(RegisterUserOutput { user })
    }
}
