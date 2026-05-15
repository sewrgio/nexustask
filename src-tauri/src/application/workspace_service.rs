use crate::domain::workspace::{Workspace, WorkspaceMember, WorkspaceRole};
use crate::ports::repository::{WorkspaceRepository, WorkspaceMemberRepository};
use uuid::Uuid;
use anyhow::Result;

pub struct WorkspaceService {
    workspace_repo: Box<dyn WorkspaceRepository>,
    member_repo: Box<dyn WorkspaceMemberRepository>,
}

impl WorkspaceService {
    pub fn new(
        workspace_repo: Box<dyn WorkspaceRepository>,
        member_repo: Box<dyn WorkspaceMemberRepository>,
    ) -> Self {
        Self { workspace_repo, member_repo }
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

        // Add owner as member
        let member = WorkspaceMember {
            workspace_id,
            user_id: owner_id,
            role: WorkspaceRole::Owner,
            joined_at: chrono::Utc::now(),
        };
        self.member_repo.add_member(member)?;

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
    ) -> Result<()> {
        let member = WorkspaceMember {
            workspace_id,
            user_id,
            role,
            joined_at: chrono::Utc::now(),
        };
        self.member_repo.add_member(member)
    }

    pub async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<()> {
        self.member_repo.remove_member(workspace_id, user_id)
    }

    pub async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<()> {
        self.member_repo.update_role(workspace_id, user_id, role)
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
}
