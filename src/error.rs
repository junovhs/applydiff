use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PatchError>;

#[derive(Debug, Clone)]
pub enum ErrorCode {
    ValidationFailed,
    BoundsExceeded,
    FileReadFailed,
    FileWriteFailed,
    ParseFailed,
    NoMatch,
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
