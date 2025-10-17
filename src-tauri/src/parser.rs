use crate::error::{ErrorCode, PatchError, Result};
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub file: PathBuf,
    pub from: String,
    pub to: String,
    pub fuzz: f32, // 0.0..=1.0
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self { Self::default() }

    /// Parses blocks in the form:
    /// >>> file: path/to/file | fuzz=0.90
    /// --- from
    /// (old text)
    /// --- to
    /// (new text)
    /// <<<
    pub fn parse(&self, input: &str) -> Result<Vec<PatchBlock>> {
        let mut out = Vec::new();
        let mut lines = input.lines().enumerate().peekable();

        let re_head = Regex::new(r#"^>>>\s*file:\s*(?P<file>[^|]+?)(?:\s*\|\s*fuzz=(?P<fuzz>[0-9.]+))?\s*$"#).unwrap();

        while let Some((_, line)) = lines.peek().cloned() {
            if !line.trim_start().starts_with(">>>") {
                lines.next();
                continue;
            }

            // header
            let caps = re_head.captures(line).ok_or_else(|| PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Invalid header; expected '>>> file: <path> [| fuzz=<0..1>]'" .to_string(),
                context: line.to_string(),
            })?;
            let file = caps["file"].trim().to_string();
            let fuzz = caps.name("fuzz").map(|m| m.as_str().parse::<f32>().unwrap_or(0.85)).unwrap_or(0.85);
            lines.next(); // consume header

            // expect --- from
            match lines.next() {
                Some((_, l)) if l.trim() == "--- from" => {}
                Some((_, other)) => return Err(PatchError::Parse {
                    code: ErrorCode::ParseFailed,
                    message: "Expected '--- from'".to_string(),
                    context: other.to_string(),
                }),
                None => break,
            }

            // collect FROM until --- to
            let mut from = String::new();
            while let Some((_, l)) = lines.peek().cloned() {
                if l.trim() == "--- to" { lines.next(); break; }
                from.push_str(l);
                from.push('\n');
                lines.next();
            }

            // collect TO until <<<
            let mut to = String::new();
            while let Some((_, l)) = lines.peek().cloned() {
                if l.trim() == "<<<" { lines.next(); break; }
                to.push_str(l);
                to.push('\n');
                lines.next();
            }

            // Trim trailing single newline for neatness
            if from.ends_with('\n') { from.pop(); }
            if to.ends_with('\n') { to.pop(); }

            out.push(PatchBlock {
                file: PathBuf::from(file),
                from,
                to,
                fuzz: fuzz.clamp(0.0, 1.0),
            });
        }

        if out.is_empty() {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "No patch blocks found".to_string(),
                context: "".to_string(),
            });
        }

        Ok(out)
    }
}
