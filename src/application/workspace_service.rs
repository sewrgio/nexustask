use crate::domain::workspace::{Workspace, WorkspaceMember, WorkspaceRole};
use crate::ports::repository::{WorkspaceRepository, WorkspaceMemberRepository, EventLogRepository};
use uuid::Uuid;
use anyhow::Result;

pub struct WorkspaceService {
    workspace_repo: Box<dyn WorkspaceRepository>,
    member_repo: Box<dyn WorkspaceMemberRepository>,
    event_log_repo: Box<dyn EventLogRepository>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Box<dyn WorkspaceRepository>,
        member_repo: Box<dyn WorkspaceMemberRepository>,
        event_log_repo: Box<dyn EventLogRepository>,
    ) -> Self {
        Self { workspace_repo, member_repo, event_log_repo }
    }

    pub async fn create_workspace(
        &self,
        name: String,
        slug: String,
        description: Option<String>,
        owner_id: Uuid,
    ) -> Result<Workspace> {
        // Check if slug exists
        if self.workspace_repo.find_by_slug(&slug)?.is_some() {
            return Err(anyhow::anyhow!("Workspace slug already exists"));
        }

        let workspace = Workspace {
            id: Uuid::new_v4(),
            name: name.clone(),
            slug,
            description,
            owner_id,
            parent_workspace_id: None,
            lft: 1,
            rgt: 2,
            depth: 0,
            settings: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            is_active: true,
        };

        let workspace_id = self.workspace_repo.create(workspace.clone())?;
        self.log_event(owner_id, "create", "workspace", &workspace_id.to_string(), Some(workspace_id))?;

        // Add owner as member
        let member = WorkspaceMember {
            workspace_id,
            user_id: owner_id,
            role: WorkspaceRole::Owner,
            joined_at: chrono::Utc::now(),
        };
        self.member_repo.add_member(member)?;
        self.log_event(owner_id, "add_member", "workspace", &workspace_id.to_string(), Some(workspace_id))?;

        Ok(workspace)
    }

    pub async fn get_workspace(&self, workspace_id: Uuid) -> Result<Option<Workspace>> {
        self.workspace_repo.find_by_id(workspace_id)
    }

    pub async fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>> {
        self.workspace_repo.get_user_workspaces(user_id)
    }

    pub async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: WorkspaceRole,
        acting_user_id: Uuid,
    ) -> Result<()> {
        let member = WorkspaceMember {
            workspace_id,
            user_id,
            role,
            joined_at: chrono::Utc::now(),
        };
        self.member_repo.add_member(member)?;
        self.log_event(acting_user_id, "add_member", "workspace", &workspace_id.to_string(), Some(workspace_id))?;
        Ok(())
    }

    pub async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid, acting_user_id: Uuid) -> Result<()> {
        self.member_repo.remove_member(workspace_id, user_id)?;
        self.log_event(acting_user_id, "remove_member", "workspace", &workspace_id.to_string(), Some(workspace_id))?;
        Ok(())
    }

    pub async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: &str,
        acting_user_id: Uuid,
    ) -> Result<()> {
        self.member_repo.update_role(workspace_id, user_id, role)?;
        self.log_event(acting_user_id, "update_role", "workspace", &workspace_id.to_string(), Some(workspace_id))?;
        Ok(())
    }

    pub async fn get_members(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceMember>> {
        self.member_repo.get_members(workspace_id)
    }

    pub async fn get_user_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<String>> {
        self.member_repo.get_user_role(workspace_id, user_id)
    }

    fn log_event(
        &self,
        user_id: Uuid,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        workspace_id: Option<Uuid>,
    ) -> Result<()> {
        use crate::domain::audit::EventLog;
        let event = EventLog {
            id: 0,
            user_id,
            event_type: event_type.to_string(),
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id.to_string()),
            data: serde_json::json!({}),
            previous_value: None,
            new_value: None,
            workspace_id,
            created_at: chrono::Utc::now(),
        };
        self.event_log_repo.log(event)?;
        Ok(())
    }
}
