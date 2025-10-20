use crate::error::{ErrorCode, PatchError, Result};
use crate::parse::PatchBlock;
use regex::Regex;
use std::path::PathBuf;

pub fn parse_classic_block(
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>
) -> Result<PatchBlock> {
    let re_head = Regex::new(
        r#"^>>>\s*file:\s*(?P<file>[^|]+?)(?:\s*\|\s*fuzz=(?P<fuzz>[0-9.]+))?\s*$"#
    ).unwrap();

    // Header
    let (_, header) = lines.next().ok_or_else(|| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Unexpected end while reading header".to_string(),
        context: "".to_string(),
    })?;

    let caps = re_head.captures(header).ok_or_else(|| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Invalid header; expected '>>> file: <path> [| fuzz=<0..1>]'".to_string(),
        context: header.to_string(),
    })?;

    let file = caps["file"].trim().to_string();
    let fuzz = caps
        .name("fuzz")
        .map(|m| m.as_str().parse::<f64>().unwrap_or(0.85))
        .unwrap_or(0.85);

    // Expect --- from
    match lines.next() {
        Some((_, l)) if l.trim() == "--- from" => {}
        Some((_, other)) => return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '--- from'".to_string(),
            context: other.to_string(),
        }),
        None => return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Unexpected end after header".to_string(),
            context: "".to_string(),
        }),
    }

    // Collect FROM until --- to
    let mut from = String::new();
    while let Some((_, l)) = lines.peek().cloned() {
        if l.trim() == "--- to" { lines.next(); break; }
        from.push_str(l);
        from.push('\n');
        lines.next();
    }

    // Collect TO until <<< (required)
    let mut to = String::new();
    let mut found_end = false;
    while let Some((_, l)) = lines.peek().cloned() {
        if l.trim() == "<<<" { lines.next(); found_end = true; break; }
        to.push_str(l);
        to.push('\n');
        lines.next();
    }
    
    if !found_end {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected '<<<' to close patch block".to_string(),
            context: file.clone(),
        });
    }

    // Trim trailing newline
    if from.ends_with('\n') { from.pop(); }
    if to.ends_with('\n') { to.pop(); }

    Ok(PatchBlock {
        file: PathBuf::from(file),
        from,
        to,
        fuzz: fuzz.clamp(0.0, 1.0),
    })
}