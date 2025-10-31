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
    RegexError,

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
}