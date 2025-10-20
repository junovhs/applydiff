use crate::error::{ErrorCode, PatchError, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub fn make_sandbox() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "applydiff_gauntlet_{}",
        Local::now().format("%Y%m%d_%H%M%S%f")
    ));
    fs::create_dir_all(&dir).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("create sandbox failed: {e}"),
        path: dir.clone(),
    })?;
    Ok(dir)
}

pub fn cleanup(dir: &Path) -> Result<()> {
    fs::remove_dir_all(dir).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("remove sandbox failed: {e}"),
        path: dir.to_path_buf(),
    })
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn verify_dirs_match(
    log: &mut String,
    actual_dir: &Path,
    expected_dir: &Path
) -> std::result::Result<(), String> {
    let entries = match fs::read_dir(expected_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // No 'after' dir means no verification needed
    };

    for entry in entries.flatten() {
        let expected_path = entry.path();
        if expected_path.is_file() {
            let rel_path = expected_path.strip_prefix(expected_dir).unwrap();
            let actual_path = actual_dir.join(rel_path);

            if !actual_path.exists() {
                return Err(format!("Expected file '{}' not found in sandbox.", rel_path.display()));
            }

            let expected_bytes = fs::read(&expected_path).unwrap();
            let actual_bytes = fs::read(&actual_path).unwrap();

            if expected_bytes != actual_bytes {
                logln(log, format!("    ❌ File mismatch: {}", rel_path.display()));
                return Err(format!("Content of '{}' does not match expected.", rel_path.display()));
            } else {
                logln(log, format!("    ✓ File verified: {}", rel_path.display()));
            }
        }
    }
    Ok(())
}

pub fn case_header(log: &mut String, name: &str) {
    logln(log, format!("\n— Testing: {} —", name));
}

pub fn logln<S: Into<String>>(buf: &mut String, s: S) {
    if !buf.is_empty() && !buf.ends_with('\n') {
        buf.push('\n');
    }
    buf.push_str(&s.into());
}