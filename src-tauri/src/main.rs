#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![deny(warnings)]

mod apply;
mod backup;
mod error;
mod gauntlet;
mod logger;
mod matcher;
mod parser;
mod prompts;
mod tauri_commands;

use tauri_commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pick_folder,
            preview_patch,
            apply_patch,
            get_ai_prompt,
            run_self_test,
            create_demo,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}