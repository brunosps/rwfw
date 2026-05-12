use crate::types::{PostId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    UserCreated { user_id: UserId, email: String },
    UserLoggedIn { user_id: UserId },
    PostPublished { post_id: PostId, author_id: UserId },
    PostDeleted { post_id: PostId },
}
