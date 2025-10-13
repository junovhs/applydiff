#![deny(warnings)]
mod apply;
mod error;
mod logger;
mod matcher;
mod parser;
mod gauntlet; // NEW

use apply::Applier;
use error::{ErrorCode, PatchError, Result as PatchResult};
use logger::Logger;
use parser::Parser;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::Local;

slint::include_modules!();

const MAX_INPUT_SIZE: usize = 100_000_000;

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    // Folder picker
    {
        let ui_handle = ui.as_weak();
        ui.on_pick_folder(move || {
            let ui = ui_handle.unwrap();
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                ui.set_target_dir(path.display().to_string().into());
                append_log(&ui, &format!("📁 Selected: {}", path.display()));
            }
        });
    }

    // Preview
    {
        let ui_handle = ui.as_weak();
        ui.on_preview_patch(move || {
            let ui = ui_handle.unwrap();
            let target = ui.get_target_dir().to_string();
            let patch = ui.get_patch_input().to_string();

            if target.is_empty() || patch.is_empty() {
                append_log(&ui, "❌ Error: Please select directory and enter patch");
                return;
            }

            ui.set_is_processing(true);
            clear_log(&ui);
            append_log(&ui, "👁 Previewing patch...\n");

            let ui_weak = ui.as_weak();
            std::thread::spawn(move || {
                let result = preview_patch(&target, &patch);
                slint::invoke_from_event_loop(move || {
                    let ui = ui_weak.unwrap();
                    match result {
                        Ok(msg) => append_log(&ui, &msg),
                        Err(e) => append_log(&ui, &format!("❌ Error: {}", e)),
                    }
                    ui.set_is_processing(false);
                }).ok();
            });
        });
    }

    // Apply
    {
        let ui_handle = ui.as_weak();
        ui.on_apply_patch(move || {
            let ui = ui_handle.unwrap();
            let target = ui.get_target_dir().to_string();
            let patch = ui.get_patch_input().to_string();

            if target.is_empty() || patch.is_empty() {
                append_log(&ui, "❌ Error: Please select directory and enter patch");
                return;
            }

            ui.set_is_processing(true);
            clear_log(&ui);
            append_log(&ui, "⚙️ Applying patch...\n");

            let ui_weak = ui.as_weak();
            std::thread::spawn(move || {
                let result = apply_patch(&target, &patch);
                slint::invoke_from_event_loop(move || {
                    let ui = ui_weak.unwrap();
                    match result {
                        Ok(msg) => append_log(&ui, &msg),
                        Err(e) => append_log(&ui, &format!("❌ Error: {}", e)),
                    }
                    ui.set_is_processing(false);
                }).ok();
            });
        });
    }

    // Self-test gauntlet
    {
        let ui_handle = ui.as_weak();
        ui.on_run_self_test(move || {
            let ui = ui_handle.unwrap();
            ui.set_is_processing(true);
            clear_log(&ui);
            append_log(&ui, "🧪 Running self-test gauntlet…\n");

            let ui_weak = ui.as_weak();
            std::thread::spawn(move || {
                let log = gauntlet::run();
                slint::invoke_from_event_loop(move || {
                    let ui = ui_weak.unwrap();
                    append_log(&ui, &log);
                    ui.set_is_processing(false);
                }).ok();
            });
        });
    }

    ui.run()
}

/* ========================== Core operations ========================== */

fn preview_patch(target: &str, patch: &str) -> PatchResult<String> {
    let rid = generate_rid();
    let logger = Logger::new(rid);
    let mut output = String::new();

    // Validate target
    let target_path = PathBuf::from(target);
    if !target_path.exists() || !target_path.is_dir() {
        return Err(PatchError::Validation {
            code: ErrorCode::ValidationFailed,
            message: "Target directory does not exist".to_string(),
            context: target.to_string(),
        });
    }

    // Bound input size
    if patch.len() > MAX_INPUT_SIZE {
        return Err(PatchError::Validation {
            code: ErrorCode::BoundsExceeded,
            message: format!("Input exceeds max size {}", MAX_INPUT_SIZE),
            context: "input".to_string(),
        });
    }

    // Parse blocks
    let parser = Parser::new();
    let blocks = parser.parse(patch)?;
    output.push_str(&format!("✓ Parsed {} patch block(s)\n\n", blocks.len()));

    // Dry-run matching
    let applier = Applier::new(&logger, target_path.clone(), true);
    for (idx, block) in blocks.iter().enumerate() {
        output.push_str(&format!("Block {}: {}\n", idx + 1, block.file.display()));
        match applier.apply_block(block) {
            Ok(result) => {
                output.push_str(&format!(
                    "  ✓ Preview match at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));
            }
            Err(e) => {
                output.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }

    output.push_str("\n💡 Preview complete. Press 'Apply Patch' to make changes.");
    Ok(output)
}

fn apply_patch(target: &str, patch: &str) -> PatchResult<String> {
    let rid = generate_rid();
    let logger = Logger::new(rid);
    let mut output = String::new();

    // Validate target
    let target_path = PathBuf::from(target);
    if !target_path.exists() || !target_path.is_dir() {
        return Err(PatchError::Validation {
            code: ErrorCode::ValidationFailed,
            message: "Target directory does not exist".to_string(),
            context: target.to_string(),
        });
    }

    // Bound input size
    if patch.len() > MAX_INPUT_SIZE {
        return Err(PatchError::Validation {
            code: ErrorCode::BoundsExceeded,
            message: format!("Input exceeds max size {}", MAX_INPUT_SIZE),
            context: "input".to_string(),
        });
    }

    // Parse blocks
    let parser = Parser::new();
    let blocks = parser.parse(patch)?;
    output.push_str(&format!("✓ Parsed {} patch block(s)\n", blocks.len()));

    // Safety: backup all target files referenced by blocks (no Git required)
    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = create_backup(&target_path, &files_to_backup)?;
    output.push_str(&format!("✓ Backup created at {}\n", backup_dir.display()));

    // Apply
    let applier = Applier::new(&logger, target_path.clone(), false);
    let mut success = 0usize;
    let mut failed = 0usize;

    for (idx, block) in blocks.iter().enumerate() {
        output.push_str(&format!("Block {}: {}\n", idx + 1, block.file.display()));
        match applier.apply_block(block) {
            Ok(result) => {
                success += 1;
                output.push_str(&format!(
                    "  ✓ Applied at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));
            }
            Err(e) => {
                failed += 1;
                output.push_str(&format!("  ❌ {}\n", e));
            }
        }
    }

    assert!(success + failed > 0, "No blocks processed");
    output.push_str(&format!("\n✅ Done. {} applied, {} failed.\n", success, failed));
    output.push_str("↩ To restore, copy files back from the backup directory.\n");

    Ok(output)
}

/* ========================== Helpers ========================== */

fn create_backup(base: &Path, files: &[PathBuf]) -> PatchResult<PathBuf> {
    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dir = base.join(format!(".applydiff_backup_{}", stamp));
    fs::create_dir_all(&dir).map_err(to_file_write_error("create_backup_dir", &dir))?;

    for rel in files {
        let src = base.join(rel);
        if !src.exists() || !src.is_file() {
            continue; // nothing to back up
        }
        let dst = dir.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(to_file_write_error("create_backup_parent", parent))?;
        }
        fs::copy(&src, &dst).map_err(to_file_write_error("backup_copy", &dst))?;
    }

    Ok(dir)
}

fn to_file_write_error(action: &'static str, path: &Path) -> impl Fn(io::Error) -> PatchError {
    let p = path.to_path_buf();
    move |e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("{} failed: {}", action, e),
        path: p.clone(),
    }
}

fn generate_rid() -> u64 {
    (Local::now().timestamp_millis() as u64) ^ (std::process::id() as u64)
}

fn append_log(ui: &MainWindow, msg: &str) {
    let mut buf = ui.get_log_output().to_string();
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf.push_str(msg);
    ui.set_log_output(buf.into());
}

fn clear_log(ui: &MainWindow) {
    ui.set_log_output("".into());
}
