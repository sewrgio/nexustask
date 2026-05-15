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
