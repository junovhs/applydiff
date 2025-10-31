use crate::error::{Result, SaccadeError};
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum GitMode { Auto, Yes, No }

#[derive(Debug, Clone)]
pub struct Config {
    pub pack_dir: PathBuf,
    pub max_depth: usize,
    pub git_mode: GitMode,
    pub include_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<Regex>,
    pub code_only: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            pack_dir: PathBuf::from("ai-pack"),
            max_depth: 3,
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            dry_run: false,
            verbose: false,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.max_depth < 1 || self.max_depth > 10 {
            return Err(SaccadeError::InvalidConfig {
                field: "max_depth".to_string(),
                value: self.max_depth.to_string(),
                reason: "must be between 1 and 10".to_string(),
            });
        }
        Ok(())
    }

    pub fn parse_patterns(input: &str) -> Result<Vec<Regex>> {
        input.split(',').filter(|s| !s.is_empty()).map(|s| Regex::new(s.trim()).map_err(Into::into)).collect()
    }
}

impl Default for Config {
    fn default() -> Self { Self::new() }
}

pub const PRUNE_DIRS: &[&str] = &[".git", "node_modules", "dist", "build", "target", "vendor"];
pub const BIN_EXT_PATTERN: &str = r"(?i)\.(png|jpe?g|gif|svg|ico|webp|woff2?|ttf|otf|pdf|mp4|mov|zip|gz|rar|bin|exe|dll|so|dylib)$";
pub const SECRET_PATTERN: &str = r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa|id_ed25519|.*\.(pem|p12|jks|keystore|pfx))$)";
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(c|h|cpp|hpp|rs|go|py|js|jsx|ts|tsx|java|kt|rb|php|sh|sql|html|xml|yaml|yml|toml|json|md)$";
pub const CODE_BARE_PATTERN: &str = r"(?i)(Makefile|Dockerfile|CMakeLists\.txt)$";