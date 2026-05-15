use crate::domain::task::Task;
use crate::domain::user::User;
use crate::domain::workspace::{Workspace, WorkspaceMember};
use crate::domain::comment::Comment;
use crate::domain::audit::EventLog;
use uuid::Uuid;
use anyhow::Result;

pub trait TaskRepository: Send + Sync {
    fn create(&self, task: Task) -> Result<Uuid>;
    fn update(&self, task: Task) -> Result<()>;
    fn delete(&self, id: Uuid) -> Result<()>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Task>>;
    fn get_tree(&self, root_id: Uuid) -> Result<Vec<Task>>;
    fn get_workspace_tasks(&self, workspace_id: Uuid) -> Result<Vec<Task>>;
    fn move_task(&self, task_id: Uuid, new_parent_id: Option<Uuid>) -> Result<()>;
    fn get_subordinates(&self, user_id: Uuid) -> Result<Vec<User>>;
}

pub trait UserRepository: Send + Sync {
    fn create(&self, user: User) -> Result<Uuid>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    fn update(&self, user: User) -> Result<()>;
    fn update_last_login(&self, user_id: Uuid) -> Result<()>;
}

pub trait WorkspaceRepository: Send + Sync {
    fn create(&self, workspace: Workspace) -> Result<Uuid>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Workspace>>;
    fn find_by_slug(&self, slug: &str) -> Result<Option<Workspace>>;
    fn update(&self, workspace: Workspace) -> Result<()>;
    fn get_user_workspaces(&self, user_id: Uuid) -> Result<Vec<Workspace>>;
}

pub trait WorkspaceMemberRepository: Send + Sync {
    fn add_member(&self, member: WorkspaceMember) -> Result<()>;
    fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<()>;
    fn update_role(&self, workspace_id: Uuid, user_id: Uuid, role: &str) -> Result<()>;
    fn get_members(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceMember>>;
    fn get_user_role(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Option<String>>;
}

pub trait CommentRepository: Send + Sync {
    fn create(&self, comment: Comment) -> Result<Uuid>;
    fn update(&self, comment: Comment) -> Result<()>;
    fn delete(&self, id: Uuid) -> Result<()>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Comment>>;
    fn get_task_comments(&self, task_id: Uuid) -> Result<Vec<Comment>>;
    fn get_thread(&self, root_id: Uuid) -> Result<Vec<Comment>>;
}

pub trait EventLogRepository: Send + Sync {
    fn log(&self, event: EventLog) -> Result<i64>;
    fn get_user_events(&self, user_id: Uuid, limit: i64) -> Result<Vec<EventLog>>;
    fn get_workspace_events(&self, workspace_id: Uuid, limit: i64) -> Result<Vec<EventLog>>;
    fn get_entity_events(&self, entity_type: &str, entity_id: &str, limit: i64) -> Result<Vec<EventLog>>;
}
