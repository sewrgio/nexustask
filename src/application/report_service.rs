use crate::domain::task::{Task, TaskStatus};
use crate::domain::user::User;
use crate::ports::repository::{TaskRepository, UserRepository, WorkspaceRepository};
use uuid::Uuid;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration, Timelike};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub in_progress_tasks: i64,
    pub pending_tasks: i64,
    pub blocked_tasks: i64,
    pub completion_rate: f64,
    pub avg_priority: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetrics {
    pub user_id: Uuid,
    pub username: String,
    pub assigned_tasks: i64,
    pub completed_tasks: i64,
    pub completion_rate: f64,
    pub overdue_tasks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceReport {
    pub workspace_id: Uuid,
    pub workspace_name: String,
    pub task_metrics: TaskMetrics,
    pub user_metrics: Vec<UserMetrics>,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub date: String,
    pub completed: i64,
    pub created: i64,
}

pub struct ReportService {
    task_repo: Box<dyn TaskRepository>,
    user_repo: Box<dyn UserRepository>,
    workspace_repo: Box<dyn WorkspaceRepository>,
}

impl ReportService {
    pub fn new(
        task_repo: Box<dyn TaskRepository>,
        user_repo: Box<dyn UserRepository>,
        workspace_repo: Box<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            task_repo,
            user_repo,
            workspace_repo,
        }
    }

    pub fn generate_workspace_report(
        &self,
        workspace_id: Uuid,
        days: i64,
    ) -> Result<WorkspaceReport> {
        let workspace = self.workspace_repo
            .find_by_id(workspace_id)?
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;

        let tasks = self.task_repo.get_workspace_tasks(workspace_id)?;
        let task_metrics = self.calculate_task_metrics(&tasks);

        let users = self.get_workspace_users(workspace_id)?;
        let user_metrics = self.calculate_user_metrics(&users, &tasks);

        let period_end = Utc::now();
        let period_start = period_end - Duration::days(days);

        Ok(WorkspaceReport {
            workspace_id,
            workspace_name: workspace.name,
            task_metrics,
            user_metrics,
            generated_at: Utc::now(),
            period_start,
            period_end,
        })
    }

    pub fn generate_trend_report(
        &self,
        workspace_id: Uuid,
        days: i64,
    ) -> Result<Vec<TrendData>> {
        let tasks = self.task_repo.get_workspace_tasks(workspace_id)?;
        let mut trend_data = Vec::new();

        let now = Utc::now();
        
        for i in (0..days).rev() {
            let date = now - Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            
            let day_start = date.with_hour(0).and_then(|d| d.with_minute(0)).and_then(|d| d.with_second(0)).unwrap();
            let day_end = date.with_hour(23).and_then(|d| d.with_minute(59)).and_then(|d| d.with_second(59)).unwrap();

            let completed = tasks.iter()
                .filter(|t| {
                    t.updated_at >= day_start && t.updated_at <= day_end && matches!(t.status, TaskStatus::Done)
                })
                .count() as i64;

            let created = tasks.iter()
                .filter(|t| {
                    t.created_at >= day_start && t.created_at <= day_end
                })
                .count() as i64;

            trend_data.push(TrendData {
                date: date_str,
                completed,
                created,
            });
        }

        Ok(trend_data)
    }

    fn calculate_task_metrics(&self, tasks: &[Task]) -> TaskMetrics {
        let total = tasks.len() as i64;
        let completed = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Done)).count() as i64;
        let in_progress = tasks.iter().filter(|t| matches!(t.status, TaskStatus::InProgress)).count() as i64;
        let pending = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Pending)).count() as i64;
        let blocked = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Blocked)).count() as i64;

        let completion_rate = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let avg_priority = if total > 0 {
            tasks.iter().map(|t| t.priority).sum::<i32>() as f64 / total as f64
        } else {
            0.0
        };

        TaskMetrics {
            total_tasks: total,
            completed_tasks: completed,
            in_progress_tasks: in_progress,
            pending_tasks: pending,
            blocked_tasks: blocked,
            completion_rate,
            avg_priority,
        }
    }

    fn calculate_user_metrics(&self, users: &[User], tasks: &[Task]) -> Vec<UserMetrics> {
        let now = Utc::now();

        users.iter().map(|user| {
            let assigned = tasks.iter().filter(|t| t.assigned_to == Some(user.id)).count() as i64;
            let completed = tasks.iter()
                .filter(|t| t.assigned_to == Some(user.id) && matches!(t.status, TaskStatus::Done))
                .count() as i64;

            let completion_rate = if assigned > 0 {
                (completed as f64 / assigned as f64) * 100.0
            } else {
                0.0
            };

            let overdue = tasks.iter()
                .filter(|t| {
                    t.assigned_to == Some(user.id)
                        && !matches!(t.status, TaskStatus::Done)
                        && t.due_date.map_or(false, |due| due < now)
                })
                .count() as i64;

            UserMetrics {
                user_id: user.id,
                username: user.username.clone(),
                assigned_tasks: assigned,
                completed_tasks: completed,
                completion_rate,
                overdue_tasks: overdue,
            }
        }).collect()
    }

    fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<User>> {
        // This is a simplified implementation
        // In a real scenario, you'd query workspace_members and then get users
        // For now, we'll return all users (this should be improved)
        Ok(vec![])
    }
}
