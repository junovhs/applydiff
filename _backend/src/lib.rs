#![deny(warnings)]

pub mod commands;

/// Runs the Tauri application.
///
/// # Panics
///
/// Panics if the Tauri application fails to run.
pub fn main() {
    tauri::Builder::default()
        .manage(commands::AppState(std::sync::Mutex::default()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::init_session,
            commands::get_session_briefing,
            commands::refresh_session,
            commands::resolve_file_request,
            commands::preview_patch,
            commands::apply_patch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}