use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Comment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub data: serde_json::Value,
    pub lft: i64,
    pub rgt: i64,
    pub depth: i32,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
