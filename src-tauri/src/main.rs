#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

mod tauri_commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::pick_folder,
            tauri_commands::preview_patch,
            tauri_commands::apply_patch,
            tauri_commands::get_ai_prompt,
            tauri_commands::run_self_test,
            tauri_commands::create_demo,
            tauri_commands::resize_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
