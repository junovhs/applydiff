use crate::detection::BuildSystemType;
use crate::error::Result;
use std::path::PathBuf;

pub struct Stage1Generator;

impl Stage1Generator {
    pub fn new() -> Self { Self }
    pub fn generate_combined_apis(&self, _rust_crates: &[PathBuf], _frontend_dirs: &[PathBuf], _file_index: &[PathBuf]) -> Result<String> { Ok(String::new()) }
    pub fn find_rust_crates(&self) -> Result<Vec<PathBuf>> { Ok(vec![]) }
    pub fn find_frontend_dirs(&self) -> Result<Vec<PathBuf>> { Ok(vec![]) }
    pub fn generate_all_deps(&self, _detected_systems: &[BuildSystemType]) -> Result<String> { Ok(String::new()) }
}