use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("No files match pattern: {0}")]
    NoMatches(String),
    #[error("Invalid glob pattern: {0}")]
    InvalidPattern(String),
    #[error("Invalid line range: {0}")]
    InvalidLineRange(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, RequestError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestFile {
    #[serde(flatten)]
    pub target: RequestTarget,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RequestRange>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestTarget {
    SinglePath { path: String },
    Pattern { pattern: String },
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestRange {
    Lines { lines: String },
    Symbol { symbol: String },
}
#[derive(Debug)]
pub struct ResolvedRequest {
    pub files: Vec<FileContent>,
    pub reason: String,
}
#[derive(Debug)]
pub struct FileContent {
    pub path: PathBuf,
    pub content: String,
    pub range_info: Option<String>,
}

impl RequestFile {
    pub fn resolve(&self, available_files: &[PathBuf], base_dir: &Path) -> Result<ResolvedRequest> {
        let matching_paths = self.find_matching_files(available_files)?;
        let files = matching_paths
            .into_iter()
            .filter_map(|p| self.read_file_with_range(&base_dir.join(&p), &p).ok())
            .collect();
        Ok(ResolvedRequest {
            files,
            reason: self.reason.clone(),
        })
    }

    fn find_matching_files(&self, available_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        match &self.target {
            RequestTarget::SinglePath { path } => {
                let p = PathBuf::from(path);
                if available_files.contains(&p) {
                    Ok(vec![p])
                } else {
                    Err(RequestError::FileNotFound(path.clone()))
                }
            }
            RequestTarget::Pattern { pattern } => {
                let glob =
                    Pattern::new(pattern).map_err(|e| RequestError::InvalidPattern(e.to_string()))?;
                let matches: Vec<_> =
                    available_files.iter().filter(|p| glob.matches_path(p)).cloned().collect();
                if matches.is_empty() {
                    Err(RequestError::NoMatches(pattern.clone()))
                } else {
                    Ok(matches)
                }
            }
        }
    }

    fn read_file_with_range(&self, abs_path: &Path, rel_path: &Path) -> Result<FileContent> {
        let full_content = fs::read_to_string(abs_path)?;
        let (content, range_info) = match &self.range {
            None => (full_content, None),
            Some(RequestRange::Lines { lines }) => self.extract_line_range(&full_content, lines)?,
            Some(RequestRange::Symbol { symbol }) => self.extract_symbol(&full_content, symbol)?,
        };
        Ok(FileContent {
            path: rel_path.to_path_buf(),
            content,
            range_info,
        })
    }

    fn extract_line_range(&self, content: &str, range_spec: &str) -> Result<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let (start_line, end_line) = if let Some((s, e)) = range_spec.split_once('-') {
            let start = s.trim().parse::<usize>().map_err(|_| RequestError::InvalidLineRange(range_spec.to_string()))?;
            let end = if e.trim().is_empty() { total } else { e.trim().parse::<usize>().map_err(|_| RequestError::InvalidLineRange(range_spec.to_string()))? };
            (start, end)
        } else { 
            let line = range_spec.trim().parse::<usize>().map_err(|_| RequestError::InvalidLineRange(range_spec.to_string()))?;
            (line, line)
        };

        if start_line < 1 || start_line > end_line || end_line > total {
            return Err(RequestError::InvalidLineRange(format!("{} (file has {} lines)", range_spec, total)));
        }
        
        // Convert 1-based line numbers to 0-based indices for slicing.
        // `lines` is 0-indexed. A request for line `1` is index `0`.
        // A range `start..end` in Rust is exclusive of `end`.
        // So for "2-4", we want indices 1, 2, 3. The slice is `[1..4]`.
        let start_idx = start_line.saturating_sub(1);
        let end_idx = end_line;

        let extracted = lines.get(start_idx..end_idx).unwrap_or(&[]).join("\n");
        let info = format!("lines {}-{} of {}", start_line, end_line, total);
        Ok((extracted, info))
    }

    fn extract_symbol(&self, content: &str, symbol: &str) -> Result<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();
        let target_line_idx = lines.iter().position(|l| l.contains(symbol)).ok_or_else(|| RequestError::SymbolNotFound(symbol.to_string()))?;
        let context = 5;

        let start_idx = target_line_idx.saturating_sub(context);
        let end_idx = (target_line_idx + context + 1).min(lines.len());

        let extracted = lines.get(start_idx..end_idx).unwrap_or(&[]).join("\n");
        let info = format!("symbol '{}' at line {} (±{} lines context)", symbol, target_line_idx + 1, context);
        Ok((extracted, info))
    }
}

impl ResolvedRequest {
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        for file in &self.files {
            output.push_str(&format!("## {}\n", file.path.display()));
            if let Some(ref info) = file.range_info {
                output.push_str(&format!("*Showing: {}*\n", info));
            }
            output.push_str("```\n");
            output.push_str(&file.content);
            output.push_str("\n```\n");
        }
        output
    }
}