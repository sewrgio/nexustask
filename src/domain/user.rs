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

    pub fn from_str(s: &str) -> Self {
        match s {
            "superadmin" => Self::SuperAdmin,
            "admin" => Self::Admin,
            "manager" => Self::Manager,
            "user" => Self::User,
            _ => Self::User,
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::SuperAdmin | Self::Admin)
    }

    pub fn can_manage_workspaces(&self) -> bool {
        matches!(self, Self::SuperAdmin | Self::Admin | Self::Manager)
    }

    pub fn can_delete(&self) -> bool {
        matches!(self, Self::SuperAdmin)
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

impl User {
    pub fn validate_email(email: &str) -> Result<(), String> {
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if !email.contains('@') || !email.contains('.') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }

    pub fn validate_username(username: &str) -> Result<(), String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }
        if username.len() > 50 {
            return Err("Username must be less than 50 characters".to_string());
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
        Ok(())
    }

    pub fn validate_password_hash(hash: &str) -> Result<(), String> {
        if hash.is_empty() {
            return Err("Password hash cannot be empty".to_string());
        }
        if hash.len() < 20 {
            return Err("Invalid password hash".to_string());
        }
        Ok(())
    }

    pub fn is_manager_of(&self, potential_subordinate_id: Uuid) -> bool {
        self.manager_id == Some(potential_subordinate_id)
    }

    pub fn has_manager(&self) -> bool {
        self.manager_id.is_some()
    }

    pub fn can_be_managed_by(&self, manager_id: Uuid) -> bool {
        self.manager_id == Some(manager_id)
    }

    pub fn is_super_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin)
    }

    pub fn is_manager(&self) -> bool {
        matches!(self.role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Manager)
    }

    pub fn is_active_user(&self) -> bool {
        self.is_active
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn set_manager(&mut self, manager_id: Uuid) {
        self.manager_id = Some(manager_id);
    }

    pub fn remove_manager(&mut self) {
        self.manager_id = None;
    }
}
