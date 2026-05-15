use utoipa::OpenApi;
use axum::{
    routing::get,
    Json,
    response::IntoResponse,
};

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        crate::domain::user::User,
        crate::domain::user::UserRole,
        crate::domain::task::Task,
        crate::domain::task::TaskStatus,
        crate::domain::workspace::Workspace,
        crate::domain::workspace::WorkspaceRole,
        crate::domain::workspace::WorkspaceMember,
        crate::domain::comment::Comment,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management"),
        (name = "tasks", description = "Task management"),
        (name = "comments", description = "Comment management"),
        (name = "workspaces", description = "Workspace management"),
    ),
    info(
        title = "NexusTask API",
        version = "2.0.0",
        description = "Enterprise Task Management System API with JWT authentication",
        contact(name = "NexusTask Team")
    ),
    servers(
        (url = "http://localhost:8765", description = "Local development server"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;

pub async fn get_openapi() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}
