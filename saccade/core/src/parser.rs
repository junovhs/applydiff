use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Parser, Query, QueryCursor};

const CHUNK_SEPARATOR: &str = "\n---⋯\n";
const JAVASCRIPT_QUERY: &str = r#"(import_statement) @capture (export_statement) @capture (comment) @capture (function_declaration) @def (method_definition) @def (class_declaration) @def"#;
const TYPESCRIPT_QUERY: &str = r#"(import_statement) @capture (export_statement) @capture (comment) @capture (interface_declaration) @capture (type_alias_declaration) @capture (function_declaration) @def (method_definition) @def (class_declaration) @def"#;
const RUST_QUERY: &str = r#"(line_comment) @capture (block_comment) @capture (use_declaration) @capture (struct_item) @capture (enum_item) @capture (function_item body: (_) @body) @def (trait_item body: (_) @body) @def (impl_item body: (_) @body) @def"#;
const PYTHON_QUERY: &str = r#"(comment) @capture (import_statement) @capture (import_from_statement) @capture (function_definition body: (block) @body) @def (class_definition body: (block) @body) @def"#;

pub fn skeletonize_file(content: &str, file_extension: &str) -> Option<String> {
    let (language, query_str) = match file_extension {
        "js" | "jsx" => (tree_sitter_javascript::language(), JAVASCRIPT_QUERY),
        "ts" => (tree_sitter_typescript::language_typescript(), TYPESCRIPT_QUERY),
        "tsx" => (tree_sitter_typescript::language_tsx(), TYPESCRIPT_QUERY),
        "rs" => (tree_sitter_rust::language(), RUST_QUERY),
        "py" => (tree_sitter_python::language(), PYTHON_QUERY),
        _ => return None,
    };
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(content, None)?;
    let query = Query::new(&language, query_str).ok()?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    let mut results = Vec::new();
    let mut seen_ids: HashSet<usize> = HashSet::new();
    for m in matches {
        let mut caps: HashMap<&str, Node> = HashMap::new();
        for c in m.captures { caps.insert(&query.capture_names()[c.index as usize], c.node); }
        if let (Some(def), Some(body)) = (caps.get("def"), caps.get("body")) {
            if seen_ids.insert(def.id()) {
                if let Some(sig) = content.get(def.start_byte()..body.start_byte()) {
                    results.push(sig.trim().to_string());
                }
            }
        } else if let Some(cap) = caps.get("capture").or(caps.get("def")) {
            if seen_ids.insert(cap.id()) {
                if let Ok(text) = cap.utf8_text(content.as_bytes()) {
                    results.push(text.trim().to_string());
                }
            }
        }
    }
    if results.is_empty() { None } else { Some(results.join(CHUNK_SEPARATOR)) }
}