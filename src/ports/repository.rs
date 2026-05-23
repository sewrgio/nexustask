use crate::domain::task::Task;
use crate::domain::user::User;
use crate::domain::workspace::{Workspace, WorkspaceMember};
use crate::domain::comment::Comment;
use crate::domain::audit::EventLog;
use crate::domain::reaction::CommentReaction;
use crate::domain::attachment::FileAttachment;
use crate::domain::snapshot::EntitySnapshot;
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
    fn search_tasks(&self, query: String, workspace_id: Option<Uuid>) -> Result<Vec<Task>>;
    
    // Dependency methods
    fn add_dependency(&self, task_id: Uuid, depends_on_id: Uuid, dependency_type: &str) -> Result<()>;
    fn remove_dependency(&self, task_id: Uuid, depends_on_id: Uuid) -> Result<()>;
    fn get_dependencies(&self, task_id: Uuid) -> Result<Vec<Task>>;
    fn get_dependents(&self, task_id: Uuid) -> Result<Vec<Task>>;
    fn has_cycle(&self, task_id: Uuid, depends_on_id: Uuid) -> Result<bool>;
    
    // For downcasting to concrete types
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait UserRepository: Send + Sync {
    fn create(&self, user: User) -> Result<Uuid>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    fn update(&self, user: User) -> Result<()>;
    fn update_last_login(&self, user_id: Uuid) -> Result<()>;
    fn get_workload(&self, user_id: Uuid) -> Result<i32>;
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
    
    // Mention methods
    fn add_mention(&self, comment_id: Uuid, mentioned_user_id: Uuid, workspace_id: Uuid) -> Result<()>;
    fn get_user_mentions(&self, user_id: Uuid) -> Result<Vec<Uuid>>;
    fn mark_mention_read(&self, comment_id: Uuid, user_id: Uuid) -> Result<()>;
}

pub trait EventLogRepository: Send + Sync {
    fn log(&self, event: EventLog) -> Result<i64>;
    fn get_user_events(&self, user_id: Uuid, limit: i64) -> Result<Vec<EventLog>>;
    fn get_workspace_events(&self, workspace_id: Uuid, limit: i64) -> Result<Vec<EventLog>>;
    fn get_entity_events(&self, entity_type: &str, entity_id: &str, limit: i64) -> Result<Vec<EventLog>>;
}

pub trait CustomFieldRepository: Send + Sync {
    fn create(&self, workspace_id: Uuid, field_name: &str, field_type: &str, field_options: Option<&str>, is_required: bool) -> Result<Uuid>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<crate::domain::custom_field::CustomField>>;
    fn get_workspace_fields(&self, workspace_id: Uuid) -> Result<Vec<crate::domain::custom_field::CustomField>>;
    fn update(&self, id: Uuid, field_name: &str, field_options: Option<&str>, is_required: bool) -> Result<()>;
    fn delete(&self, id: Uuid) -> Result<()>;
}

pub trait ReactionRepository: Send + Sync {
    fn add_reaction(&self, reaction: CommentReaction) -> Result<Uuid>;
    fn remove_reaction(&self, comment_id: Uuid, user_id: Uuid, emoji: &str) -> Result<()>;
    fn get_comment_reactions(&self, comment_id: Uuid) -> Result<Vec<CommentReaction>>;
    fn get_user_reactions(&self, user_id: Uuid) -> Result<Vec<CommentReaction>>;
}

pub trait AttachmentRepository: Send + Sync {
    fn create_attachment(&self, attachment: FileAttachment) -> Result<Uuid>;
    fn delete_attachment(&self, id: Uuid) -> Result<()>;
    fn get_entity_attachments(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<FileAttachment>>;
    fn get_user_attachments(&self, user_id: Uuid) -> Result<Vec<FileAttachment>>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<FileAttachment>>;
}

pub trait SnapshotRepository: Send + Sync {
    fn create_snapshot(&self, snapshot: EntitySnapshot) -> Result<i64>;
    fn get_entity_snapshots(&self, entity_type: &str, entity_id: &str, limit: i64) -> Result<Vec<EntitySnapshot>>;
    fn get_latest_snapshot(&self, entity_type: &str, entity_id: &str) -> Result<Option<EntitySnapshot>>;
    fn delete_old_snapshots(&self, days: i32) -> Result<i64>;
}
