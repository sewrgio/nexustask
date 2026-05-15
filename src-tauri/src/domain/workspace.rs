use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub parent_workspace_id: Option<Uuid>,
    pub lft: i64,
    pub rgt: i64,
    pub depth: i32,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
}
