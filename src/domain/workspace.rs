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

impl Workspace {
    pub fn new(name: String, slug: String, owner_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            slug,
            description: None,
            owner_id,
            parent_workspace_id: None,
            lft: 0,
            rgt: 1,
            depth: 0,
            settings: serde_json::json!({}),
            created_at: Utc::now(),
            is_active: true,
        }
    }

    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Workspace name cannot be empty".to_string());
        }
        if name.len() > 100 {
            return Err("Workspace name must be less than 100 characters".to_string());
        }
        Ok(())
    }

    pub fn validate_slug(slug: &str) -> Result<(), String> {
        if slug.is_empty() {
            return Err("Workspace slug cannot be empty".to_string());
        }
        if slug.len() > 50 {
            return Err("Workspace slug must be less than 50 characters".to_string());
        }
        if !slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Workspace slug can only contain alphanumeric characters, hyphens, and underscores".to_string());
        }
        Ok(())
    }

    pub fn is_root(&self) -> bool {
        self.parent_workspace_id.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.rgt == self.lft + 1
    }

    pub fn has_children(&self) -> bool {
        self.rgt > self.lft + 1
    }

    pub fn is_descendant_of(&self, potential_parent: &Workspace) -> bool {
        self.lft > potential_parent.lft && self.rgt < potential_parent.rgt
    }

    pub fn is_ancestor_of(&self, potential_child: &Workspace) -> bool {
        self.lft < potential_child.lft && self.rgt > potential_child.rgt
    }

    pub fn is_sibling_of(&self, other: &Workspace) -> bool {
        self.parent_workspace_id == other.parent_workspace_id && self.id != other.id
    }

    pub fn can_move_to(&self, new_parent_id: Option<Uuid>, all_workspaces: &[Workspace]) -> Result<(), String> {
        if let Some(new_parent) = new_parent_id {
            if new_parent == self.id {
                return Err("Workspace cannot be its own parent".to_string());
            }

            if let Some(parent_id) = self.parent_workspace_id {
                if new_parent == parent_id {
                    return Ok(()); // No change
                }
            }

            for workspace in all_workspaces {
                if workspace.id == new_parent {
                    if self.is_ancestor_of(workspace) {
                        return Err("Cannot move workspace to its own descendant".to_string());
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_workspace_id = Some(parent_id);
    }

    pub fn remove_parent(&mut self) {
        self.parent_workspace_id = None;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn is_owned_by(&self, user_id: Uuid) -> bool {
        self.owner_id == user_id
    }

    pub fn get_setting<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.settings.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_setting(&mut self, key: &str, value: serde_json::Value) {
        self.settings[key] = value;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl WorkspaceRole {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "member" => Self::Member,
            "viewer" => Self::Viewer,
            _ => Self::Viewer,
        }
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_manage_tasks(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }

    pub fn can_delete_workspace(&self) -> bool {
        matches!(self, Self::Owner)
    }

    pub fn can_view_tasks(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member | Self::Viewer)
    }

    pub fn can_change_settings(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_promote_to(&self, target_role: &WorkspaceRole) -> bool {
        match (self, target_role) {
            (Self::Owner, _) => true,
            (Self::Admin, WorkspaceRole::Member | WorkspaceRole::Viewer) => true,
            (Self::Admin, WorkspaceRole::Admin | WorkspaceRole::Owner) => false,
            (Self::Member, _) => false,
            (Self::Viewer, _) => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: WorkspaceRole,
    pub joined_at: DateTime<Utc>,
}

impl WorkspaceMember {
    pub fn new(workspace_id: Uuid, user_id: Uuid, role: WorkspaceRole) -> Self {
        Self {
            workspace_id,
            user_id,
            role,
            joined_at: Utc::now(),
        }
    }

    pub fn can_perform_action(&self, action: &str) -> bool {
        match action {
            "manage_members" => self.role.can_manage_members(),
            "manage_tasks" => self.role.can_manage_tasks(),
            "delete_workspace" => self.role.can_delete_workspace(),
            "view_tasks" => self.role.can_view_tasks(),
            "change_settings" => self.role.can_change_settings(),
            _ => false,
        }
    }

    pub fn promote(&mut self, new_role: WorkspaceRole) -> Result<(), String> {
        if !self.role.can_promote_to(&new_role) {
            return Err(format!("Cannot promote from {:?} to {:?}", self.role, new_role));
        }
        self.role = new_role;
        Ok(())
    }

    pub fn demote(&mut self, new_role: WorkspaceRole) -> Result<(), String> {
        if matches!(self.role, WorkspaceRole::Owner) && new_role != WorkspaceRole::Owner {
            return Err("Cannot demote owner".to_string());
        }
        self.role = new_role;
        Ok(())
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.role, WorkspaceRole::Owner)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, WorkspaceRole::Owner | WorkspaceRole::Admin)
    }

    pub fn is_member(&self) -> bool {
        matches!(self.role, WorkspaceRole::Owner | WorkspaceRole::Admin | WorkspaceRole::Member)
    }
}
