use crate::ports::repository::{TaskRepository, WorkspaceRepository, UserRepository};
use anyhow::Result;
use uuid::Uuid;
use serde_json;

pub struct ExportService {
    task_repo: Box<dyn TaskRepository>,
    workspace_repo: Box<dyn WorkspaceRepository>,
    user_repo: Box<dyn UserRepository>,
}

impl ExportService {
    pub fn new(
        task_repo: Box<dyn TaskRepository>,
        workspace_repo: Box<dyn WorkspaceRepository>,
        user_repo: Box<dyn UserRepository>,
    ) -> Self {
        Self { task_repo, workspace_repo, user_repo }
    }

    pub async fn export_workspace_json(&self, workspace_id: Uuid) -> Result<serde_json::Value> {
        let workspace = self.workspace_repo.find_by_id(workspace_id)?
            .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;

        let tasks = self.task_repo.get_workspace_tasks(workspace_id)?;

        let export = serde_json::json!({
            "workspace": workspace,
            "tasks": tasks,
            "exported_at": chrono::Utc::now(),
        });

        Ok(export)
    }

    pub async fn export_workspace_csv(&self, workspace_id: Uuid) -> Result<String> {
        let tasks = self.task_repo.get_workspace_tasks(workspace_id)?;

        let mut csv = String::new();
        csv.push_str("id,title,description,status,priority,created_by,assigned_to,parent_id,created_at,updated_at,due_date\n");

        for task in tasks {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                task.id,
                escape_csv(&task.title),
                escape_csv(&task.description.unwrap_or_default()),
                task.status.as_str(),
                task.priority,
                task.created_by,
                task.assigned_to.map(|u| u.to_string()).unwrap_or_default(),
                task.parent_id.map(|u| u.to_string()).unwrap_or_default(),
                task.created_at,
                task.updated_at,
                task.due_date.map(|d| d.to_string()).unwrap_or_default(),
            ));
        }

        Ok(csv)
    }

    pub async fn import_workspace_json(&self, _json_data: serde_json::Value) -> Result<Uuid> {
        // This would need to be implemented to handle import
        // For now, return placeholder
        Ok(Uuid::new_v4())
    }

    pub async fn export_workspace_pdf(&self, _workspace_id: Uuid) -> Result<Vec<u8>> {
        // PDF export requires additional dependencies (genpdf or similar)
        // For now, return an error indicating this feature is not yet implemented
        Err(anyhow::anyhow!("PDF export not yet implemented - requires PDF generation library"))
    }
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace("\"", "\"\""))
    } else {
        value.to_string()
    }
}
