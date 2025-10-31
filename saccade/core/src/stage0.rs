use crate::config::Config;
use crate::error::Result;
use std::collections::{BTreeSet, BTreeMap};
use std::fs;

pub struct Stage0Generator { config: Config }

impl Stage0Generator {
    pub fn new(config: Config) -> Self { Self { config } }
    pub fn generate_combined_structure(&self, files: &[std::path::PathBuf], detected_systems: &[crate::detection::BuildSystemType]) -> Result<String> {
        let mut output = String::new();
        output.push_str("DIRECTORY TREE\n----------------\n");
        let mut dirs = BTreeSet::new();
        for path in files {
            if let Some(parent) = path.parent() {
                let comps: Vec<_> = parent.components().collect();
                for i in 1..=comps.len().min(self.config.max_depth) {
                    let dir_path: std::path::PathBuf = comps[..i].iter().collect();
                    dirs.insert(dir_path.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        for dir in &dirs { output.push_str(&format!("{}\n", dir)); }

        output.push_str("\nFILE INDEX\n----------\n");
        let mut sorted: Vec<String> = files.iter().map(|p| p.to_string_lossy().replace('\\', "/")).collect();
        sorted.sort();
        for file in &sorted { output.push_str(&format!("{}\n", file)); }
        Ok(output)
    }
    pub fn generate_languages(&self, files: &[std::path::PathBuf]) -> Result<String> {
        let mut ext_counts: BTreeMap<String, usize> = BTreeMap::new();
        for path in files {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("(noext)").to_string();
            *ext_counts.entry(ext).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = ext_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let mut output = String::new();
        for (ext, count) in sorted { output.push_str(&format!("- .{}: {}\n", ext, count)); }
        Ok(output)
    }
}