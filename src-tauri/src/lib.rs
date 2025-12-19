mod core;
mod commands;

use commands::*;
use core::auth::SessionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let session = Arc::new(SessionManager::new());
    let state = commands::AppState {
        session,
        cleaner: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![login, get_galleries, start_cleaning])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
