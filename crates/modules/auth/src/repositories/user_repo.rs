use crate::models::user::{self, ActiveModel, Column, Entity as UserEntity};
use crate::strategies::local::LocalAuthStrategy;
use sea_orm::*;
use serde::Deserialize;

pub struct UserRepository {
    db: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOauthUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> anyhow::Result<Option<user::Model>> {
        Ok(UserEntity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<user::Model>> {
        Ok(UserEntity::find()
            .filter(Column::Email.eq(email))
            .one(&self.db)
            .await?)
    }

    pub async fn create(&self, data: CreateUser) -> anyhow::Result<user::Model> {
        let password_hash = LocalAuthStrategy::hash_password(&data.password)?;
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            name: Set(data.name),
            email: Set(data.email),
            password_hash: Set(Some(password_hash)),
            created_at: Set(now),
            updated_at: Set(now),
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn create_oauth_user(&self, data: CreateOauthUser) -> anyhow::Result<user::Model> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            name: Set(data.name),
            email: Set(data.email),
            password_hash: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn update(&self, id: uuid::Uuid, data: UpdateUser) -> anyhow::Result<user::Model> {
        let user = UserEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let mut active: ActiveModel = user.into();
        if let Some(name) = data.name {
            active.name = Set(name);
        }
        if let Some(email) = data.email {
            active.email = Set(email);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        Ok(active.update(&self.db).await?)
    }
}
