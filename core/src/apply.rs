use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use crate::matcher::find_best_match;
use crate::parser::PatchBlock;

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, PathBuf};

pub struct ApplyResult {
    pub matched_at: usize,
    pub matched_end: usize,
    pub score: f64,
}

pub struct Applier<'a> {
    #[allow(dead_code)]
    logger: &'a Logger,
    root: PathBuf,
    dry_run: bool,
}

impl<'a> Applier<'a> {
    pub fn new(logger: &'a Logger, root: PathBuf, dry_run: bool) -> Self {
        Self { logger, root, dry_run }
    }

    pub fn apply_block(&self, blk: &PatchBlock) -> Result<ApplyResult> {
        // harden: disallow absolute paths or '..' traversal
        if blk.file.is_absolute() || blk.file.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(PatchError::Validation {
                code: ErrorCode::ValidationFailed,
                message: "Patch path escapes target directory".to_string(),
                context: blk.file.display().to_string(),
            });
        }

        let path = self.root.join(&blk.file);

        // read file, allow append-create if FROM is empty
        let content = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                if blk.from.trim().is_empty() && e.kind() == ErrorKind::NotFound {
                    String::new()
                } else {
                    return Err(PatchError::File {
                        code: ErrorCode::FileReadFailed,
                        message: format!("Failed to read {}: {}", blk.file.display(), e),
                        path: path.clone(),
                    });
                }
            }
        };

        // append/create when FROM is empty
        if blk.from.trim().is_empty() {
            let mut new_content = content.clone();

            // FIX: only insert a separator newline when appending to a NON-empty file that lacks one.
            if !new_content.is_empty() && !new_content.ends_with('\n') && !blk.to.is_empty() {
                new_content.push('\n');
            }
            new_content.push_str(&blk.to);

            if !self.dry_run {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| PatchError::File {
                        code: ErrorCode::FileWriteFailed,
                        message: format!("Failed to create parent dir for {}: {}", blk.file.display(), e),
                        path: parent.to_path_buf(),
                    })?;
                }
                fs::write(&path, new_content).map_err(|e| PatchError::File {
                    code: ErrorCode::FileWriteFailed,
                    message: format!("Failed to write {}: {}", blk.file.display(), e),
                    path: path.clone(),
                })?;
            }

            let at = content.len();
            return Ok(ApplyResult { matched_at: at, matched_end: at, score: 1.0 });
        }

        // find match (exact or fuzzy)
        let Some(m) = find_best_match(&content, &blk.from, blk.fuzz, self.logger) else {
            return Err(PatchError::Apply {
                code: ErrorCode::NoMatch,
                message: "Could not match block: either no suitable match found, or multiple ambiguous matches detected. Check logs for details.".to_string(),
                file: blk.file.clone(),
            });
        };

        // harmonize EOL with matched slice
        let matched_slice = &content[m.start..m.end];
        let matched_nl = if matched_slice.ends_with("\r\n") {
            "\r\n"
        } else if matched_slice.ends_with('\n') {
            "\n"
        } else {
            ""
        };

        let mut to_text = blk.to.clone();
        if !matched_nl.is_empty() {
            if to_text.ends_with("\r\n") && matched_nl == "\n" {
                to_text.truncate(to_text.len().saturating_sub(2));
                to_text.push('\n');
            } else if to_text.ends_with('\n') && matched_nl == "\r\n" {
                to_text.pop();
                to_text.push_str("\r\n");
            } else if !to_text.ends_with('\n') && !to_text.ends_with("\r\n") {
                to_text.push_str(matched_nl);
            }
        }

        let mut new_content = String::new();
        new_content.push_str(&content[..m.start]);
        new_content.push_str(&to_text);
        new_content.push_str(&content[m.end..]);

        if !self.dry_run {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| PatchError::File {
                    code: ErrorCode::FileWriteFailed,
                    message: format!("Failed to create parent dir for {}: {}", blk.file.display(), e),
                    path: parent.to_path_buf(),
                })?;
            }
            fs::write(&path, new_content).map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("Failed to write {}: {}", blk.file.display(), e),
                path: path.clone(),
            })?;
        }

        Ok(ApplyResult { matched_at: m.start, matched_end: m.end, score: m.score })
    }
}
