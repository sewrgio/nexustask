mod domain;
mod application;
mod ports;
mod infrastructure;
mod state;

use crate::infrastructure::api::start_api_server;
use crate::infrastructure::database::{init_db_pool, sqlite_repo, auth_provider};
use crate::infrastructure::api::middleware::RateLimiter;
use crate::application::{task_service, user_service, workspace_service, custom_field_service, report_service, notification_service, backup_service};
use crate::ports::repository::{TaskRepository, UserRepository, WorkspaceRepository, WorkspaceMemberRepository, CommentRepository, EventLogRepository, CustomFieldRepository};
use crate::ports::auth::AuthProvider;
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // 1. Initialize Database with Connection Pool
    let db_path = "nexustask.db";
    let pool = init_db_pool(db_path, 10).expect("Failed to initialize database pool");

    // 2. Initialize Repositories
    let task_repo: Box<dyn TaskRepository> = Box::new(sqlite_repo::SqliteTaskRepository::new(pool.clone()));
    let user_repo: Box<dyn UserRepository> = Box::new(sqlite_repo::SqliteUserRepository::new(pool.clone()));
    let workspace_repo: Box<dyn WorkspaceRepository> = Box::new(sqlite_repo::SqliteWorkspaceRepository::new(pool.clone()));
    let member_repo: Box<dyn WorkspaceMemberRepository> = Box::new(sqlite_repo::SqliteWorkspaceMemberRepository::new(pool.clone()));
    let comment_repo: Box<dyn CommentRepository> = Box::new(sqlite_repo::SqliteCommentRepository::new(pool.clone()));
    let event_log_repo: Box<dyn EventLogRepository> = Box::new(sqlite_repo::SqliteEventLogRepository::new(pool.clone()));
    let custom_field_repo: Box<dyn CustomFieldRepository> = Box::new(sqlite_repo::SqliteCustomFieldRepository::new(pool.clone()));

    // 3. Initialize Auth Provider
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let auth_provider: Box<dyn AuthProvider> = Box::new(auth_provider::Argon2AuthProvider::new(jwt_secret.clone()));

    // 4. Initialize Services
    let task_service = Arc::new(task_service::TaskService::new(
        task_repo,
        comment_repo,
        event_log_repo,
        Box::new(sqlite_repo::SqliteUserRepository::new(pool.clone())),
    ));

    let user_service = Arc::new(user_service::UserService::new(
        user_repo,
        auth_provider,
        Box::new(sqlite_repo::SqliteEventLogRepository::new(pool.clone())),
    ));

    let workspace_service = Arc::new(workspace_service::WorkspaceService::new(
        workspace_repo,
        member_repo,
        Box::new(sqlite_repo::SqliteEventLogRepository::new(pool.clone())),
    ));

    let custom_field_service = Arc::new(custom_field_service::CustomFieldService::new(
        custom_field_repo,
    ));

    // Create additional repository instances for services that need them
    let user_repo_2: Box<dyn UserRepository> = Box::new(sqlite_repo::SqliteUserRepository::new(pool.clone()));
    let workspace_repo_2: Box<dyn WorkspaceRepository> = Box::new(sqlite_repo::SqliteWorkspaceRepository::new(pool.clone()));

    let report_service = Arc::new(report_service::ReportService::new(
        Box::new(sqlite_repo::SqliteTaskRepository::new(pool.clone())),
        user_repo_2,
        workspace_repo_2,
    ));

    let notification_service = Arc::new(notification_service::NotificationService::new());

    // Initialize Backup Service
    let backup_dir = PathBuf::from("./backups");
    let backup_service = Arc::new(backup_service::BackupService::new(
        db_path.to_string(),
        backup_dir,
        10, // max 10 backups
    ));

    // Initialize Rate Limiter (100 requests per minute)
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));

    // 5. Start Automatic Backup Scheduler
    let backup_service_clone = backup_service.clone();
    tokio::spawn(async move {
        let mut scheduler = tokio_cron_scheduler::JobScheduler::new().await.expect("Failed to create scheduler");
        
        // Schedule daily backup at 2 AM
        let job = tokio_cron_scheduler::Job::new_async("0 2 * * *", move |_uuid, _l| {
            let backup_service = backup_service_clone.clone();
            Box::pin(async move {
                if let Err(e) = backup_service.create_backup().await {
                    eprintln!("Failed to create scheduled backup: {}", e);
                } else {
                    println!("Scheduled backup created successfully");
                }
            })
        }).expect("Failed to create backup job");
        
        scheduler.add(job).await.expect("Failed to add backup job");
        scheduler.start().await.expect("Failed to start scheduler");
        
        // Keep scheduler running
        tokio::signal::ctrl_c().await.ok();
        scheduler.shutdown().await.ok();
    });

    // 6. Create App State
    let app_state = AppState {
        task_service: task_service.clone(),
        user_service: user_service.clone(),
        workspace_service: workspace_service.clone(),
        custom_field_service,
        report_service,
        notification_service,
        backup_service,
        jwt_secret,
    };

    // 6. Start API Server (blocking call)
    start_api_server(app_state).await;

    Ok(())
}
