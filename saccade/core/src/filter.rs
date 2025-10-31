use crate::config::{Config, BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, SECRET_PATTERN};
use crate::error::Result;
use regex::Regex;
use std::path::Path;

pub struct FileFilter {
    config: Config,
    bin_ext_re: Regex,
    secret_re: Regex,
    code_ext_re: Option<Regex>,
    code_bare_re: Option<Regex>,
}

impl FileFilter {
    pub fn new(config: Config) -> Result<Self> {
        let (code_ext_re, code_bare_re) = if config.code_only {
            (Some(Regex::new(CODE_EXT_PATTERN)?), Some(Regex::new(CODE_BARE_PATTERN)?))
        } else { (None, None) };
        Ok(Self {
            config,
            bin_ext_re: Regex::new(BIN_EXT_PATTERN)?,
            secret_re: Regex::new(SECRET_PATTERN)?,
            code_ext_re, code_bare_re,
        })
    }
    pub fn filter(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files.into_iter().filter(|p| self.should_keep(p)).collect()
    }
    fn should_keep(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().replace('\\', "/");
        if self.secret_re.is_match(&path_str) { return false; }
        if self.bin_ext_re.is_match(&path_str) { return false; }
        if self.config.exclude_patterns.iter().any(|p| p.is_match(&path_str)) { return false; }
        if !self.config.include_patterns.is_empty() && !self.config.include_patterns.iter().any(|p| p.is_match(&path_str)) { return false; }
        if let (Some(ext_re), Some(bare_re)) = (&self.code_ext_re, &self.code_bare_re) {
            if !(ext_re.is_match(&path_str) || bare_re.is_match(&path_str)) { return false; }
        }
        true
    }
}