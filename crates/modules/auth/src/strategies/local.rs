use super::{AuthRequest, AuthResult, AuthStrategy};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub struct LocalAuthStrategy;

impl LocalAuthStrategy {
    pub fn new() -> Self {
        Self
    }

    pub fn hash_password(password: &str) -> anyhow::Result<String> {
        rwfw_core::auth::hash_password(password)
    }

    pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
        rwfw_core::auth::verify_password(password, hash)
    }
}

#[async_trait::async_trait]
impl AuthStrategy for LocalAuthStrategy {
    fn name(&self) -> &str {
        "local"
    }

    async fn authenticate(
        &self,
        db: &sea_orm::DatabaseConnection,
        req: &AuthRequest,
    ) -> anyhow::Result<AuthResult> {
        use crate::models::user::{Column, Entity as UserEntity};

        let user = UserEntity::find()
            .filter(Column::Email.eq(&req.email))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid email or password"))?;

        let hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("User does not have a password (OAuth only)"))?;

        if !Self::verify_password(&req.password, hash)? {
            anyhow::bail!("Invalid email or password");
        }

        Ok(AuthResult {
            user_id: user.id,
            email: user.email,
            name: user.name,
        })
    }
}
