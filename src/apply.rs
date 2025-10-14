use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use crate::matcher::find_best_match;
use crate::parser::PatchBlock;

use std::fs;
use std::path::PathBuf;

pub struct ApplyResult {
    pub matched_at: usize,
    pub score: f32,
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
        let path = self.root.join(&blk.file);
        let content = fs::read_to_string(&path).map_err(|e| PatchError::File {
            code: ErrorCode::FileReadFailed,
            message: format!("Failed to read {}: {}", blk.file.display(), e),
            path: path.clone(),
        })?;

        // Append-only if "from" is empty
        if blk.from.trim().is_empty() {
            let mut new_content = content.clone();
            if !new_content.ends_with('\n') && !blk.to.is_empty() {
                new_content.push('\n');
            }
            new_content.push_str(&blk.to);
            if !self.dry_run {
                fs::write(&path, new_content).map_err(|e| PatchError::File {
                    code: ErrorCode::FileWriteFailed,
                    message: format!("Failed to write {}: {}", blk.file.display(), e),
                    path: path.clone(),
                })?;
            }
            return Ok(ApplyResult { matched_at: content.len(), score: 1.0 });
        }

        // Find best match (exact or fuzzy)
        let Some(m) = find_best_match(&content, &blk.from, blk.fuzz) else {
            return Err(PatchError::Apply {
                code: ErrorCode::NoMatch,
                message: format!("No match >= {:.2} for block", blk.fuzz),
                file: blk.file.clone(),
            });
        };

        // Harmonize the replacement's trailing EOL with the matched slice's EOL (CRLF/LF)
        let matched_slice = &content[m.start..m.end];
        let matched_nl = if matched_slice.ends_with("\r\n") { "\r\n" }
                         else if matched_slice.ends_with('\n') { "\n" }
                         else { "" };

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
            fs::write(&path, new_content).map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("Failed to write {}: {}", blk.file.display(), e),
                path: path.clone(),
            })?;
        }

        Ok(ApplyResult { matched_at: m.start, score: m.score })
    }
}
