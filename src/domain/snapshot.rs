use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntitySnapshot {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: String,
    pub snapshot_data: serde_json::Value,
    pub created_by: Uuid,
    pub workspace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl EntitySnapshot {
    pub fn new(
        entity_type: String,
        entity_id: String,
        snapshot_data: serde_json::Value,
        created_by: Uuid,
        workspace_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: 0,
            entity_type,
            entity_id,
            snapshot_data,
            created_by,
            workspace_id,
            created_at: Utc::now(),
        }
    }

    pub fn validate_entity_type(entity_type: &str) -> Result<(), String> {
        if entity_type.is_empty() {
            return Err("Entity type cannot be empty".to_string());
        }
        if !matches!(entity_type, "task" | "comment" | "workspace" | "user") {
            return Err("Invalid entity type".to_string());
        }
        Ok(())
    }

    pub fn get_entity_uuid(&self) -> Result<Uuid, String> {
        Uuid::parse_str(&self.entity_id)
            .map_err(|_| "Invalid entity ID format".to_string())
    }

    pub fn is_task_snapshot(&self) -> bool {
        self.entity_type == "task"
    }

    pub fn is_comment_snapshot(&self) -> bool {
        self.entity_type == "comment"
    }

    pub fn is_workspace_snapshot(&self) -> bool {
        self.entity_type == "workspace"
    }

    pub fn is_user_snapshot(&self) -> bool {
        self.entity_type == "user"
    }
}
