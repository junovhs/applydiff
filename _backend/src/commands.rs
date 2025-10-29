use applydiff_core::{
    engine::{apply::Applier, backup},
    error::Result as CoreResult,
    logger::Logger,
    parse::Parser,
    session::prompts,
};
use chrono::Local;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::{DialogExt, FilePath};

// The state is now just the project path. Simple and reliable.
pub struct AppState(pub Mutex<Option<PathBuf>>);

#[derive(Serialize)]
pub struct PreviewResult {
    log: String,
    diff: String,
}

fn to_string_error<T>(result: CoreResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

fn generate_rid() -> u64 {
    (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64)
}

/* ========================== Core Commands ========================== */

#[tauri::command]
pub async fn pick_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or("No folder selected".to_string())?;

    let path = match folder {
        FilePath::Path(p) => p,
        FilePath::Url(u) => PathBuf::from(u.path()),
    };

    let path_str = path.to_string_lossy().to_string();
    *state.0.lock().unwrap() = Some(path);

    Ok(path_str)
}

#[tauri::command]
pub fn get_ai_prompt() -> String {
    prompts::build_ai_prompt()
}

#[tauri::command]
pub fn preview_patch(
    patch: String,
    state: State<'_, AppState>,
) -> Result<PreviewResult, String> {
    let guard = state.0.lock().unwrap();
    let project_root = guard.as_ref().cloned().ok_or("Project not loaded".to_string())?;

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(&patch))?;

    let mut log_output = String::new();
    let mut diff_output = String::new();
    let applier = Applier::new(&logger, project_root.clone(), true); // Dry run

    for block in &blocks {
        log_output.push_str(&format!("Block: {}\n", block.file.display()));
        let original_content = fs::read_to_string(project_root.join(&block.file)).unwrap_or_default();
        match applier.apply_block(block) {
            Ok(result) => {
                log_output.push_str(&format!("  ✔ Match found (score: {:.2})\n", result.score));
                let mut new_content = String::new();
                new_content.push_str(&original_content[..result.matched_at]);
                new_content.push_str(&block.to);
                new_content.push_str(&original_content[result.matched_end..]);
                let udiff = similar::TextDiff::from_lines(&original_content, &new_content)
                    .unified_diff().header("before", "after").to_string();
                if !udiff.trim().is_empty() {
                    diff_output.push_str(&udiff);
                }
            }
            Err(e) => {
                log_output.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }
    Ok(PreviewResult { log: log_output, diff: diff_output })
}

#[tauri::command]
pub fn apply_patch(
    patch: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let guard = state.0.lock().unwrap();
    let project_root = guard.as_ref().cloned().ok_or("Project not loaded".to_string())?;

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(&patch))?;

    let mut output = String::new();
    output.push_str(&format!("✔ Parsed {} block(s)\n", blocks.len()));

    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = to_string_error(backup::create_backup(&project_root, &files_to_backup))?;
    output.push_str(&format!("✔ Backup created at {}\n", backup_dir.display()));

    let applier = Applier::new(&logger, project_root.clone(), false);
    let (success_count, fail_count) = blocks.iter().fold((0, 0), |(s, f), block| {
        output.push_str(&format!("Applying to {}\n", block.file.display()));
        match applier.apply_block(block) {
            Ok(res) => {
                output.push_str(&format!("  ✔ Applied (score: {:.2})\n", res.score));
                (s + 1, f)
            }
            Err(e) => {
                output.push_str(&format!("  ❌ {}\n", e));
                (s, f + 1)
            }
        }
    });

    output.push_str(&format!("\n✅ Done. {} applied, {} failed.\n", success_count, fail_count));
    Ok(output)
}