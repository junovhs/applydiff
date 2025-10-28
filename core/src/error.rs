use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PatchError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // --- Session & State ---
    SessionReadFailed,
    SessionWriteFailed,
    SessionCorrupt,

    // --- Parsing ---
    ParseFailed,
    NoBlocksFound,

    // --- Application ---
    NoMatch,
    AmbiguousMatch,

    // --- File I/O ---
    FileReadFailed,
    FileWriteFailed,
    BackupFailed,

    // --- Validation ---
    ValidationFailed,
    BoundsExceeded,
    PathTraversal,
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("Session Error: {message} (path: {path:?})")]
    Session { code: ErrorCode, message: String, path: PathBuf },

    #[error("Validation Error: {message} (context: {context})")]
    Validation { code: ErrorCode, message: String, context: String },

    #[error("File Error: {message} (path: {path:?})")]
    File { code: ErrorCode, message: String, path: PathBuf },

    #[error("Parse Error: {message} (context: {context})")]
    Parse { code: ErrorCode, message: String, context: String },

    #[error("Apply Error: {message} (file: {file:?})")]
    Apply { code: ErrorCode, message: String, file: PathBuf },
}```

---
**File: `/core/src/logger.rs`**
```rust
use chrono::Utc;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Logger {
    rid: u64,
}

impl Logger {
    pub fn new(rid: u64) -> Self {
        assert!(rid > 0, "Logger rid must be non-zero");
        Self { rid }
    }

    /// Logs a structured info message to stdout.
    pub fn info(&self, subsystem: &str, action: &str, message: &str) {
        self.emit("info", subsystem, action, message);
    }

    /// Logs a structured error message to stderr.
    pub fn error(&self, subsystem: &str, action: &str, message: &str) {
        self.emit("error", subsystem, action, message);
    }

    fn emit(&self, level: &str, subsystem: &str, action: &str, message: &str) {
        let log_entry = json!({
            "ts": Utc::now().to_rfc3339(),
            "level": level,
            "rid": self.rid,
            "subsystem": subsystem,
            "action": action,
            "msg": message,
        });

        // This ensures logs are machine-parseable (JSONL format)
        // and errors go to the correct stream.
        if level == "error" {
            eprintln!("{}", log_entry);
        } else {
            println!("{}", log_entry);
        }
    }
}