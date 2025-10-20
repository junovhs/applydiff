use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PatchError>;

#[derive(Debug, Clone)]
pub enum ErrorCode {
    // Parse / matching
    ParseFailed,
    NoMatch,

    // File I/O
    FileReadFailed,
    FileWriteFailed,

    // Validation / bounds
    ValidationFailed,
    BoundsExceeded,
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("{message} (context: {context})")]
    Validation { code: ErrorCode, message: String, context: String },

    #[error("{message} (file: {path:?})")]
    File { code: ErrorCode, message: String, path: PathBuf },

    #[error("{message} (context: {context})")]
    Parse { code: ErrorCode, message: String, context: String },

    #[error("{message} (file: {file:?})")]
    Apply { code: ErrorCode, message: String, file: PathBuf },
}
