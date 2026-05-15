use crate::ports::repository::TaskRepository;
use crate::domain::task::Task;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;

pub struct RecurringTaskService {
    task_repo: Box<dyn TaskRepository>,
}

impl RecurringTaskService {
    pub fn new(task_repo: Box<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    pub async fn create_recurring_task(
        &self,
        title: String,
        created_by: Uuid,
        workspace_id: Uuid,
        cron_expression: String,
        description: Option<String>,
    ) -> Result<Task> {
        // Validate cron expression
        Schedule::from_str(&cron_expression)?;

        let mut task = Task::new(title, created_by, workspace_id);
        task.description = description;
        
        // Store cron expression in data JSON
        task.data = serde_json::json!({
            "recurring": true,
            "cron_expression": cron_expression,
            "next_run": Self::calculate_next_run(&cron_expression)?
        });

        let task_id = self.task_repo.create(task)?;
        self.task_repo.find_by_id(task_id)?.ok_or_else(|| anyhow::anyhow!("Failed to create task"))
    }

    pub async fn process_due_recurring_tasks(&self) -> Result<Vec<Task>> {
        // This would be called periodically by a background job
        // For now, return empty
        Ok(vec![])
    }

    fn calculate_next_run(cron_expression: &str) -> Result<String> {
        let schedule = Schedule::from_str(cron_expression)?;
        let next_run = schedule.upcoming(Utc).next()
            .ok_or_else(|| anyhow::anyhow!("No next run time"))?;
        Ok(next_run.to_rfc3339())
    }

    pub async fn update_next_run(&self, task_id: Uuid) -> Result<()> {
        let mut task = self.task_repo.find_by_id(task_id)?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        if let Some(cron_expr) = task.data.get("cron_expression").and_then(|v| v.as_str()) {
            let next_run = Self::calculate_next_run(cron_expr)?;
            task.data["next_run"] = serde_json::Value::String(next_run);
            self.task_repo.update(task)?;
        }

        Ok(())
    }
}
