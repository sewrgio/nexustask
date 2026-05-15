pub mod routes;
pub mod middleware;
pub mod openapi;

use std::net::SocketAddr;
use crate::state::AppState;
use axum::{Router, routing::get};

pub async fn start_api_server(state: AppState) {
    let app = routes::create_router(state);
    let app = app.route("/api-docs/openapi.json", get(openapi::get_openapi));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8765));
    println!("API server listening on {}", addr);
    println!("OpenAPI documentation available at http://{}/api-docs/openapi.json", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
