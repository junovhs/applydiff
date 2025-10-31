use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionState {
    pub last_refresh_ts: DateTime<Utc>,
    pub exchange_count: u32,
    pub total_errors: u32,
    pub file_metrics: HashMap<PathBuf, FileMetrics>,
    pub keystone_files: Vec<PathBuf>,
    #[serde(skip)]
    pub project_root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetrics {
    pub original_hash: String,
    pub patch_count: u32,
}

impl SessionState {
    /// Creates a new, empty session state for a given project root.
    #[must_use]
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            last_refresh_ts: Utc::now(),
            exchange_count: 0,
            total_errors: 0,
            file_metrics: HashMap::new(),
            keystone_files: Vec::new(),
            project_root,
        }
    }
}