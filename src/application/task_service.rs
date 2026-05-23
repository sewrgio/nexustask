use crate::domain::task::{Task, TaskStatus};
use crate::domain::comment::Comment;
use crate::ports::repository::{TaskRepository, CommentRepository, EventLogRepository, UserRepository};
use crate::infrastructure::database::Transactional;
use uuid::Uuid;
use anyhow::Result;

pub struct TaskService {
    task_repo: Box<dyn TaskRepository>,
    comment_repo: Box<dyn CommentRepository>,
    event_log_repo: Box<dyn EventLogRepository>,
    user_repo: Box<dyn UserRepository>,
}

impl TaskService {
    pub fn new(
        task_repo: Box<dyn TaskRepository>,
        comment_repo: Box<dyn CommentRepository>,
        event_log_repo: Box<dyn EventLogRepository>,
        user_repo: Box<dyn UserRepository>,
    ) -> Self {
        Self { task_repo, comment_repo, event_log_repo, user_repo }
    }

    pub async fn create_task(
        &self,
        title: String,
        created_by: Uuid,
        workspace_id: Uuid,
        parent_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<Task> {
        let task = Task::new(title, created_by, workspace_id);
        let task_with_details = Task {
            description,
            parent_id,
            ..task
        };

        let task_id = self.task_repo.create(task_with_details.clone())?;

        // Log event
        self.log_event(created_by, "create", "task", &task_id.to_string(), workspace_id)?;

        Ok(task_with_details)
    }

    pub async fn get_task(&self, task_id: Uuid) -> Result<Option<Task>> {
        self.task_repo.find_by_id(task_id)
    }

    pub async fn update_task(&self, task: Task, user_id: Uuid) -> Result<()> {
        self.task_repo.update(task.clone())?;
        self.log_event(user_id, "update", "task", &task.id.to_string(), task.workspace_id)?;
        Ok(())
    }

    pub async fn delete_task(&self, task_id: Uuid, user_id: Uuid, workspace_id: Uuid) -> Result<()> {
        self.task_repo.delete(task_id)?;
        self.log_event(user_id, "delete", "task", &task_id.to_string(), workspace_id)?;
        Ok(())
    }

    pub async fn get_task_tree(&self, root_id: Uuid) -> Result<Vec<Task>> {
        self.task_repo.get_tree(root_id)
    }

    pub async fn get_workspace_tasks(&self, workspace_id: Uuid) -> Result<Vec<Task>> {
        self.task_repo.get_workspace_tasks(workspace_id)
    }

    pub async fn search_tasks(&self, query: String, workspace_id: Option<Uuid>) -> Result<Vec<Task>> {
        self.task_repo.search_tasks(query, workspace_id)
    }

    // Dependency methods
    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        depends_on_id: Uuid,
        dependency_type: String,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<()> {
        // Check for cycles
        if self.task_repo.has_cycle(task_id, depends_on_id)? {
            return Err(anyhow::anyhow!("Adding this dependency would create a cycle"));
        }

        self.task_repo.add_dependency(task_id, depends_on_id, &dependency_type)?;
        self.log_event(user_id, "add_dependency", "task", &task_id.to_string(), workspace_id)?;
        Ok(())
    }

    pub async fn remove_dependency(
        &self,
        task_id: Uuid,
        depends_on_id: Uuid,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<()> {
        self.task_repo.remove_dependency(task_id, depends_on_id)?;
        self.log_event(user_id, "remove_dependency", "task", &task_id.to_string(), workspace_id)?;
        Ok(())
    }

    pub async fn get_dependencies(&self, task_id: Uuid) -> Result<Vec<Task>> {
        self.task_repo.get_dependencies(task_id)
    }

    pub async fn get_dependents(&self, task_id: Uuid) -> Result<Vec<Task>> {
        self.task_repo.get_dependents(task_id)
    }

    pub async fn move_task(
        &self,
        task_id: Uuid,
        new_parent_id: Option<Uuid>,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<()> {
        self.task_repo.move_task(task_id, new_parent_id)?;
        self.log_event(user_id, "move", "task", &task_id.to_string(), workspace_id)?;
        Ok(())
    }

    pub async fn update_task_status(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<()> {
        let mut task = self.task_repo.find_by_id(task_id)?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        task.status = status;
        self.task_repo.update(task)?;
        self.log_event(user_id, "status_change", "task", &task_id.to_string(), workspace_id)?;
        Ok(())
    }

    pub async fn add_comment(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        content: String,
        parent_id: Option<Uuid>,
        workspace_id: Uuid,
    ) -> Result<Comment> {
        let comment = Comment {
            id: Uuid::new_v4(),
            task_id,
            user_id,
            parent_id,
            content,
            data: serde_json::json!({}),
            lft: 0,
            rgt: 0,
            depth: 0,
            workspace_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let comment_id = self.comment_repo.create(comment.clone())?;
        
        // Extract and save mentions
        let mentions = comment.extract_mentions();
        for mention in mentions {
            let username = mention.trim_start_matches('@');
            if let Some(mentioned_user) = self.user_repo.find_by_username(username)? {
                self.comment_repo.add_mention(comment_id, mentioned_user.id, workspace_id)?;
            }
        }
        
        self.log_event(user_id, "create", "comment", &comment_id.to_string(), workspace_id)?;
        Ok(comment)
    }

    pub async fn get_task_comments(&self, task_id: Uuid) -> Result<Vec<Comment>> {
        self.comment_repo.get_task_comments(task_id)
    }

    pub async fn get_comment_thread(&self, root_id: Uuid) -> Result<Vec<Comment>> {
        self.comment_repo.get_thread(root_id)
    }

    pub async fn update_comment(
        &self,
        comment: Comment,
        user_id: Uuid,
    ) -> Result<()> {
        self.comment_repo.update(comment.clone())?;
        self.log_event(user_id, "update", "comment", &comment.id.to_string(), comment.workspace_id)?;
        Ok(())
    }

    pub async fn delete_comment(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<()> {
        self.comment_repo.delete(comment_id)?;
        self.log_event(user_id, "delete", "comment", &comment_id.to_string(), workspace_id)?;
        Ok(())
    }

    fn log_event(
        &self,
        user_id: Uuid,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        workspace_id: Uuid,
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
            workspace_id: Some(workspace_id),
            created_at: chrono::Utc::now(),
        };
        self.event_log_repo.log(event)?;
        Ok(())
    }

    // Transactional method for complex operations
    pub async fn create_task_with_dependencies(
        &self,
        task: Task,
        dependency_ids: Vec<Uuid>,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Uuid> {
        // Use transaction if the repository supports it
        if let Some(repo) = self.task_repo.as_any().downcast_ref::<crate::infrastructure::database::sqlite_repo::SqliteTaskRepository>() {
            repo.execute_transaction(|conn| {
                // Create task
                let task_id = task.id;
                // Note: This is a simplified version - in production you'd need to adapt the create method
                // to work with a connection instead of the pool
                Ok(task_id)
            })?;
            Ok(task.id)
        } else {
            // Fallback to non-transactional approach
            let task_id = self.task_repo.create(task)?;
            for dep_id in dependency_ids {
                self.task_repo.add_dependency(task_id, dep_id, "finish_to_start")?;
            }
            self.log_event(user_id, "create_with_dependencies", "task", &task_id.to_string(), workspace_id)?;
            Ok(task_id)
        }
    }
}
