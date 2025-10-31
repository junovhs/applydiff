use crate::config::{Config, PRUNE_DIRS};
use crate::error::{Result, SaccadeError};
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub struct FileEnumerator { config: Config }

impl FileEnumerator {
    pub fn new(config: Config) -> Self { Self { config } }
    pub fn enumerate(&self) -> Result<Vec<PathBuf>> {
        use crate::config::GitMode;
        match self.config.git_mode {
            GitMode::Yes => self.git_ls_files(),
            GitMode::No => self.walk_all_files(),
            GitMode::Auto => {
                if self.in_git_repo()? { self.git_ls_files() }
                else { self.walk_all_files() }
            }
        }
    }
    fn in_git_repo(&self) -> Result<bool> {
        Ok(Command::new("git").arg("rev-parse").arg("--is-inside-work-tree").output().map(|o| o.status.success()).unwrap_or(false))
    }
    fn git_ls_files(&self) -> Result<Vec<PathBuf>> {
        let out = Command::new("git").arg("ls-files").arg("-z").arg("--exclude-standard").output()?;
        if !out.status.success() { return Err(SaccadeError::Other("git ls-files failed".to_string())); }
        Ok(out.stdout.split(|b| *b == 0).filter(|s| !s.is_empty()).map(|s| PathBuf::from(String::from_utf8_lossy(s).as_ref())).collect())
    }
    fn walk_all_files(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        let walker = WalkDir::new(".").follow_links(false).into_iter();
        for item in walker.filter_entry(|e| !PRUNE_DIRS.iter().any(|p| e.file_name() == *p)) {
            let entry = item?;
            if entry.file_type().is_file() {
                paths.push(entry.path().strip_prefix(".").unwrap_or(entry.path()).to_path_buf());
            }
        }
        Ok(paths)
    }
}