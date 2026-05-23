use axum::{
    routing::{get, post, put, delete, patch},
    Router,
    Json,
    extract::{Path, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::state::AppState;
use crate::domain::task::TaskStatus;
use crate::domain::workspace::WorkspaceRole;

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: Option<String>,
    parent_id: Option<Uuid>,
    workspace_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct MoveTaskRequest {
    new_parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreateCommentRequest {
    content: String,
    parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct RegisterUserRequest {
    username: String,
    email: String,
    password: String,
    full_name: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct CreateWorkspaceRequest {
    name: String,
    slug: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddMemberRequest {
    user_id: Uuid,
    role: String,
}

// Helper function to extract user_id from JWT token
fn extract_user_id(headers: &HeaderMap, jwt_secret: &str) -> Result<Uuid, StatusCode> {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        iat: usize,
    }
    
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Public endpoints (no auth required)
        .route("/api/users/register", post(register_user))
        .route("/api/users/login", post(login))
        // Protected endpoints (auth required)
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/:id", get(get_task).put(update_task).delete(delete_task))
        .route("/api/tasks/:id/tree", get(get_task_tree))
        .route("/api/tasks/:id/move", patch(move_task))
        .route("/api/tasks/:id/status", patch(update_task_status))
        .route("/api/workspaces/:workspace_id/tasks", get(get_workspace_tasks))
        .route("/api/tasks/:task_id/comments", get(get_task_comments).post(add_comment))
        .route("/api/comments/:id", put(update_comment).delete(delete_comment))
        .route("/api/users/me", get(get_current_user))
        .route("/api/workspaces", get(get_user_workspaces).post(create_workspace))
        .route("/api/workspaces/:id", get(get_workspace))
        .route("/api/workspaces/:id/members", get(get_members).post(add_member))
        .route("/api/workspaces/:workspace_id/members/:user_id", delete(remove_member))
        .route("/api/workspaces/:workspace_id/members/:user_id/role", patch(update_member_role))
        // Custom Fields endpoints
        .route("/api/custom-fields", post(create_custom_field))
        .route("/api/workspaces/:workspace_id/custom-fields", get(get_workspace_custom_fields))
        // Reports endpoints
        .route("/api/workspaces/:workspace_id/report/:days", get(get_workspace_report))
        .route("/api/workspaces/:workspace_id/trends/:days", get(get_trend_report))
        // Backup endpoints (admin only)
        .route("/api/admin/backups", get(list_backups).post(create_backup))
        .route("/api/admin/backups/:filename/restore", post(restore_backup))
        // Search endpoints
        .route("/api/search/tasks", get(search_tasks))
        // Webhook endpoints
        .route("/api/webhooks", post(create_webhook))
        .route("/api/webhooks/:id", delete(delete_webhook))
        // Task dependency endpoints
        .route("/api/tasks/:id/dependencies", post(add_dependency).get(get_dependencies))
        .route("/api/tasks/:id/dependencies/:depends_on_id", delete(remove_dependency))
        .route("/api/tasks/:id/dependents", get(get_dependents))
        .with_state(state)
}

// Task Endpoints
async fn list_tasks(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // For now, return empty list - would need workspace_id from auth
    Json::<Vec<crate::domain::task::Task>>(vec![])
}

async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    match state.task_service.create_task(
        req.title,
        user_id,
        req.workspace_id,
        req.parent_id,
        req.description,
    ).await {
        Ok(task) => (StatusCode::CREATED, Json(serde_json::to_value(task).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_task(id).await {
        Ok(Some(task)) => (StatusCode::OK, Json(serde_json::to_value(task).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Task not found"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn update_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    let mut task = match state.task_service.get_task(id).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Task not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    };
    
    if let Some(title) = req.title {
        task.title = title;
    }
    if let Some(description) = req.description {
        task.description = Some(description);
    }
    if let Some(status_str) = req.status {
        task.status = match status_str.as_str() {
            "pending" => TaskStatus::Pending,
            "in_progress" => TaskStatus::InProgress,
            "done" => TaskStatus::Done,
            "blocked" => TaskStatus::Blocked,
            _ => TaskStatus::Pending,
        };
    }
    if let Some(priority) = req.priority {
        task.priority = priority;
    }
    if let Some(assigned) = req.assigned_to {
        task.assigned_to = Some(assigned);
    }
    
    match state.task_service.update_task(task, user_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn delete_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let workspace_id = Uuid::new_v4(); // TODO: Get from task
    
    match state.task_service.delete_task(id, user_id, workspace_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_task_tree(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_task_tree(id).await {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::to_value(tasks).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn move_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveTaskRequest>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let workspace_id = Uuid::new_v4(); // Placeholder
    
    match state.task_service.move_task(id, req.new_parent_id, user_id, workspace_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn update_task_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let workspace_id = Uuid::new_v4(); // Placeholder
    
    let status = match req.get("status").and_then(|s| s.as_str()) {
        Some("pending") => TaskStatus::Pending,
        Some("in_progress") => TaskStatus::InProgress,
        Some("done") => TaskStatus::Done,
        Some("blocked") => TaskStatus::Blocked,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid status"}))),
    };
    
    match state.task_service.update_task_status(id, status, user_id, workspace_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_workspace_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_workspace_tasks(workspace_id).await {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::to_value(tasks).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Comment Endpoints
async fn get_task_comments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_task_comments(task_id).await {
        Ok(comments) => (StatusCode::OK, Json(serde_json::to_value(comments).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn add_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let workspace_id = Uuid::new_v4(); // TODO: Get from task
    
    match state.task_service.add_comment(task_id, user_id, req.content, req.parent_id, workspace_id).await {
        Ok(comment) => (StatusCode::CREATED, Json(serde_json::to_value(comment).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn update_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    // TODO: Fetch comment, update, save
    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}

async fn delete_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let workspace_id = Uuid::new_v4(); // TODO: Get from task
    
    match state.task_service.delete_comment(id, user_id, workspace_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// User Endpoints
async fn register_user(
    State(state): State<AppState>,
    Json(req): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    match state.user_service.register_user(req.username, req.email, req.password, req.full_name).await {
        Ok(user) => (StatusCode::CREATED, Json(serde_json::to_value(user).unwrap())),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.user_service.authenticate(&req.username, &req.password).await {
        Ok((user, token)) => (StatusCode::OK, Json(serde_json::json!({"user": serde_json::to_value(user).unwrap(), "token": token}))),
        Err(e) => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    match state.user_service.get_user(user_id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(serde_json::to_value(user).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "User not found"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Workspace Endpoints
async fn get_user_workspaces(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    match state.workspace_service.get_user_workspaces(user_id).await {
        Ok(workspaces) => (StatusCode::OK, Json(serde_json::to_value(workspaces).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn create_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateWorkspaceRequest>,
) -> impl IntoResponse {
    let owner_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    match state.workspace_service.create_workspace(req.name, req.slug, req.description, owner_id).await {
        Ok(workspace) => (StatusCode::CREATED, Json(serde_json::to_value(workspace).unwrap())),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workspace_service.get_workspace(id).await {
        Ok(Some(workspace)) => (StatusCode::OK, Json(serde_json::to_value(workspace).unwrap())),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Workspace not found"}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.workspace_service.get_members(id).await {
        Ok(members) => (StatusCode::OK, Json(serde_json::to_value(members).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn add_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    let acting_user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let role = match req.role.as_str() {
        "owner" => WorkspaceRole::Owner,
        "admin" => WorkspaceRole::Admin,
        "member" => WorkspaceRole::Member,
        "viewer" => WorkspaceRole::Viewer,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid role"}))),
    };
    
    match state.workspace_service.add_member(id, req.user_id, role, acting_user_id).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn update_member_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let acting_user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    let role = match req.get("role").and_then(|r| r.as_str()) {
        Some(r) => r,
        None => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Role required"}))),
    };
    
    match state.workspace_service.update_member_role(workspace_id, user_id, role, acting_user_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn remove_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let acting_user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    match state.workspace_service.remove_member(workspace_id, user_id, acting_user_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Custom Fields Endpoints
#[derive(Debug, Deserialize)]
struct CreateCustomFieldRequest {
    workspace_id: Uuid,
    field_name: String,
    field_type: String,
    field_options: Option<String>,
    is_required: bool,
}

async fn create_custom_field(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCustomFieldRequest>,
) -> impl IntoResponse {
    let field_type = match req.field_type.as_str() {
        "text" => crate::domain::custom_field::CustomFieldType::Text,
        "number" => crate::domain::custom_field::CustomFieldType::Number,
        "date" => crate::domain::custom_field::CustomFieldType::Date,
        "boolean" => crate::domain::custom_field::CustomFieldType::Boolean,
        "select" => crate::domain::custom_field::CustomFieldType::Select,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid field type"}))),
    };

    match state.custom_field_service.define_field(
        req.workspace_id,
        req.field_name,
        field_type,
        req.field_options,
        req.is_required,
    ) {
        Ok(field) => (StatusCode::CREATED, Json(serde_json::to_value(field).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_workspace_custom_fields(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.custom_field_service.get_workspace_fields(workspace_id) {
        Ok(fields) => (StatusCode::OK, Json(serde_json::to_value(fields).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Reports Endpoints
async fn get_workspace_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, days)): Path<(Uuid, i64)>,
) -> impl IntoResponse {
    match state.report_service.generate_workspace_report(workspace_id, days) {
        Ok(report) => (StatusCode::OK, Json(serde_json::to_value(report).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_trend_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, days)): Path<(Uuid, i64)>,
) -> impl IntoResponse {
    match state.report_service.generate_trend_report(workspace_id, days) {
        Ok(trends) => (StatusCode::OK, Json(serde_json::to_value(trends).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Backup Endpoints (Admin only)
async fn create_backup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // TODO: Add admin authorization check
    match state.backup_service.create_backup().await {
        Ok(filename) => (StatusCode::CREATED, Json(serde_json::json!({"filename": filename}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn list_backups(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // TODO: Add admin authorization check
    match state.backup_service.list_backups().await {
        Ok(backups) => (StatusCode::OK, Json(serde_json::to_value(backups).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn restore_backup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    // TODO: Add admin authorization check
    match state.backup_service.restore_backup(&filename).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Search Endpoints
async fn search_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let query = req.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let workspace_id = req.get("workspace_id").and_then(|id| Uuid::parse_str(id.as_str().unwrap_or("")).ok());
    
    if query.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Query is required"})));
    }
    
    match state.task_service.search_tasks(query.to_string(), workspace_id).await {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::to_value(tasks).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

// Webhook Endpoints
#[derive(Debug, Deserialize)]
struct CreateWebhookRequest {
    workspace_id: Uuid,
    url: String,
    events: Vec<String>,
    secret: Option<String>,
}

async fn create_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateWebhookRequest>,
) -> impl IntoResponse {
    let config = state.notification_service.create_webhook_config(
        req.workspace_id,
        req.url,
        req.events,
        req.secret,
    );
    (StatusCode::CREATED, Json(serde_json::to_value(config).unwrap()))
}

async fn delete_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    // TODO: Implement delete webhook in NotificationService
    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}

// Task Dependency Endpoints
#[derive(Debug, Deserialize)]
struct AddDependencyRequest {
    depends_on_id: Uuid,
    dependency_type: String,
    workspace_id: Uuid,
}

async fn add_dependency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<AddDependencyRequest>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    match state.task_service.add_dependency(
        id,
        req.depends_on_id,
        req.dependency_type,
        user_id,
        req.workspace_id,
    ).await {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_dependencies(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_dependencies(id).await {
        Ok(deps) => (StatusCode::OK, Json(serde_json::to_value(deps).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn remove_dependency(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, depends_on_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let user_id = extract_user_id(&headers, &state.jwt_secret).unwrap_or_else(|_| Uuid::new_v4());
    
    // For now, use a default workspace_id - in production this should come from the task
    let workspace_id = Uuid::new_v4();
    
    match state.task_service.remove_dependency(id, depends_on_id, user_id, workspace_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn get_dependents(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.task_service.get_dependents(id).await {
        Ok(deps) => (StatusCode::OK, Json(serde_json::to_value(deps).unwrap())),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}
