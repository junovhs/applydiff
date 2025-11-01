use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaccadeError {
    #[error("IO error at {path}: {source}")]
    Io {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Invalid configuration: {field} = {value} ({reason})")]
    InvalidConfig {
        field: String,
        value: String,
        reason: String,
    },
    #[error("Mutex poisoned")]
    MutexPoisoned,
    #[error("Other error: {0}")]
    Other(String),
}

impl From<std::io::Error> for SaccadeError {
    fn from(err: std::io::Error) -> Self {
        SaccadeError::Io {
            source: err,
            path: PathBuf::from("<unknown>"),
        }
    }
}

pub type Result<T> = std::result::Result<T, SaccadeError>;