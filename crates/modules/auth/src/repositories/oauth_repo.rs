use crate::models::oauth_identity::{
    ActiveModel as IdentityActiveModel, Column as IdentityColumn, Entity as IdentityEntity,
    Model as IdentityModel,
};
use crate::models::oauth_login_state::{
    ActiveModel as LoginStateActiveModel, Column as LoginStateColumn, Entity as LoginStateEntity,
    Model as LoginStateModel,
};
use crate::repositories::user_repo::{CreateOauthUser, UserRepository};
use sea_orm::*;

pub struct OauthRepository {
    db: DatabaseConnection,
}

pub struct CreateLoginState {
    pub provider: String,
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub return_to: Option<String>,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
}

pub struct ExternalIdentity {
    pub provider: String,
    pub issuer: String,
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub claims: serde_json::Value,
}

impl OauthRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn save_login_state(&self, data: CreateLoginState) -> anyhow::Result<()> {
        let model = LoginStateActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            provider: Set(data.provider),
            state: Set(data.state),
            nonce: Set(data.nonce),
            pkce_verifier: Set(data.pkce_verifier),
            return_to: Set(data.return_to),
            expires_at: Set(data.expires_at),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        model.insert(&self.db).await?;
        Ok(())
    }

    pub async fn take_login_state(
        &self,
        provider: &str,
        state: &str,
    ) -> anyhow::Result<Option<LoginStateModel>> {
        let row = LoginStateEntity::find()
            .filter(LoginStateColumn::Provider.eq(provider))
            .filter(LoginStateColumn::State.eq(state))
            .one(&self.db)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        LoginStateEntity::delete_by_id(row.id)
            .exec(&self.db)
            .await?;

        if row.expires_at <= chrono::Utc::now().fixed_offset() {
            return Ok(None);
        }

        Ok(Some(row))
    }

    pub async fn resolve_or_create_user(
        &self,
        identity: ExternalIdentity,
        require_verified_email: bool,
    ) -> anyhow::Result<uuid::Uuid> {
        if let Some(existing) = self
            .find_identity(&identity.provider, &identity.subject)
            .await?
        {
            let user_id = existing.user_id;
            self.update_identity(existing, &identity).await?;
            return Ok(user_id);
        }

        if require_verified_email && !identity.email_verified {
            anyhow::bail!("SSO provider did not return a verified email address");
        }

        let users = UserRepository::new(self.db.clone());
        let user = if let Some(user) = users.find_by_email(&identity.email).await? {
            user
        } else {
            let should_assign_admin = rwfw_core::auth::user_count(&self.db).await? == 0;
            let user = users
                .create_oauth_user(CreateOauthUser {
                    name: identity.name.clone(),
                    email: identity.email.clone(),
                })
                .await?;

            if should_assign_admin {
                rwfw_core::auth::assign_role(&self.db, user.id, "admin").await?;
            }

            user
        };

        self.create_identity(user.id, &identity).await?;
        Ok(user.id)
    }

    async fn find_identity(
        &self,
        provider: &str,
        subject: &str,
    ) -> anyhow::Result<Option<IdentityModel>> {
        Ok(IdentityEntity::find()
            .filter(IdentityColumn::Provider.eq(provider))
            .filter(IdentityColumn::Subject.eq(subject))
            .one(&self.db)
            .await?)
    }

    async fn create_identity(
        &self,
        user_id: uuid::Uuid,
        identity: &ExternalIdentity,
    ) -> anyhow::Result<()> {
        let now = chrono::Utc::now().fixed_offset();
        IdentityActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(user_id),
            provider: Set(identity.provider.clone()),
            issuer: Set(identity.issuer.clone()),
            subject: Set(identity.subject.clone()),
            email: Set(Some(identity.email.clone())),
            email_verified: Set(identity.email_verified),
            claims: Set(identity.claims.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await?;
        Ok(())
    }

    async fn update_identity(
        &self,
        existing: IdentityModel,
        identity: &ExternalIdentity,
    ) -> anyhow::Result<()> {
        let mut active: IdentityActiveModel = existing.into();
        active.email = Set(Some(identity.email.clone()));
        active.email_verified = Set(identity.email_verified);
        active.claims = Set(identity.claims.clone());
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        active.update(&self.db).await?;
        Ok(())
    }
}
