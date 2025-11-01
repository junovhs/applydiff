use applydiff_core::{
    engine::{apply::Applier, backup},
    error::{ErrorCode, PatchError, Result as CoreResult},
    logger::Logger,
    parse::Parser,
    session::{prompts, state::{FileMetrics, SessionState}},
};
use chrono::{Local, Utc};
use saccade_core::{
    config::Config as SaccadeConfig,
    request::{RequestFile, RequestRange, RequestTarget},
    SaccadePack,
};
use serde::Serialize;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

pub struct AppState(pub Mutex<Option<SessionState>>);

#[derive(Serialize, Debug)]
pub struct PreviewResult {
    pub log: String,
    pub diff: String,
}

#[derive(Serialize, Debug)]
pub struct CommandResult {
    pub output: String,
    pub session_state: Option<SessionState>,
}

fn to_string_error<T>(result: CoreResult<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}

fn generate_rid() -> u64 {
    // Note: timestamp_millis() is i64. Cast is acknowledged as acceptable for a unique ID.
    #[allow(clippy::cast_sign_loss)]
    let timestamp = Local::now().timestamp_millis() as u64;
    timestamp ^ u64::from(std::process::id())
}

fn get_session_path(project_root: &Path) -> PathBuf {
    project_root.join(".applydiff/session.json")
}

fn save_session_state(session_state: &SessionState) -> Result<(), String> {
    let path = get_session_path(&session_state.project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(session_state).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

// ======================================================================
// TESTABLE LOGIC FUNCTIONS
// ======================================================================

/// # Errors
/// Will return an error if it fails to set the CWD, run Saccade, or initialize state.
pub fn init_session_logic(project_root: &Path) -> Result<SessionState, String> {
    let original_cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(project_root).map_err(|e| format!("Failed to change directory: {e}"))?;

    let mut saccade_config = SaccadeConfig::new();
    saccade_config.pack_dir = project_root.join(".applydiff/saccade");
    SaccadePack::new(saccade_config).generate().map_err(|e| format!("Saccade failed: {e}"))?;
    std::env::set_current_dir(&original_cwd).map_err(|e| e.to_string())?;

    let mut session_state = SessionState::new(project_root.to_path_buf());
    for entry in walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|e| !e.path().to_string_lossy().contains(".applydiff"))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(content) = fs::read(entry.path()) {
            let hash = format!("{:x}", md5::compute(content));
            if let Ok(relative_path) = entry.path().strip_prefix(project_root) {
                session_state.file_metrics.insert(relative_path.to_path_buf(), FileMetrics { original_hash: hash, patch_count: 0 });
            }
        }
    }
    save_session_state(&session_state)?;
    Ok(session_state)
}

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if the session is not loaded.
pub fn get_session_briefing_logic(app_state: &AppState) -> Result<String, String> {
    let guard = app_state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded".to_string())?;
    Ok(prompts::build_session_briefing(session))
}

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if the session is not loaded.
pub fn refresh_session_logic(app_state: &AppState) -> Result<SessionState, String> {
    let mut guard = app_state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded".to_string())?;
    session.exchange_count = 0;
    session.last_refresh_ts = Utc::now();
    // ... (rest of the refresh logic)
    save_session_state(session)?;
    Ok(session.clone())
}

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if the session is not loaded or the request is invalid.
pub fn resolve_file_request_logic(request_yaml: &str, app_state: &AppState) -> Result<String, String> {
    let guard = app_state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded".to_string())?;
    
    let mut path = None;
    let mut reason = "No reason provided".to_string();
    let mut range = None;

    for line in request_yaml.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let val = value.trim();
            match key.trim() {
                "path" => path = Some(val.to_string()),
                "reason" => reason = val.to_string(),
                "range" => {
                    // More robustly find the first separator (space or colon)
                    let separator_idx = val.find(|c: char| c == ' ' || c == ':');
                    if let Some(idx) = separator_idx {
                        let (r_key, r_val_with_sep) = val.split_at(idx);
                        // The rest of the string, skipping the separator itself.
                        let r_val = r_val_with_sep.get(1..).unwrap_or("").trim();
                        range = match r_key.trim() {
                            "lines" => Some(RequestRange::Lines { lines: r_val.to_string() }),
                            "symbol" => Some(RequestRange::Symbol { symbol: r_val.to_string() }),
                            _ => None,
                        };
                    }
                }
                _ => {}
            }
        }
    }
    
    let target = if let Some(p) = path { RequestTarget::SinglePath { path: p } } else { return Err("Request must contain a 'path'".to_string()); };
    let req = RequestFile { target, reason, range };
    let available_files: Vec<PathBuf> = session.file_metrics.keys().cloned().collect();

    let resolved = req.resolve(&available_files, &session.project_root).map_err(|e| e.to_string())?;
    Ok(resolved.to_markdown())
}

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if the session is not loaded or the patch is invalid.
pub fn preview_patch_logic(patch: &str, app_state: &AppState) -> Result<PreviewResult, String> {
    let guard = app_state.0.lock().unwrap();
    let session = guard.as_ref().ok_or("Session not loaded".to_string())?;
    let project_root = &session.project_root;

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(patch))?;

    let mut log_output = String::new();
    let mut diff_output = String::new();
    let applier = Applier::new(&logger, project_root.clone(), true);

    for block in &blocks {
        writeln!(&mut log_output, "Block: {} (mode: {:?})", block.file.display(), block.mode).unwrap();
        let original_content = fs::read_to_string(project_root.join(&block.file)).unwrap_or_default();
        match applier.apply_block(block) {
            Ok(result) => {
                writeln!(&mut log_output, "  ✔ Preview successful (score: {:.2})", result.score).unwrap();
                let new_content = match block.mode {
                    applydiff_core::parse::PatchMode::Classic => {
                        let mut nc = String::new();
                        nc.push_str(&original_content[..result.matched_at]);
                        nc.push_str(&block.to);
                        nc.push_str(&original_content[result.matched_end..]);
                        nc
                    }
                    applydiff_core::parse::PatchMode::Replace => block.to.clone(),
                    applydiff_core::parse::PatchMode::Regex => {
                        regex::Regex::new(&block.from).map_err(|e| e.to_string())?
                            .replace_all(&original_content, &block.to[..]).to_string()
                    }
                };
                let udiff = similar::TextDiff::from_lines(&original_content, &new_content)
                    .unified_diff().header("before", "after").to_string();
                if !udiff.trim().is_empty() { diff_output.push_str(&udiff); }
            }
            Err(e) => { writeln!(&mut log_output, "  ❌ {e}").unwrap(); }
        }
    }
    Ok(PreviewResult { log: log_output, diff: diff_output })
}

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if the session is not loaded or the patch is invalid.
pub fn apply_patch_logic(patch: &str, app_state: &AppState) -> Result<CommandResult, String> {
    let mut guard = app_state.0.lock().unwrap();
    let session = guard.as_mut().ok_or("Session not loaded".to_string())?;
    let project_root = session.project_root.clone();
    session.exchange_count += 1;

    let rid = generate_rid();
    let logger = Logger::new(rid);
    let parser = Parser::new();
    let blocks = to_string_error(parser.parse(patch))?;
    let mut output = String::new();
    writeln!(&mut output, "✔ Parsed {} block(s)", blocks.len()).unwrap();

    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = to_string_error(backup::create_backup(&project_root, &files_to_backup))?;
    writeln!(&mut output, "✔ Backup created at {}", backup_dir.display()).unwrap();

    let applier = Applier::new(&logger, project_root.clone(), false);
    for block in &blocks {
        writeln!(&mut output, "Applying to {}", block.file.display()).unwrap();
        match applier.apply_block(block) {
            Ok(res) => {
                writeln!(&mut output, "  ✔ Applied (score: {:.2})", res.score).unwrap();
                session.file_metrics.entry(block.file.clone()).or_insert(FileMetrics { original_hash: String::new(), patch_count: 0 }).patch_count += 1;
            }
            Err(e) => {
                if let PatchError::Apply { code, .. } = &e {
                    if *code == ErrorCode::NoMatch || *code == ErrorCode::AmbiguousMatch {
                        session.total_errors += 1;
                        output.push_str("  -> Prediction Error detected. Incrementing total_errors.\n");
                    }
                }
                 writeln!(&mut output, "  ❌ {e}").unwrap();
            }
        }
    }
    save_session_state(session)?;
    Ok(CommandResult { output, session_state: Some(session.clone()) })
}


// ======================================================================
// TAURI COMMAND WRAPPERS
// ======================================================================

/// # Panics
/// Panics if the mutex is poisoned.
/// # Errors
/// Returns an error if folder selection is cancelled or session init fails.
#[tauri::command]
pub async fn init_session(app: AppHandle, state: State<'_, AppState>) -> Result<SessionState, String> {
    let folder = app.dialog().file().blocking_pick_folder().ok_or("No folder selected".to_string())?;
    let project_root = match folder {
        FilePath::Path(p) => p,
        FilePath::Url(u) => PathBuf::from(u.path()),
    };
    let session_state = init_session_logic(&project_root)?;
    let return_state = session_state.clone();
    *state.0.lock().unwrap() = Some(session_state);
    Ok(return_state)
}

/// # Errors
/// Returns an error if the logic function fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn get_session_briefing(state: State<'_, AppState>) -> Result<String, String> {
    get_session_briefing_logic(&state)
}

/// # Errors
/// Returns an error if the logic function fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn refresh_session(state: State<'_, AppState>) -> Result<SessionState, String> {
    refresh_session_logic(&state)
}

/// # Errors
/// Returns an error if the logic function fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn resolve_file_request(request_yaml: String, state: State<'_, AppState>) -> Result<String, String> {
    resolve_file_request_logic(&request_yaml, &state)
}

/// # Errors
/// Returns an error if the logic function fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn preview_patch(patch: String, state: State<'_, AppState>) -> Result<PreviewResult, String> {
    preview_patch_logic(&patch, &state)
}

/// # Errors
/// Returns an error if the logic function fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn apply_patch(patch: String, state: State<'_, AppState>) -> Result<CommandResult, String> {
    apply_patch_logic(&patch, &state)
}