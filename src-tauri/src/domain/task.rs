use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: i32,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub lft: i64,
    pub rgt: i64,
    pub depth: i32,
    pub data: serde_json::Value,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub version: i32,
}

impl Task {
    pub fn new(title: String, created_by: Uuid, workspace_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: 3,
            created_by,
            assigned_to: None,
            parent_id: None,
            lft: 0,
            rgt: 1,
            depth: 0,
            data: serde_json::json!({}),
            workspace_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date: None,
            version: 1,
        }
    }
}
