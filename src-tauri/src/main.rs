#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(serde::Serialize)]
struct PatchResult {
    success: bool,
    message: String,
    files_patched: Vec<String>,
}

#[tauri::command]
fn apply_patch(patch_content: String, base_path: String) -> PatchResult {
    let base = PathBuf::from(&base_path);
    
    if !base.exists() {
        return PatchResult {
            success: false,
            message: "Base path does not exist".to_string(),
            files_patched: vec![],
        };
    }

    let mut files_patched = Vec::new();
    let lines: Vec<&str> = patch_content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        if lines[i].starts_with("--- ") && i + 1 < lines.len() && lines[i + 1].starts_with("+++ ") {
            let mut file_path = lines[i + 1][4..].trim().to_string();
            if file_path.starts_with("b/") {
                file_path = file_path[2..].to_string();
            }
            
            let full_path = base.join(&file_path);
            if !full_path.exists() {
                return PatchResult {
                    success: false,
                    message: format!("File {} does not exist", file_path),
                    files_patched,
                };
            }
            
            let original_content = match fs::read_to_string(&full_path) {
                Ok(content) => content,
                Err(e) => {
                    return PatchResult {
                        success: false,
                        message: format!("Failed to read {}: {}", file_path, e),
                        files_patched,
                    };
                }
            };
            
            let mut original_lines: Vec<String> = original_content.lines().map(|s| s.to_string()).collect();
            let mut offset: i32 = 0;
            
            i += 2;
            
            while i < lines.len() && lines[i].starts_with("@@") {
                let hunk_header = lines[i];
                let parts: Vec<&str> = hunk_header.split_whitespace().collect();
                if parts.len() < 3 {
                    i += 1;
                    continue;
                }
                
                let old_range = parts[1];
                let old_start = if let Some(comma_pos) = old_range.find(',') {
                    old_range[1..comma_pos].parse::<i32>().unwrap_or(1) - 1
                } else {
                    old_range[1..].parse::<i32>().unwrap_or(1) - 1
                };
                
                i += 1;
                let mut old_line_num = (old_start + offset) as usize;
                
                while i < lines.len() && !lines[i].starts_with("@@") && !lines[i].starts_with("--- ") {
                    if lines[i].starts_with("-") {
                        if old_line_num < original_lines.len() {
                            original_lines.remove(old_line_num);
                            offset -= 1;
                        }
                    } else if lines[i].starts_with("+") {
                        let new_line = lines[i][1..].to_string();
                        if old_line_num <= original_lines.len() {
                            original_lines.insert(old_line_num, new_line);
                            old_line_num += 1;
                            offset += 1;
                        }
                    } else if lines[i].starts_with(" ") {
                        old_line_num += 1;
                    }
                    i += 1;
                }
            }
            
            let patched_content = original_lines.join("\n");
            if let Err(e) = fs::write(&full_path, patched_content) {
                return PatchResult {
                    success: false,
                    message: format!("Failed to write {}: {}", file_path, e),
                    files_patched,
                };
            }
            
            files_patched.push(file_path);
        } else {
            i += 1;
        }
    }

    PatchResult {
        success: true,
        message: format!("Successfully patched {} file(s)", files_patched.len()),
        files_patched,
    }
}

#[tauri::command]
fn get_directory_info(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if p.is_dir() {
        Ok(format!("📁 {}", p.file_name().unwrap_or_default().to_string_lossy()))
    } else if p.is_file() {
        Ok(format!("📄 {}", p.file_name().unwrap_or_default().to_string_lossy()))
    } else {
        Err("Invalid path".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.set_always_on_top(true)?;
            window.set_decorations(false)?;
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![apply_patch, get_directory_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}