mod domain;
mod application;
mod ports;
mod infrastructure;
mod state;

use crate::infrastructure::ui::app::NexusTaskApp;
use crate::infrastructure::api::start_api_server;
use crate::infrastructure::database::{init_db, sqlite_repo, auth_provider};
use crate::application::{task_service, user_service, workspace_service};
use crate::ports::repository::{TaskRepository, UserRepository, WorkspaceRepository, WorkspaceMemberRepository, CommentRepository, EventLogRepository};
use crate::ports::auth::AuthProvider;
use crate::state::AppState;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    // 1. Initialize Database
    let db_path = "nexustask.db";
    let conn = init_db(db_path).expect("Failed to initialize database");
    let conn_arc = Arc::new(Mutex::new(conn));

    // 2. Initialize Repositories
    let task_repo: Box<dyn TaskRepository> = Box::new(sqlite_repo::SqliteTaskRepository::new(conn_arc.clone()));
    let user_repo: Box<dyn UserRepository> = Box::new(sqlite_repo::SqliteUserRepository::new(conn_arc.clone()));
    let workspace_repo: Box<dyn WorkspaceRepository> = Box::new(sqlite_repo::SqliteWorkspaceRepository::new(conn_arc.clone()));
    let member_repo: Box<dyn WorkspaceMemberRepository> = Box::new(sqlite_repo::SqliteWorkspaceMemberRepository::new(conn_arc.clone()));
    let comment_repo: Box<dyn CommentRepository> = Box::new(sqlite_repo::SqliteCommentRepository::new(conn_arc.clone()));
    let event_log_repo: Box<dyn EventLogRepository> = Box::new(sqlite_repo::SqliteEventLogRepository::new(conn_arc.clone()));

    // 3. Initialize Auth Provider
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let auth_provider: Box<dyn AuthProvider> = Box::new(auth_provider::Argon2AuthProvider::new(jwt_secret.clone()));

    // 4. Initialize Services
    let task_service = Arc::new(task_service::TaskService::new(
        task_repo,
        comment_repo,
        event_log_repo,
    ));

    let user_service = Arc::new(user_service::UserService::new(
        user_repo,
        auth_provider,
    ));

    let workspace_service = Arc::new(workspace_service::WorkspaceService::new(
        workspace_repo,
        member_repo,
    ));

    // 5. Create App State
    let app_state = AppState {
        task_service: task_service.clone(),
        user_service: user_service.clone(),
        workspace_service: workspace_service.clone(),
        jwt_secret,
    };

    // 6. Start API Server in a separate thread
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        start_api_server(app_state_clone).await;
    });

    // 7. Start UI in the main thread
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "NexusTask Enterprise",
        options,
        Box::new(|cc| Box::new(NexusTaskApp::new(cc, app_state))),
    )
}
