use super::PatchBlock;
use crate::error::{ErrorCode, PatchError, Result};
use regex::Regex;
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::Lines;

pub fn parse_classic_block(lines: &mut Peekable<Lines<'_>>) -> Result<PatchBlock> {
    // Regex for the header line, e.g., ">>> file: src/main.rs | fuzz=0.8"
    let re_head = Regex::new(
        r#"^>>>\s*file:\s*(?P<file>[^|]+?)(?:\s*\|\s*fuzz=(?P<fuzz>[0-9.]+))?\s*$"#
    ).unwrap();

    // 1. Parse Header
    let header = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Unexpected end of input while parsing block header".to_string(),
        context: "header".to_string(),
    })?;

    let caps = re_head.captures(header).ok_or_else(|| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Invalid header format. Expected '>>> file: <path> [| fuzz=<value>]'".to_string(),
        context: header.to_string(),
    })?;

    let file_path_str = caps["file"].trim().to_string();
    let fuzz = caps
        .name("fuzz")
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .unwrap_or(0.85)
        .clamp(0.1, 1.0); // Enforce a sane fuzz range

    // 2. Parse "--- from" section
    let from_marker = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Expected '--- from' marker but found end of input".to_string(),
        context: file_path_str.clone(),
    })?;
    if from_marker.trim() != "--- from" {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '--- from' marker".to_string(),
            context: from_marker.to_string(),
        });
    }

    let mut from_lines = Vec::new();
    while let Some(line) = lines.peek() {
        if line.trim() == "--- to" {
            break;
        }
        from_lines.push(lines.next().unwrap());
    }

    // 3. Parse "--- to" section
    let to_marker = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Expected '--- to' marker but found end of input".to_string(),
        context: file_path_str.clone(),
    })?;
    if to_marker.trim() != "--- to" {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '--- to' marker".to_string(),
            context: to_marker.to_string(),
        });
    }

    let mut to_lines = Vec::new();
    while let Some(line) = lines.peek() {
        if line.trim() == "<<<" {
            break;
        }
        to_lines.push(lines.next().unwrap());
    }

    // 4. Expect and consume end marker "<<<"
    let end_marker = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Expected '<<<' end marker but found end of input".to_string(),
        context: file_path_str.clone(),
    })?;
    if end_marker.trim() != "<<<" {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '<<<' end marker".to_string(),
            context: end_marker.to_string(),
        });
    }

    Ok(PatchBlock {
        file: PathBuf::from(file_path_str),
        from: from_lines.join("\n"),
        to: to_lines.join("\n"),
        fuzz,
    })
}