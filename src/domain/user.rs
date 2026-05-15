use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Manager,
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SuperAdmin => "superadmin",
            Self::Admin => "admin",
            Self::Manager => "manager",
            Self::User => "user",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: UserRole,
    pub manager_id: Option<Uuid>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}
