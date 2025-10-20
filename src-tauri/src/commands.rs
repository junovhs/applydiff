use applydiff_core::{
    apply::Applier,
    backup,
    error::Result as PatchResult,
    logger::Logger,
    parser::Parser,
};
use chrono::Local;
use serde::Serialize;
use similar::TextDiff;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_dialog::{DialogExt, FilePath};

const MAX_INPUT_SIZE: usize = 100_000_000;

#[derive(Serialize)]
pub struct PreviewResult {
    pub log: String,
    pub diff: String,
}

/* ========================== Commands ========================== */

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<String, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    match folder {
        Some(FilePath::Path(path)) => Ok(path.to_string_lossy().to_string()),
        Some(FilePath::Url(url)) => Ok(url.to_string()),
        None => Err("No folder selected".to_string()),
    }
}

#[tauri::command]
pub fn get_ai_prompt() -> String {
    applydiff_core::prompts::build_ai_prompt()
}

#[tauri::command]
pub fn run_self_test() -> String {
    applydiff_core::gauntlet::run()
}

#[tauri::command]
pub fn preview_patch(target: String, patch: String) -> Result<PreviewResult, String> {
    preview_patch_impl(&target, &patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_patch(target: String, patch: String) -> Result<String, String> {
    apply_patch_impl(&target, &patch).map_err(|e| e.to_string())
}

/* ========================== Impl ========================== */

fn preview_patch_impl(target: &str, patch: &str) -> PatchResult<PreviewResult> {
    use applydiff_core::error::{ErrorCode, PatchError};

    let rid = generate_rid();
    let logger = Logger::new(rid);

    let mut log = String::new();
    let mut diffs = String::new();

    let target_path = PathBuf::from(target);
    if !target_path.exists() || !target_path.is_dir() {
        return Err(PatchError::Validation {
            code: ErrorCode::ValidationFailed,
            message: "Target directory does not exist".to_string(),
            context: target.to_string(),
        });
    }

    if patch.len() > MAX_INPUT_SIZE {
        return Err(PatchError::Validation {
            code: ErrorCode::BoundsExceeded,
            message: format!("Input exceeds max size {}", MAX_INPUT_SIZE),
            context: "input".to_string(),
        });
    }

    let parser = Parser::new();
    let blocks = parser.parse(patch)?;
    log.push_str(&format!("✔ Parsed {} patch block(s)\n\n", blocks.len()));

    let applier = Applier::new(&logger, target_path.clone(), true);
    for (idx, block) in blocks.iter().enumerate() {
        log.push_str(&format!("Block {}: {}\n", idx + 1, block.file.display()));
        match applier.apply_block(block) {
            Ok(result) => {
                log.push_str(&format!(
                    "  ✔ Preview match at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));

                let file_path = target_path.join(&block.file);
                let content = fs::read_to_string(&file_path).unwrap_or_default();

                let start = result.matched_at as usize;
                let end = result.matched_end.min(content.len());

                if start <= end {
                    let before = &content[start..end];

                    // Harmonize EOLs
                    let matched_nl = if before.ends_with("\r\n") {
                        "\r\n"
                    } else if before.ends_with('\n') {
                        "\n"
                    } else {
                        ""
                    };
                    
                    let mut to_text = block.to.clone();
                    if !matched_nl.is_empty() {
                        if to_text.ends_with("\r\n") && matched_nl == "\n" {
                            let new_len = to_text.len().saturating_sub(2);
                            to_text.truncate(new_len);
                            to_text.push('\n');
                        } else if to_text.ends_with('\n') && matched_nl == "\r\n" {
                            to_text.pop();
                            to_text.push_str("\r\n");
                        } else if !to_text.ends_with('\n') && !to_text.ends_with("\r\n") {
                            to_text.push_str(matched_nl);
                        }
                    }

                    // Mirror append separator newline from apply logic
                    if start == content.len() && !content.is_empty() && !content.ends_with('\n') && !to_text.is_empty() {
                        to_text.insert(0, '\n');
                    }

                    let udiff = TextDiff::from_lines(before, &to_text)
                        .unified_diff()
                        .header(
                            &format!("a/{}", block.file.display()),
                            &format!("b/{}", block.file.display()),
                        )
                        .to_string();

                    if !udiff.trim().is_empty() {
                        diffs.push_str(&udiff);
                        if !diffs.ends_with('\n') {
                            diffs.push('\n');
                        }
                    }
                }
            }
            Err(e) => {
                log.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }

    log.push_str("\n💡 Preview complete. Press 'Apply Patch' to make changes.");
    Ok(PreviewResult { log, diff: diffs })
}

fn apply_patch_impl(target: &str, patch: &str) -> PatchResult<String> {
    use applydiff_core::error::{ErrorCode, PatchError};

    let rid = generate_rid();
    let logger = Logger::new(rid);

    let mut output = String::new();

    let target_path = PathBuf::from(target);
    if !target_path.exists() || !target_path.is_dir() {
        return Err(PatchError::Validation {
            code: ErrorCode::ValidationFailed,
            message: "Target directory does not exist".to_string(),
            context: target.to_string(),
        });
    }

    if patch.len() > MAX_INPUT_SIZE {
        return Err(PatchError::Validation {
            code: ErrorCode::BoundsExceeded,
            message: format!("Input exceeds max size {}", MAX_INPUT_SIZE),
            context: "input".to_string(),
        });
    }

    let parser = Parser::new();
    let blocks = parser.parse(patch)?;
    output.push_str(&format!("✔ Parsed {} patch block(s)\n", blocks.len()));

    // Backup before applying
    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = backup::create_backup(&target_path, &files_to_backup)?;
    output.push_str(&format!("✔ Backup created at {}\n", backup_dir.display()));

    // Apply (partial success allowed)
    let applier = Applier::new(&logger, target_path.clone(), false);
    let mut success = 0usize;
    let mut failed = 0usize;

    for (idx, block) in blocks.iter().enumerate() {
        output.push_str(&format!("Block {}: {}\n", idx + 1, block.file.display()));
        match applier.apply_block(block) {
            Ok(result) => {
                success += 1;
                output.push_str(&format!(
                    "  ✔ Applied at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));
            }
            Err(e) => {
                failed += 1;
                output.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }

    output.push_str(&format!("\n✅ Done. {} applied, {} failed.\n", success, failed));
    output.push_str("↩ Backups live next to your files in a timestamped .applydiff_backup_* folder.\n");
    Ok(output)
}

fn generate_rid() -> u64 {
    (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64)
}