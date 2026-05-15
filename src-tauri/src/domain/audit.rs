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
    pub workspace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
