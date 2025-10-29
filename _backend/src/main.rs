#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

mod commands;

fn main() {
    tauri::Builder::default()
        .manage(commands::AppState(Default::default()))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::pick_project,
            commands::get_ai_prompt,
            commands::preview_patch,
            commands::apply_patch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}