use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaccadeError {
    #[error("I/O error: {source} (path: {path})")]
    Io { source: std::io::Error, path: PathBuf },
    #[error("Invalid configuration: {field} = {value} ({reason})")]
    InvalidConfig { field: String, value: String, reason: String },
    #[error("Mutex lock failed: a thread panicked while holding the lock")]
    MutexPoisoned,
    #[error("Generic error: {0}")]
    Other(String),
}
pub type Result<T> = std::result::Result<T, SaccadeError>;

impl From<std::io::Error> for SaccadeError {
    fn from(source: std::io::Error) -> Self { SaccadeError::Io { source, path: PathBuf::from("<unknown>") } }
}
impl From<walkdir::Error> for SaccadeError {
    fn from(e: walkdir::Error) -> Self { SaccadeError::Other(e.to_string()) }
}