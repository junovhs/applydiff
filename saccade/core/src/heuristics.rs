use crate::config::{CODE_BARE_PATTERN, CODE_EXT_PATTERN};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

static CODE_EXT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(CODE_EXT_PATTERN).unwrap());
static CODE_BARE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(CODE_BARE_PATTERN).unwrap());

pub struct HeuristicFilter;

impl HeuristicFilter {
    pub fn new() -> Self { Self }
    pub fn filter(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files.into_iter().filter(|path| self.should_keep(path)).collect()
    }
    fn should_keep(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        if CODE_EXT_RE.is_match(&path_str) || CODE_BARE_RE.is_match(&path_str) {
            return true;
        }
        if let Ok(entropy) = calculate_entropy(path) {
            if entropy < 3.5 || entropy > 5.5 { return false; }
        } else { return false; }
        true
    }
}

fn calculate_entropy(path: &Path) -> std::io::Result<f64> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() { return Ok(0.0); }
    let mut freq_map = HashMap::new();
    for &byte in &bytes { *freq_map.entry(byte).or_insert(0) += 1; }
    let len = bytes.len() as f64;
    let entropy = freq_map.values().fold(0.0, |acc, &count| {
        let p = count as f64 / len;
        acc - p * p.log2()
    });
    Ok(entropy)
}