#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

mod commands;

fn main() {
    tauri::Builder::default()
        .manage(commands::AppState(Default::default())) // Add the session state manager
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            // Session and State Commands
            commands::load_session,
            commands::get_session_briefing,
            commands::get_session_state,
            commands::refresh_session,
            // Core Patching Commands
            commands::preview_patch,
            commands::apply_patch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}