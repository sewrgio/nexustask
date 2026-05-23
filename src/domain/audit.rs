use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventLog {
    pub id: i64,
    pub user_id: Uuid,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub data: serde_json::Value,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub workspace_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl EventLog {
    pub fn new(
        user_id: Uuid,
        event_type: String,
        entity_type: Option<String>,
        entity_id: Option<String>,
        data: serde_json::Value,
        previous_value: Option<String>,
        new_value: Option<String>,
        workspace_id: Option<Uuid>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id,
            event_type,
            entity_type,
            entity_id,
            data,
            previous_value,
            new_value,
            workspace_id,
            ip_address,
            user_agent,
            created_at: Utc::now(),
        }
    }

    pub fn validate_event_type(event_type: &str) -> Result<(), String> {
        if event_type.is_empty() {
            return Err("Event type cannot be empty".to_string());
        }
        if event_type.len() > 100 {
            return Err("Event type must be less than 100 characters".to_string());
        }
        Ok(())
    }

    pub fn is_user_event(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
    }

    pub fn is_workspace_event(&self, workspace_id: Uuid) -> bool {
        self.workspace_id == Some(workspace_id)
    }

    pub fn is_entity_event(&self, entity_type: &str, entity_id: &str) -> bool {
        self.entity_type.as_deref() == Some(entity_type) && self.entity_id.as_deref() == Some(entity_id)
    }

    pub fn get_change_summary(&self) -> Option<String> {
        self.data.get("summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn get_previous_value(&self) -> Option<serde_json::Value> {
        self.data.get("previous").cloned()
    }

    pub fn get_new_value(&self) -> Option<serde_json::Value> {
        self.data.get("new").cloned()
    }

    pub fn has_changes(&self) -> bool {
        self.data.get("previous").is_some() || self.data.get("new").is_some()
    }

    pub fn is_create_event(&self) -> bool {
        self.event_type == "create" || self.event_type.starts_with("create_")
    }

    pub fn is_update_event(&self) -> bool {
        self.event_type == "update" || self.event_type.starts_with("update_")
    }

    pub fn is_delete_event(&self) -> bool {
        self.event_type == "delete" || self.event_type.starts_with("delete_")
    }

    pub fn is_read_event(&self) -> bool {
        self.event_type == "read" || self.event_type.starts_with("read_")
    }
}
