use crate::application::{task_service, user_service, workspace_service};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<task_service::TaskService>,
    pub user_service: Arc<user_service::UserService>,
    pub workspace_service: Arc<workspace_service::WorkspaceService>,
    pub jwt_secret: String,
}
