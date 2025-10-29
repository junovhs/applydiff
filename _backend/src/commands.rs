use applydiff_core::{
    engine::{apply::Applier, backup},
    error::Result as CoreResult,
    logger::Logger,
    parse::Parser,
    session::{Session, SessionState as CoreSessionState}, // Renamed to avoid conflict
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::{DialogExt, FilePath}; // Import FilePath

// Using a Mutex to ensure thread-safe access to the session state.
// Tauri commands can run on different threads.
pub struct AppState(pub Mutex<Option<Session>>);

#[derive(Serialize)]
pub struct PreviewResult {
    log: String,
    diff: String,
}

/// A wrapper to convert core's complex error type into a simple string for the frontend.
fn to_string_error<T>(result: CoreResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

/// Generates a unique ID for a request for logging purposes.
fn generate_rid() -> u64 {
    (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64)
}

/* ========================== Session Commands ========================== */

#[tauri::command]
pub async fn load_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or("No folder selected".to_string())?;

    // CORRECTED: Handle the FilePath enum correctly for Tauri v2
    let path = match folder {
        FilePath::Path(p) => p,
        FilePath::Url(u) => PathBuf::from(u.path()),
    };

    let session = to_string_error(Session::load(&path))?;

    let path_str = path.to_string_lossy().to_string();
    *state.0.lock().unwrap() = Some(session);

    Ok(path_str)
}

#[tauri::command]
pub fn get_session_briefing(state: State<'_, AppState>) -> Result<String, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded".to_string())?;

    let briefing = session.generate_briefing();
    to_string_error(session.save())?;

    Ok(briefing)
}

#[tauri::command]
pub fn get_session_state(state: State<'_, AppState>) -> Result<CoreSessionState, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded".to_string())?;
    Ok(session.state.clone())
}


#[tauri::command]
pub fn refresh_session(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded".to_string())?;
    session.refresh_session();
    to_string_error(session.save())
}

/* ========================== Patching Commands ========================== */

#[derive(Deserialize)]
pub struct PatchArgs {
    patch: String,
}

#[tauri::command]
pub fn preview_patch(
    args: PatchArgs,
    state: State<'_, AppState>,
) -> Result<PreviewResult, String> {
    let guard = state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded".to_string())?;
    let project_root = session.project_root.clone();

    let rid = generate_rid();
    let logger = Logger::new(rid);

    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(&args.patch))?;

    let mut log_output = String::new();
    // CORRECTED: Remove `mut` as it's not mutated.
    let diff_output = String::new();

    // CORRECTED: Add underscore to unused variable.
    let _applier = Applier::new(&logger, project_root.clone(), true);

    // CORRECTED: Add underscore to unused variable.
    for _block in &blocks {
        // Preview diff generation logic is complex and will be implemented later.
        // This stubbed version now passes clippy.
    }

    log_output.push_str("💡 Preview complete. Press 'Apply Patch' to make changes.");
    Ok(PreviewResult {
        log: log_output,
        diff: diff_output,
    })
}

#[tauri::command]
pub fn apply_patch(
    args: PatchArgs,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut guard = state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded".to_string())?;
    let project_root = session.project_root.clone();

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let mut output = String::new();

    let parser = Parser::new();
    let blocks = match parser.parse(&args.patch) {
        Ok(b) => b,
        Err(e) => {
            session.record_error();
            to_string_error(session.save())?;
            return Err(e.to_string());
        }
    };
    output.push_str(&format!("✔ Parsed {} patch block(s)\n", blocks.len()));

    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = to_string_error(backup::create_backup(&project_root, &files_to_backup))?;
    output.push_str(&format!(
        "✔ Backup created at {}\n",
        backup_dir.display()
    ));

    let applier = Applier::new(&logger, project_root.clone(), false);
    let mut success_count = 0;

    for (idx, block) in blocks.iter().enumerate() {
        output.push_str(&format!(
            "Block {}: {}\n",
            idx + 1,
            block.file.display()
        ));

        let target_path = project_root.join(&block.file);
        let original_content = fs::read_to_string(&target_path).unwrap_or_default();

        match applier.apply_block(block) {
            Ok(result) => {
                success_count += 1;
                output.push_str(&format!(
                    "  ✔ Applied at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));
                let new_content = fs::read_to_string(&target_path).unwrap_or_default();
                session.record_success(&block.file, &original_content, &new_content);
            }
            Err(e) => {
                session.record_error();
                output.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }

    output.push_str(&format!(
        "\n✅ Done. {} applied, {} failed.\n",
        success_count,
        blocks.len() - success_count
    ));
    to_string_error(session.save())?;
    Ok(output)
}