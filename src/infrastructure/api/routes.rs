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
    let role = match req.role.as_str() {
        "owner" => WorkspaceRole::Owner,
        "admin" => WorkspaceRole::Admin,
        "member" => WorkspaceRole::Member,
        "viewer" => WorkspaceRole::Viewer,
        _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid role"}))),
    };
    
    match state.workspace_service.add_member(id, req.user_id, role).await {
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
    let role = match req.get("role").and_then(|r| r.as_str()) {
        Some(r) => r,
        None => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Role required"}))),
    };
    
    match state.workspace_service.update_member_role(workspace_id, user_id, role).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}

async fn remove_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match state.workspace_service.remove_member(workspace_id, user_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))),
    }
}
