use applydiff_core::{
    engine::{apply::Applier, backup},
    error::Result as CoreResult,
    logger::Logger,
    parse::Parser,
    session::Session,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

// Using a Mutex to ensure thread-safe access to the session state.
// Tauri commands can run on different threads.
type SessionState = Mutex<Option<Session>>;

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
    session_state: State<'_, SessionState>,
) -> Result<String, String> {
    let folder_path = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or("No folder selected")?;

    let path = PathBuf::from(folder_path.path().to_string_lossy().to_string());
    let session = to_string_error(Session::load(&path))?;

    let path_str = path.to_string_lossy().to_string();
    *session_state.lock().unwrap() = Some(session);

    Ok(path_str)
}

#[tauri::command]
pub fn get_session_briefing(session_state: State<'_, SessionState>) -> Result<String, String> {
    let mut guard = session_state.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded")?;

    let briefing = session.generate_briefing();
    to_string_error(session.save())?; // Save the session to update exchange_count

    Ok(briefing)
}

#[tauri::command]
pub fn refresh_session(session_state: State<'_, SessionState>) -> Result<(), String> {
    let mut guard = session_state.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded")?;
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
    session_state: State<'_, SessionState>,
) -> Result<PreviewResult, String> {
    let guard = session_state.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded")?;
    let project_root = session.project_root.clone();

    let rid = generate_rid();
    let logger = Logger::new(rid);

    // Parse the patch
    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(&args.patch))?;

    let mut log_output = String::new();
    let mut diff_output = String::new();

    let applier = Applier::new(&logger, project_root.clone(), true); // Dry run for preview

    for block in &blocks {
        // ... implementation for generating diffs (omitted for brevity, as it's complex and UI-focused)
        // In a real implementation, you'd use the `similar` crate to generate a unified diff
        // between the 'before' and 'after' state of the matched block.
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
    session_state: State<'_, SessionState>,
) -> Result<String, String> {
    let mut guard = session_state.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded")?;
    let project_root = session.project_root.clone();

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let mut output = String::new();

    // Parse
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

    // Backup
    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = to_string_error(backup::create_backup(&project_root, &files_to_backup))?;
    output.push_str(&format!(
        "✔ Backup created at {}\n",
        backup_dir.display()
    ));

    // Apply
    let applier = Applier::new(&logger, project_root.clone(), false); // Not a dry run
    let mut success_count = 0;

    for (idx, block) in blocks.iter().enumerate() {
        output.push_str(&format!(
            "Block {}: {}\n",
            idx + 1,
            block.file.display()
        ));

        // Read original content for metrics before applying
        let target_path = project_root.join(&block.file);
        let original_content = fs::read_to_string(&target_path).unwrap_or_default();

        match applier.apply_block(block) {
            Ok(result) => {
                success_count += 1;
                output.push_str(&format!(
                    "  ✔ Applied at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));
                // Read new content for metrics
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
    to_string_error(session.save())?; // Save session state after applying
    Ok(output)
}