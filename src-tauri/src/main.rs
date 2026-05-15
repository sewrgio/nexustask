// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    success: bool,
    message: String,
    token: Option<String>,
}

#[tauri::command]
async fn login(username: String, password: String) -> Result<AuthResponse, String> {
    // Aquí integraremos tu lógica real de src/application/user_service.rs
    if username == "sergio" && password == "sergio123" {
        Ok(AuthResponse {
            success: true,
            message: "Bienvenido Sergio".into(),
            token: Some("dummy_token".into()),
        })
    } else {
        Ok(AuthResponse {
            success: false,
            message: "Credenciales inválidas".into(),
            token: None,
        })
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
