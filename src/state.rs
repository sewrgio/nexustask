use crate::application::{task_service, user_service, workspace_service, custom_field_service, report_service, notification_service, backup_service};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<task_service::TaskService>,
    pub user_service: Arc<user_service::UserService>,
    pub workspace_service: Arc<workspace_service::WorkspaceService>,
    pub custom_field_service: Arc<custom_field_service::CustomFieldService>,
    pub report_service: Arc<report_service::ReportService>,
    pub notification_service: Arc<notification_service::NotificationService>,
    pub backup_service: Arc<backup_service::BackupService>,
    pub jwt_secret: String,
}
