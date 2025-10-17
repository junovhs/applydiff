#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(warnings)]

mod apply;
mod backup;
mod error;
mod logger;
mod matcher;
mod parser;
mod prompts;
mod gauntlet;
mod tauri_commands;

fn main() {
    tauri::Builder::default()
        // plugins
        .plugin(tauri_plugin_dialog::init())
        // commands your UI can invoke
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
