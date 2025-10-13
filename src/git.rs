use crate::error::{ErrorCode, PatchError, Result};
use crate::logger::Logger;
use git2::{Repository, StatusOptions};
use std::path::Path;

pub struct GitGuard<'a> {
    logger: &'a Logger,
}

impl<'a> GitGuard<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self { logger }
    }
    
    pub fn check_repo(&self, path: &Path) -> Result<Repository> {
        // Pre: path exists
        assert!(path.exists(), "Path must exist");
        
        self.logger.info("git", "check_repo", "Checking if directory is a git repo");
        
        let repo = Repository::discover(path).map_err(|e| {
            self.logger.error(
                "git",
                "check_repo",
                ErrorCode::GitNotRepo.as_u32(),
                "Not a git repository",
                Some(serde_json::json!({ "error": e.to_string() })),
            );
            PatchError::Git {
                code: ErrorCode::GitNotRepo,
                message: "Directory is not a git repository".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        // Post: repository opened successfully
        self.logger.info("git", "check_repo", "Repository found");
        Ok(repo)
    }
    
    pub fn ensure_clean(&self, repo: &Repository) -> Result<()> {
        // Pre: repo is valid
        self.logger.info("git", "ensure_clean", "Checking working tree status");
        
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);
        
        let statuses = repo.statuses(Some(&mut opts)).map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitDirtyState,
                message: "Failed to get repository status".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        // Bounded loop: at most statuses.len() iterations
        let dirty_count = statuses.iter()
            .filter(|s| !s.status().is_ignored())
            .count();
        
        if dirty_count > 0 {
            self.logger.error(
                "git",
                "ensure_clean",
                ErrorCode::GitDirtyState.as_u32(),
                "Working tree has uncommitted changes",
                Some(serde_json::json!({ "dirty_files": dirty_count })),
            );
            
            return Err(PatchError::Git {
                code: ErrorCode::GitDirtyState,
                message: format!(
                    "Repository has {} uncommitted changes. Commit or stash first.",
                    dirty_count
                ),
                detail: "Run 'git status' to see changes".to_string(),
            });
        }
        
        self.logger.info("git", "ensure_clean", "Working tree is clean");
        Ok(())
    }
    
    pub fn create_safety_commit(&self, repo: &Repository) -> Result<String> {
        // Pre: repo clean (checked by caller)
        self.logger.info("git", "create_commit", "Creating pre-patch safety commit");
        
        let mut index = repo.index().map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to get index".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| {
                PatchError::Git {
                    code: ErrorCode::GitCommitFailed,
                    message: "Failed to stage changes".to_string(),
                    detail: e.to_string(),
                }
            })?;
        
        let tree_id = index.write_tree().map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to write tree".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let tree = repo.find_tree(tree_id).map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to find tree".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let head = repo.head().map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to get HEAD".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let parent_commit = head.peel_to_commit().map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to get parent commit".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let sig = repo.signature().map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to create signature".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "[applydiff] Pre-patch safety commit",
            &tree,
            &[&parent_commit],
        ).map_err(|e| {
            PatchError::Git {
                code: ErrorCode::GitCommitFailed,
                message: "Failed to create commit".to_string(),
                detail: e.to_string(),
            }
        })?;
        
        let oid_str = commit_id.to_string();
        
        self.logger.info(
            "git",
            "create_commit",
            &format!("Safety commit created: {}", &oid_str[..8]),
        );
        
        // Post: commit ID is valid hex string
        assert_eq!(oid_str.len(), 40, "Invalid commit OID length");
        
        Ok(oid_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_not_a_repo() {
        let logger = Logger::new(1);
        let guard = GitGuard::new(&logger);
        let tmp = TempDir::new().unwrap();
        
        let result = guard.check_repo(tmp.path());
        assert!(result.is_err());
    }
}