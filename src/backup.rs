use crate::error::{ErrorCode, PatchError, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_backup(base: &Path, files: &[PathBuf]) -> Result<PathBuf> {
    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dir = base.join(format!(".applydiff_backup_{}", stamp));
    fs::create_dir_all(&dir).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("create backup dir failed: {}", e),
        path: dir.clone(),
    })?;

    for rel in files {
        let src = base.join(rel);
        if !src.exists() || !src.is_file() {
            continue;
        }
        let dst = dir.join(rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("create parent dir failed: {}", e),
                path: parent.to_path_buf(),
            })?;
        }
        fs::copy(&src, &dst).map_err(|e| PatchError::File {
            code: ErrorCode::FileWriteFailed,
            message: format!("backup copy failed: {}", e),
            path: dst.clone(),
        })?;
    }

    Ok(dir)
}

#[allow(dead_code)] // UI feature for this is not currently connected
pub fn latest_backup(base: &Path) -> Option<PathBuf> {
    let entries = match fs::read_dir(base) {
        Ok(v) => v,
        Err(_) => return None,
    };

    let mut best: Option<(String, PathBuf)> = None;
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.starts_with(".applydiff_backup_") {
                    let key = name.to_string();
                    if best.as_ref().map(|(k, _)| &key > k).unwrap_or(true) {
                        best = Some((key, p));
                    }
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

#[allow(dead_code)] // UI feature for this is not currently connected
pub fn restore_backup(base: &Path, backup_root: &Path) -> Result<()> {
    // Recursively copy files from backup_root back into base.
    fn walk_copy(base: &Path, root: &Path, cur: &Path) -> Result<()> {
        for ent in fs::read_dir(cur).map_err(|e| PatchError::File {
            code: ErrorCode::FileWriteFailed,
            message: format!("read_dir failed: {}", e),
            path: cur.to_path_buf(),
        })? {
            let ent = ent.map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("read_dir entry failed: {}", e),
                path: cur.to_path_buf(),
            })?;
            let p = ent.path();
            if p.is_dir() {
                walk_copy(base, root, &p)?;
            } else if p.is_file() {
                let rel = p.strip_prefix(root).unwrap_or(&p);
                let dst = base.join(rel);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent).map_err(|e| PatchError::File {
                        code: ErrorCode::FileWriteFailed,
                        message: format!("mkdir for restore failed: {}", e),
                        path: parent.to_path_buf(),
                    })?;
                }
                fs::copy(&p, &dst).map_err(|e| PatchError::File {
                    code: ErrorCode::FileWriteFailed,
                    message: format!("restore copy failed: {}", e),
                    path: dst,
                })?;
            }
        }
        Ok(())
    }
    walk_copy(base, backup_root, backup_root)
}