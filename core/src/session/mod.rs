use crate::error::{ErrorCode, PatchError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod prompts;

const SESSION_FILE_NAME: &str = ".applydiff_session.json";
const MAX_SESSION_FILE_SIZE: u64 = 5_000_000; // 5 MB limit

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetrics {
    pub original_hash: String,
    pub patch_count: u32,
    pub percent_changed: f32,
    #[serde(default)] // For backwards compatibility
    pub is_keystone: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionState {
    pub version: u32,
    pub last_modified: DateTime<Utc>,
    pub exchange_count: u32,
    pub total_errors: u32,
    pub files: HashMap<PathBuf, FileMetrics>,
}

impl SessionState {
    /// Creates a new, empty session state.
    pub fn new() -> Self {
        SessionState {
            version: 1,
            last_modified: Utc::now(),
            exchange_count: 0,
            total_errors: 0,
            files: HashMap::new(),
        }
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Session {
    pub state: SessionState,
    project_root: PathBuf,
    session_file_path: PathBuf,
}

impl Session {
    /// Loads a session from the project root, or creates a new one if none exists.
    pub fn load(project_root: &Path) -> Result<Self> {
        assert!(project_root.is_dir(), "Session project_root must be a directory");
        let session_file_path = project_root.join(SESSION_FILE_NAME);

        let state = if session_file_path.exists() {
            // File size guard
            let metadata = fs::metadata(&session_file_path).map_err(|e| PatchError::Session {
                code: ErrorCode::SessionReadFailed,
                message: format!("Could not read session file metadata: {}", e),
                path: session_file_path.clone(),
            })?;
            if metadata.len() > MAX_SESSION_FILE_SIZE {
                return Err(PatchError::Session {
                    code: ErrorCode::BoundsExceeded,
                    message: "Session file size exceeds limit".to_string(),
                    path: session_file_path,
                });
            }

            let content = fs::read_to_string(&session_file_path).map_err(|e| PatchError::Session {
                code: ErrorCode::SessionReadFailed,
                message: format!("Could not read session file: {}", e),
                path: session_file_path.clone(),
            })?;

            serde_json::from_str(&content).map_err(|e| PatchError::Session {
                code: ErrorCode::SessionCorrupt,
                message: format!("Could not parse session file: {}", e),
                path: session_file_path.clone(),
            })?
        } else {
            SessionState::new()
        };

        Ok(Session {
            state,
            project_root: project_root.to_path_buf(),
            session_file_path,
        })
    }

    /// Saves the current session state to disk.
    pub fn save(&mut self) -> Result<()> {
        self.state.last_modified = Utc::now();
        let content = serde_json::to_string_pretty(&self.state).map_err(|e| PatchError::Session {
            code: ErrorCode::SessionWriteFailed,
            message: format!("Could not serialize session state: {}", e),
            path: self.session_file_path.clone(),
        })?;

        fs::write(&self.session_file_path, content).map_err(|e| PatchError::Session {
            code: ErrorCode::SessionWriteFailed,
            message: format!("Could not write to session file: {}", e),
            path: self.session_file_path.clone(),
        })
    }

    /// Increments the Prediction Error count.
    pub fn record_error(&mut self) {
        self.state.total_errors = self.state.total_errors.saturating_add(1);
    }

    /// Records a successful patch application for a given file.
    pub fn record_success(&mut self, file: &Path, original_content: &str, new_content: &str) {
        let metrics = self.state.files.entry(file.to_path_buf()).or_insert_with(|| {
            FileMetrics {
                original_hash: format!("{:x}", md5::compute(original_content)),
                patch_count: 0,
                percent_changed: 0.0,
                is_keystone: false, // This would be populated by Saccade integration later
            }
        });

        metrics.patch_count = metrics.patch_count.saturating_add(1);

        if !original_content.is_empty() {
            let diff = similar::TextDiff::from_lines(original_content, new_content);
            let num_different_lines = diff.ops().iter().filter(|op| op.tag() != similar::ChangeTag::Equal).map(|op| op.new_range().len()).sum::<usize>();
            let total_lines = new_content.lines().count().max(1); // Avoid division by zero
            metrics.percent_changed = (num_different_lines as f32 / total_lines as f32) * 100.0;
        } else if !new_content.is_empty() {
            metrics.percent_changed = 100.0; // Created a new file
        }
    }

    /// Generates the proactive guidance briefing for the AI.
    pub fn generate_briefing(&mut self) -> String {
        self.state.exchange_count = self.state.exchange_count.saturating_add(1);
        prompts::build_ai_prompt(&self.state)
    }

    /// Resets counters for a new "checkpoint".
    pub fn refresh_session(&mut self) {
        self.state.exchange_count = 0;
        self.state.total_errors = 0;
        // In the future, this would also archive the old session file.
    }
}