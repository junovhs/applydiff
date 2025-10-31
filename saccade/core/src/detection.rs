use crate::error::Result;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;
use tree_sitter::{Parser, Query};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BuildSystemType { Rust, Node, Python, Go, CMake, Conan }

impl fmt::Display for BuildSystemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{:?}", self) }
}

pub struct Detector;

const CMAKE_AST_QUERY: &str = r#"(identifier) @cmd"#;
const CMAKE_CONFIRMATION_KEYWORDS: &[&str] = &["add_executable", "target_link_libraries", "project", "cmake_minimum_required", "find_package"];

impl Detector {
    pub fn new() -> Self { Self }

    pub fn detect_build_systems(&self, files: &[std::path::PathBuf]) -> Result<Vec<BuildSystemType>> {
        let mut detected = HashSet::new();
        for file in files {
            if file.ends_with("Cargo.toml") { detected.insert(BuildSystemType::Rust); }
            if file.ends_with("package.json") { detected.insert(BuildSystemType::Node); }
            if matches!(file.file_name().and_then(|n| n.to_str()), Some("requirements.txt" | "pyproject.toml")) { detected.insert(BuildSystemType::Python); }
            if file.ends_with("go.mod") { detected.insert(BuildSystemType::Go); }
            if self.is_cmake_validated(file)? { detected.insert(BuildSystemType::CMake); }
            if matches!(file.file_name().and_then(|n| n.to_str()), Some("conanfile.txt" | "conanfile.py")) { detected.insert(BuildSystemType::Conan); }
        }
        Ok(detected.into_iter().collect())
    }

    fn is_cmake_validated(&self, path: &Path) -> Result<bool> {
        let path_str = path.to_string_lossy();
        if !path_str.contains("CMakeLists.txt") && !path_str.ends_with(".cmake") { return Ok(false); }
        let content = match fs::read_to_string(path) { Ok(c) => c, Err(_) => return Ok(false) };

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_cmake::language()).map_err(|e| crate::error::SaccadeError::Other(e.to_string()))?;
        let tree = match parser.parse(&content, None) { Some(t) => t, None => return Ok(false) };

        let query = Query::new(&tree_sitter_cmake::language(), CMAKE_AST_QUERY).map_err(|e| crate::error::SaccadeError::Other(e.to_string()))?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        for m in matches {
            for capture in m.captures {
                if let Ok(cmd) = capture.node.utf8_text(content.as_bytes()) {
                    if CMAKE_CONFIRMATION_KEYWORDS.contains(&cmd) { return Ok(true); }
                }
            }
        }
        Ok(false)
    }
}