use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header from request
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check Bearer token format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify JWT token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Extract user_id from claims
    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add user_id to request extensions
    request.extensions_mut().insert(AuthenticatedUser { user_id });

    Ok(next.run(request).await)
}

pub async fn optional_auth_middleware(
    State(jwt_secret): State<String>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract Authorization header
    if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Try to verify JWT token
            if let Ok(token_data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(jwt_secret.as_ref()),
                &Validation::default(),
            ) {
                // Extract user_id from claims
                if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                    // Add user_id to request extensions
                    request.extensions_mut().insert(AuthenticatedUser { user_id });
                }
            }
        }
    }

    next.run(request).await
}

// Workspace authorization middleware
// This middleware checks if the user has the required role in the workspace
#[derive(Clone)]
pub struct WorkspaceAuth {
    pub workspace_id: Uuid,
    pub user_role: String,
}

pub async fn workspace_auth_middleware(
    State(state): State<crate::state::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract user_id from request extensions (set by auth_middleware)
    let user_id = request
        .extensions()
        .get::<AuthenticatedUser>()
        .map(|u| u.user_id)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract workspace_id from path parameters
    let workspace_id = extract_workspace_id_from_path(&request)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Check if user is a member of the workspace
    match state.workspace_service.get_user_role(workspace_id, user_id).await {
        Ok(Some(role)) => {
            // Check if user has required permissions
            // For now, we'll allow all workspace members to access workspace resources
            // TODO: Implement proper role-based authorization
            request.extensions_mut().insert(WorkspaceAuth {
                workspace_id,
                user_role: role,
            });
            Ok(next.run(request).await)
        }
        Ok(None) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn extract_workspace_id_from_path(request: &Request) -> Option<Uuid> {
    // Extract workspace_id from URI path
    let uri = request.uri().path();
    
    // Try to extract workspace_id from patterns like:
    // /api/workspaces/:workspace_id/...
    // /api/workspaces/:workspace_id/members/...
    // /api/workspaces/:workspace_id/tasks/...
    // /api/workspaces/:workspace_id/report/...
    // /api/workspaces/:workspace_id/trends/...
    // /api/workspaces/:workspace_id/custom-fields/...
    
    let parts: Vec<&str> = uri.split('/').collect();
    
    // Look for workspace_id in the path
    for (i, part) in parts.iter().enumerate() {
        if *part == "workspaces" && i + 1 < parts.len() {
            if let Ok(uuid) = Uuid::parse_str(parts[i + 1]) {
                return Some(uuid);
            }
        }
    }
    
    None
}

// Rate limiting middleware
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }
    
    pub fn check_rate_limit(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        // Get or create request history for this key
        let history = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the time window
        history.retain(|&timestamp| now.duration_since(timestamp) < self.window);
        
        // Check if under limit
        if history.len() < self.max_requests {
            history.push(now);
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP address or user_id)
    let key = extract_rate_limit_key(&request);
    
    // Check rate limit
    if limiter.check_rate_limit(&key) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

fn extract_rate_limit_key(request: &Request) -> String {
    // Try to get user_id from authenticated user first
    if let Some(auth_user) = request.extensions().get::<AuthenticatedUser>() {
        return format!("user:{}", auth_user.user_id);
    }
    
    // Fall back to IP address
    if let Some(addr) = request.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>() {
        return format!("ip:{}", addr.0.ip());
    }
    
    // Default to a generic key
    "unknown".to_string()
}
