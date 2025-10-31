use super::r#match::find_best_match;
use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use crate::parse::{PatchBlock, PatchMode};
use regex::Regex;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct ApplyResult {
    pub matched_at: usize,
    pub matched_end: usize,
    pub score: f64,
}

pub struct Applier<'a> {
    logger: &'a Logger,
    project_root: PathBuf,
    dry_run: bool,
}

/// Helper to write file and create parent directories if needed.
fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| PatchError::File {
                code: ErrorCode::FileWriteFailed,
                message: format!("Failed to create parent directories: {e}"),
                path: parent.to_path_buf(),
            })?;
        }
    }
    fs::write(path, content).map_err(|e| PatchError::File {
        code: ErrorCode::FileWriteFailed,
        message: format!("Failed to write to file: {e}"),
        path: path.to_path_buf(),
    })
}

impl<'a> Applier<'a> {
    /// Creates a new `Applier`.
    ///
    /// # Panics
    ///
    /// Panics if `project_root` is not a directory.
    #[must_use]
    pub fn new(logger: &'a Logger, project_root: PathBuf, dry_run: bool) -> Self {
        assert!(
            project_root.is_dir(),
            "Applier project_root must be a directory"
        );
        Self {
            logger,
            project_root,
            dry_run,
        }
    }

    /// Applies a single patch block to the corresponding file.
    ///
    /// # Errors
    ///
    /// Returns an error if path traversal is detected, if the file cannot be
    /// read, or if the patch application itself fails.
    pub fn apply_block(&self, block: &PatchBlock) -> Result<ApplyResult> {
        if block.file.components().any(|c| matches!(c, Component::ParentDir))
            || block.file.is_absolute()
        {
            return Err(PatchError::Validation {
                code: ErrorCode::PathTraversal,
                message: "Patch contains a path that escapes the project directory".to_string(),
                context: block.file.display().to_string(),
            });
        }
        let target_path = self.project_root.join(&block.file);

        let original_content = match fs::read_to_string(&target_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Err(PatchError::File {
                    code: ErrorCode::FileReadFailed,
                    message: format!("Failed to read target file: {e}"),
                    path: target_path,
                });
            }
        };

        match block.mode {
            PatchMode::Classic => self.apply_classic_block(block, &original_content, &target_path),
            PatchMode::Replace => self.apply_replace_block(block, &original_content, &target_path),
            PatchMode::Regex => self.apply_regex_block(block, &original_content, &target_path),
        }
    }

    fn apply_classic_block(
        &self,
        block: &PatchBlock,
        original_content: &str,
        target_path: &Path,
    ) -> Result<ApplyResult> {
        let match_result =
            match find_best_match(original_content, &block.from, block.fuzz, self.logger) {
                Ok(res) => res,
                Err(PatchError::Apply { code, message, .. }) => {
                    return Err(PatchError::Apply { code, message, file: block.file.clone() });
                }
                Err(e) => return Err(e),
            };

        let mut new_content = String::with_capacity(original_content.len() + block.to.len());
        new_content.push_str(&original_content[..match_result.start_byte]);
        new_content.push_str(&block.to);
        new_content.push_str(&original_content[match_result.end_byte..]);

        if !self.dry_run {
            write_file(target_path, &new_content)?;
        }

        Ok(ApplyResult {
            matched_at: match_result.start_byte,
            matched_end: match_result.end_byte,
            score: match_result.score,
        })
    }

    fn apply_replace_block(
        &self,
        block: &PatchBlock,
        original_content: &str,
        target_path: &Path,
    ) -> Result<ApplyResult> {
        if !self.dry_run {
            write_file(target_path, &block.to)?;
        }
        Ok(ApplyResult {
            matched_at: 0,
            matched_end: original_content.len(),
            score: 1.0,
        })
    }

    fn apply_regex_block(
        &self,
        block: &PatchBlock,
        original_content: &str,
        target_path: &Path,
    ) -> Result<ApplyResult> {
        let re = Regex::new(&block.from).map_err(|e| PatchError::Apply {
            code: ErrorCode::RegexError,
            message: format!("Invalid regex pattern: {e}"),
            file: block.file.clone(),
        })?;

        let new_content = re.replace_all(original_content, &block.to[..]);

        if !self.dry_run {
            write_file(target_path, &new_content)?;
        }

        Ok(ApplyResult {
            matched_at: 0,
            matched_end: original_content.len(),
            score: 1.0,
        })
    }
}