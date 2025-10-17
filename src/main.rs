#![deny(warnings)]

mod apply;
mod error;
mod logger;
mod matcher;
mod parser;
mod gauntlet;
mod prompts;
mod backup;

use apply::Applier;
use error::{ErrorCode, PatchError, Result as PatchResult};
use logger::Logger;
use parser::Parser;

use chrono::Local;
use similar::TextDiff;
use std::fs;
use std::path::PathBuf;

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

    // Load Demo
    {
        let ui_handle = ui.as_weak();
        ui.on_load_demo(move || {
            let ui = ui_handle.unwrap();
            match create_demo() {
                Ok((dir, patch)) => {
                    ui.set_target_dir(dir.into());
                    ui.set_patch_input(patch.into());
                    append_log(&ui, "🎛 Demo loaded. Click 👁 Preview to see the diff, then ✓ Apply Patch.");
                }
                Err(e) => append_log(&ui, &format!("❌ Demo error: {}", e)),
            }
        });
    }

    // Preview (builds unified diff)
    {
        let ui_handle = ui.as_weak();
        ui.on_preview_patch(move || {
            let ui = ui_handle.unwrap();
            let target = ui.get_target_dir().to_string();
            let patch = ui.get_patch_input().to_string();

            if target.is_empty() || patch.is_empty() {
                append_log(&ui, "❌ Error: Please select directory and enter patch (or click 🎛 Load Demo).");
                return;
            }

            ui.set_is_processing(true);
            clear_log(&ui);
            ui.set_diff_output("".into());
            append_log(&ui, "👁 Previewing patch...\n");

            let ui_weak = ui.as_weak();
            std::thread::spawn(move || {
                let result = preview_patch(&target, &patch);
                slint::invoke_from_event_loop(move || {
                    let ui = ui_weak.unwrap();
                    match result {
                        Ok(out) => {
                            append_log(&ui, &out.log);
                            ui.set_diff_output(out.diff.into());
                        }
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
                append_log(&ui, "❌ Error: Please select directory and enter patch (or click 🎛 Load Demo).");
                return;
            }

            ui.set_is_processing(true);
            clear_log(&ui);
            ui.set_diff_output("".into());
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
            ui.set_diff_output("".into());
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

    // Copy AI Prompt
    {
        let ui_handle = ui.as_weak();
        ui.on_copy_ai_prompt(move || {
            let ui = ui_handle.unwrap();
            let prompt = prompts::build_ai_prompt();
            match copy_to_clipboard(&prompt) {
                Ok(()) => {
                    append_log(&ui, "📋 Copied AI prompt to clipboard.\n");
                    append_log(&ui, &prompt);
                }
                Err(e) => append_log(&ui, &format!("❌ Clipboard error: {}", e)),
            }
        });
    }

    // Copy Output
    {
        let ui_handle = ui.as_weak();
        ui.on_copy_output(move || {
            let ui = ui_handle.unwrap();
            let text = ui.get_log_output().to_string();
            match copy_to_clipboard(&text) {
                Ok(()) => append_log(&ui, "📋 Output copied to clipboard."),
                Err(e) => append_log(&ui, &format!("❌ Clipboard error: {}", e)),
            }
        });
    }

    ui.run()
}

/* ========================== Core operations ========================== */

struct PreviewOut { log: String, diff: String }

fn preview_patch(target: &str, patch: &str) -> PatchResult<PreviewOut> {
    let rid = generate_rid();
    let logger = Logger::new(rid);
    logger.info("ui", "preview", "start");

    let mut log = String::new();
    let mut diffs = String::new();

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
    logger.info("parser", "parsed_blocks", &format!("{}", blocks.len()));
    log.push_str(&format!("✓ Parsed {} patch block(s)\n\n", blocks.len()));

    // Dry-run & slice diffs
    let applier = Applier::new(&logger, target_path.clone(), true);
    for (idx, block) in blocks.iter().enumerate() {
        log.push_str(&format!("Block {}: {}\n", idx + 1, block.file.display()));
        match applier.apply_block(block) {
            Ok(result) => {
                log.push_str(&format!(
                    "  ✓ Preview match at offset {} (score: {:.2})\n",
                    result.matched_at, result.score
                ));

                let file_path = target_path.join(&block.file);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    let start = result.matched_at as usize;
                    let end = result.matched_end.min(content.len());
                    if start <= end {
                        let before = &content[start..end];
                        let matched_nl = if before.ends_with("\r\n") { "\r\n" }
                                         else if before.ends_with('\n') { "\n" }
                                         else { "" };
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
                        let udiff = TextDiff::from_lines(before, &to_text)
                            .unified_diff()
                            .header(&format!("a/{}", block.file.display()),
                                    &format!("b/{}", block.file.display()))
                            .to_string();
                        if !udiff.trim().is_empty() {
                            diffs.push_str(&udiff);
                            if !diffs.ends_with('\n') { diffs.push('\n'); }
                        }
                    }
                }
            }
            Err(e) => { log.push_str(&format!("  ❌ {}\n", e)); }
        }
    }

    log.push_str("\n💡 Preview complete. Press 'Apply Patch' to make changes.");
    Ok(PreviewOut { log, diff: diffs })
}

fn apply_patch(target: &str, patch: &str) -> PatchResult<String> {
    let rid = generate_rid();
    let logger = Logger::new(rid);
    logger.info("ui", "apply", "start");

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
    logger.info("parser", "parsed_blocks", &format!("{}", blocks.len()));
    output.push_str(&format!("✓ Parsed {} patch block(s)\n", blocks.len()));

    // Backup
    let files_to_backup: Vec<PathBuf> = blocks.iter().map(|b| b.file.clone()).collect();
    let backup_dir = backup::create_backup(&target_path, &files_to_backup)?;
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
    output.push_str("↩ Backups live next to your files in a timestamped .applydiff_backup_* folder.\n");

    Ok(output)
}

/* ========================== Demo + Helpers ========================== */

fn create_demo() -> Result<(String, String), String> {
    let base = std::env::temp_dir().join(format!(
        "applydiff_demo_{}",
        Local::now().format("%Y%m%d_%H%M%S")
    ));
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    let hello = base.join("hello.txt");
    let js    = base.join("web/app.js");
    let md    = base.join("docs/readme.md");

    if let Some(p) = js.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    if let Some(p) = md.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }

    fs::write(&hello, "Hello world\n").map_err(|e| e.to_string())?;
    fs::write(&js,    "function greet(){\r\n  console.log('Hello world');\r\n}\r\n").map_err(|e| e.to_string())?;
    fs::write(&md,    "# Title\r\n\r\n- item A\r\n- item B\r\n").map_err(|e| e.to_string())?;

    let patch = [
        ">>> file: hello.txt | fuzz=1.0",
        "--- from",
        "Hello world",
        "--- to",
        "Hello brave new world",
        "<<<",
        "",
        ">>> file: web/app.js | fuzz=0.85",
        "--- from",
        "  console.log('Hello world');",
        "--- to",
        "  console.log('Hello brave new world');",
        "<<<",
        "",
        ">>> file: docs/readme.md | fuzz=1.0",
        "--- from",
        "",
        "--- to",
        "## Changelog",
        "- Added greeting",
        "<<<",
    ].join("\n");

    Ok((base.display().to_string(), patch))
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

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text.to_string()).map_err(|e| e.to_string())
}
