use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Blocked => "blocked",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "in_progress" => Self::InProgress,
            "done" => Self::Done,
            "blocked" => Self::Blocked,
            _ => Self::Pending,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Blocked)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::InProgress)
    }

    pub fn can_transition_to(&self, new_status: &Self) -> bool {
        match (self, new_status) {
            (Self::Pending, Self::InProgress) => true,
            (Self::Pending, Self::Blocked) => true,
            (Self::InProgress, Self::Done) => true,
            (Self::InProgress, Self::Blocked) => true,
            (Self::Blocked, Self::Pending) => true,
            (Self::Blocked, Self::InProgress) => true,
            (Self::Done, Self::InProgress) => true,
            (Self::Done, Self::Pending) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: i32,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub lft: i64,
    pub rgt: i64,
    pub depth: i32,
    pub data: serde_json::Value,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub version: i32,
}

impl Task {
    pub fn new(title: String, created_by: Uuid, workspace_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: 3,
            created_by,
            assigned_to: None,
            parent_id: None,
            lft: 0,
            rgt: 1,
            depth: 0,
            data: serde_json::json!({}),
            workspace_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date: None,
            version: 1,
        }
    }

    pub fn validate_title(title: &str) -> Result<(), String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if title.len() > 255 {
            return Err("Title must be less than 255 characters".to_string());
        }
        Ok(())
    }

    pub fn validate_priority(priority: i32) -> Result<(), String> {
        if priority < 1 || priority > 5 {
            return Err("Priority must be between 1 and 5".to_string());
        }
        Ok(())
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.rgt == self.lft + 1
    }

    pub fn has_children(&self) -> bool {
        self.rgt > self.lft + 1
    }

    pub fn is_descendant_of(&self, potential_parent: &Task) -> bool {
        self.lft > potential_parent.lft && self.rgt < potential_parent.rgt
    }

    pub fn is_ancestor_of(&self, potential_child: &Task) -> bool {
        self.lft < potential_child.lft && self.rgt > potential_child.rgt
    }

    pub fn is_sibling_of(&self, other: &Task) -> bool {
        self.parent_id == other.parent_id && self.id != other.id
    }

    pub fn calculate_progress(&self, children_progress: f64) -> f64 {
        if self.is_leaf() {
            match self.status {
                TaskStatus::Done => 100.0,
                TaskStatus::InProgress => 50.0,
                TaskStatus::Blocked => 0.0,
                TaskStatus::Pending => 0.0,
            }
        } else {
            children_progress
        }
    }

    pub fn can_move_to(&self, new_parent_id: Option<Uuid>, all_tasks: &[Task]) -> Result<(), String> {
        if let Some(new_parent) = new_parent_id {
            if new_parent == self.id {
                return Err("Task cannot be its own parent".to_string());
            }
            
            if let Some(parent_id) = self.parent_id {
                if new_parent == parent_id {
                    return Ok(()); // No change
                }
            }

            for task in all_tasks {
                if task.id == new_parent {
                    if self.is_ancestor_of(task) {
                        return Err("Cannot move task to its own descendant".to_string());
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn set_status(&mut self, new_status: TaskStatus) -> Result<(), String> {
        if !self.status.can_transition_to(&new_status) {
            return Err(format!("Cannot transition from {:?} to {:?}", self.status, new_status));
        }
        self.status = new_status;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_id = Some(parent_id);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn remove_parent(&mut self) {
        self.parent_id = None;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn assign_to(&mut self, user_id: Uuid) {
        self.assigned_to = Some(user_id);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn unassign(&mut self) {
        self.assigned_to = None;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn set_priority(&mut self, priority: i32) -> Result<(), String> {
        Self::validate_priority(priority)?;
        self.priority = priority;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            due_date < Utc::now() && !matches!(self.status, TaskStatus::Done)
        } else {
            false
        }
    }

    pub fn is_due_soon(&self, days: i64) -> bool {
        if let Some(due_date) = self.due_date {
            let now = Utc::now();
            let duration = due_date.signed_duration_since(now);
            duration.num_days() <= days && duration.num_days() >= 0 && !matches!(self.status, TaskStatus::Done)
        } else {
            false
        }
    }

    pub fn can_add_dependency(&self, dependency_id: Uuid, all_tasks: &[Task]) -> Result<(), String> {
        if dependency_id == self.id {
            return Err("Task cannot depend on itself".to_string());
        }

        for task in all_tasks {
            if task.id == dependency_id {
                if self.is_ancestor_of(task) {
                    return Err("Task cannot depend on its own descendant".to_string());
                }
                if task.is_ancestor_of(self) {
                    return Err("Task cannot depend on its own ancestor".to_string());
                }
                break;
            }
        }

        Ok(())
    }

    pub fn validate_dependencies(&self, dependencies: &[Task]) -> Result<(), String> {
        for dep in dependencies {
            if matches!(dep.status, TaskStatus::Blocked) {
                return Err("Cannot complete task with blocked dependencies".to_string());
            }
        }
        Ok(())
    }

    pub fn can_complete(&self, dependencies: &[Task]) -> bool {
        self.validate_dependencies(dependencies).is_ok()
    }
}
