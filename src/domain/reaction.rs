use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CommentReaction {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl CommentReaction {
    pub fn new(comment_id: Uuid, user_id: Uuid, emoji: String, workspace_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            comment_id,
            user_id,
            emoji,
            workspace_id,
            created_at: Utc::now(),
        }
    }

    pub fn validate_emoji(emoji: &str) -> Result<(), String> {
        if emoji.is_empty() {
            return Err("Emoji cannot be empty".to_string());
        }
        if emoji.len() > 10 {
            return Err("Emoji must be less than 10 characters".to_string());
        }
        Ok(())
    }

    pub fn is_valid_emoji(&self) -> bool {
        Self::validate_emoji(&self.emoji).is_ok()
    }
}
