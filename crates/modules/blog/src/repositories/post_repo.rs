use crate::models::post::{self, ActiveModel, Column, Entity as PostEntity};
use sea_orm::*;
use serde::Deserialize;

pub struct PostRepository {
    db: DatabaseConnection,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub body: String,
    pub author_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub body: Option<String>,
    pub published: Option<bool>,
}

impl PostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<post::Model>> {
        Ok(PostEntity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn find_all(
        &self,
        page: u64,
        per_page: u64,
    ) -> anyhow::Result<(Vec<post::Model>, u64)> {
        let paginator = PostEntity::find()
            .order_by_desc(Column::CreatedAt)
            .paginate(&self.db, per_page);

        let total = paginator.num_items().await?;
        let posts = paginator.fetch_page(page.saturating_sub(1)).await?;
        Ok((posts, total))
    }

    pub async fn create(&self, data: CreatePost) -> anyhow::Result<post::Model> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            title: Set(data.title),
            body: Set(data.body),
            author_id: Set(data.author_id),
            published: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn update(&self, id: i32, data: UpdatePost) -> anyhow::Result<post::Model> {
        let post = PostEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Post not found"))?;

        let mut active: ActiveModel = post.into();
        if let Some(title) = data.title {
            active.title = Set(title);
        }
        if let Some(body) = data.body {
            active.body = Set(body);
        }
        if let Some(published) = data.published {
            active.published = Set(published);
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        PostEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
