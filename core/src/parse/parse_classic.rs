use super::{PatchBlock, PatchMode};
use crate::error::{ErrorCode, PatchError, Result};
use regex::Regex;
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::Lines;

/// Parses a classic patch block, now supporting `mode=replace` and `mode=regex`.
///
/// # Panics
///
/// Panics if the header regex fails to compile.
///
/// # Errors
///
/// Returns an error if the block has an invalid header format or is missing
/// expected markers (`--- from`, `--- to`, `<<<`).
pub fn parse_classic_block(lines: &mut Peekable<Lines<'_>>) -> Result<PatchBlock> {
    let re_head =
        Regex::new(r"^>>>\s*file:\s*(?P<file>[^|]+?)(?:\s*\|\s*(?P<options>.+))?\s*$").unwrap();

    let header = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Unexpected end of input while parsing block header".to_string(),
        context: "header".to_string(),
    })?;

    let caps = re_head.captures(header).ok_or_else(|| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Invalid header format. Expected '>>> file: <path> [| <options>]'".to_string(),
        context: header.to_string(),
    })?;

    let file_path_str = caps["file"].trim().to_string();
    let options_str = caps.name("options").map_or("", |m| m.as_str());

    let mut fuzz = 0.85;
    let mut mode = PatchMode::Classic;

    for part in options_str.split_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "fuzz" => {
                    fuzz = value.parse::<f64>().unwrap_or(0.85).clamp(0.1, 1.0);
                }
                "mode" => match value {
                    "replace" => mode = PatchMode::Replace,
                    "regex" => mode = PatchMode::Regex,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    if mode == PatchMode::Replace {
        let to_lines = consume_until_marker(lines, "<<<");
        consume_end_marker(lines, &file_path_str)?;
        return Ok(PatchBlock {
            file: PathBuf::from(file_path_str),
            mode,
            from: String::new(),
            to: to_lines.join("\n"),
            fuzz,
        });
    }

    consume_marker(lines, "--- from", &file_path_str)?;
    let from_lines = consume_until_marker(lines, "--- to");
    consume_marker(lines, "--- to", &file_path_str)?;
    let to_lines = consume_until_marker(lines, "<<<");
    consume_end_marker(lines, &file_path_str)?;

    Ok(PatchBlock {
        file: PathBuf::from(file_path_str),
        mode,
        from: from_lines.join("\n"),
        to: to_lines.join("\n"),
        fuzz,
    })
}

fn consume_until_marker<'a>(
    lines: &mut Peekable<Lines<'a>>,
    marker: &str,
) -> Vec<&'a str> {
    let mut content_lines = Vec::new();
    while let Some(line) = lines.peek() {
        if line.trim() == marker {
            break;
        }
        content_lines.push(lines.next().unwrap());
    }
    content_lines
}

fn consume_marker(
    lines: &mut Peekable<Lines<'_>>,
    expected_marker: &str,
    context_file: &str,
) -> Result<()> {
    let marker = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: format!("Expected '{expected_marker}' marker but found end of input"),
        context: context_file.to_string(),
    })?;
    if marker.trim() != expected_marker {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: format!("Expected '{expected_marker}' marker"),
            context: marker.to_string(),
        });
    }
    Ok(())
}

fn consume_end_marker(lines: &mut Peekable<Lines<'_>>, context_file: &str) -> Result<()> {
    let end_marker = lines.next().ok_or(PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Expected '<<<' end marker but found end of input".to_string(),
        context: context_file.to_string(),
    })?;
    if end_marker.trim() != "<<<" {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '<<<' end marker".to_string(),
            context: end_marker.to_string(),
        });
    }
    Ok(())
}