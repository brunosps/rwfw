pub mod local;
pub mod oidc;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub struct AuthResult {
    pub user_id: sea_orm::prelude::Uuid,
    pub email: String,
    pub name: String,
}

#[async_trait::async_trait]
pub trait AuthStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn login_url(&self) -> Option<String> {
        None
    }
    async fn authenticate(
        &self,
        db: &sea_orm::DatabaseConnection,
        req: &AuthRequest,
    ) -> anyhow::Result<AuthResult>;
}
