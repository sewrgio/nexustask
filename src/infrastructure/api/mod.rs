pub mod routes;
pub mod middleware;
pub mod openapi;

use std::net::SocketAddr;
use crate::state::AppState;
use axum::routing::get;
use tower_http::cors::{CorsLayer, Any};

pub async fn start_api_server(state: AppState) {
    let app = routes::create_router(state.clone());
    
    // Add CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = app.layer(cors);
    
    // Add OpenAPI JSON endpoint
    let app = app.route("/api-docs/openapi.json", get(openapi::get_openapi));
    
    // Note: Swagger UI requires additional configuration with utoipa-swagger-ui
    // For now, the OpenAPI JSON is available at /api-docs/openapi.json
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8765));
    println!("API server listening on {}", addr);
    println!("OpenAPI documentation available at http://{}/api-docs/openapi.json", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
