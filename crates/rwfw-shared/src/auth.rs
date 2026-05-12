use crate::types::UserId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

#[async_trait::async_trait]
pub trait AuthService: Send + Sync {
    async fn get_user(&self, id: UserId) -> anyhow::Result<AuthUser>;
    async fn verify_session(&self, token: &str) -> anyhow::Result<Option<AuthUser>>;
    async fn has_permission(&self, user_id: UserId, perm: &str) -> anyhow::Result<bool>;
}
