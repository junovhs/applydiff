use crate::error::{Result, SaccadeError};
use crate::parser;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::panic;

pub struct Stage2Generator { verbose: bool }
type ParseResult = (PathBuf, String);

impl Stage2Generator {
    pub fn new() -> Self { Self { verbose: false } }
    pub fn with_verbose(mut self, verbose: bool) -> Self { self.verbose = verbose; self }

    pub fn generate(&self, files: &[PathBuf], output_path: &Path) -> Result<Option<String>> {
        let results = Mutex::new(Vec::new());
        files.par_iter().for_each(|file_path| {
            if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
                if let Ok(content) = fs::read_to_string(file_path) {
                    if let Some(skeleton) = parser::skeletonize_file(&content, ext) {
                        results.lock().unwrap().push((file_path.clone(), skeleton));
                    }
                }
            }
        });
        let mut final_results = results.into_inner().map_err(|_| SaccadeError::MutexPoisoned)?;
        if final_results.is_empty() { return Ok(None); }
        final_results.sort_by(|a, b| a.0.cmp(&b.0));
        let mut xml = String::from("<?xml version=\"1.0\"?>\n<files>\n");
        for (path, skeleton) in final_results {
            xml.push_str(&format!("  <file path=\"{}\">\n    {}\n  </file>\n", path.display(), skeleton.replace('&', "&amp;").replace('<', "&lt;")));
        }
        xml.push_str("</files>\n");
        fs::write(output_path, xml).map_err(|e| SaccadeError::Io { source: e, path: output_path.to_path_buf() })?;
        Ok(Some(format!("Wrote skeleton to {}", output_path.display())))
    }
}