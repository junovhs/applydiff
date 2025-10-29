use crate::error::{ErrorCode, PatchError, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a timestamped backup of specified files within a base directory.
pub fn create_backup(base: &Path, files_to_backup: &[PathBuf]) -> Result<PathBuf> {
    assert!(base.is_dir(), "Backup base must be a directory");

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = base.join(format!(".applydiff_backup_{}", timestamp));

    fs::create_dir_all(&backup_dir).map_err(|e| PatchError::File {
        code: ErrorCode::BackupFailed,
        message: format!("Failed to create backup directory: {}", e),
        path: backup_dir.clone(),
    })?;

    for relative_path in files_to_backup {
        let source_path = base.join(relative_path);
        if !source_path.exists() {
            continue; // It's not an error if a file to be patched doesn't exist yet.
        }

        let dest_path = backup_dir.join(relative_path);
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|e| PatchError::File {
                code: ErrorCode::BackupFailed,
                message: format!(
                    "Failed to create parent directory for backup item: {}",
                    e
                ),
                path: parent.to_path_buf(),
            })?;
        }

        fs::copy(&source_path, &dest_path).map_err(|e| PatchError::File {
            code: ErrorCode::BackupFailed,
            message: format!("Failed to copy file to backup directory: {}", e),
            path: source_path,
        })?;
    }

    Ok(backup_dir)
}