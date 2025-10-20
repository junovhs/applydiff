use crate::error::{ErrorCode, PatchError, Result};
use crate::parse::{PatchBlock, decode_base64_lossy};
use std::path::PathBuf;

pub fn parse_armored_block(
    lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>
) -> Result<PatchBlock> {
    // Consume BEGIN line
    lines.next();

    let mut path: Option<String> = None;
    let mut fuzz: f64 = 0.85;
    let mut encoding = String::from("base64");

    // Read headers until "From:"
    while let Some((_, l)) = lines.peek().cloned() {
        let t = l.trim();
        if t == "From:" { break; }
        if t == "-----END APPLYDIFF AFB-1-----" {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Armored block missing 'From:'".to_string(),
                context: "".to_string(),
            });
        }
        if let Some(rest) = t.strip_prefix("Path:") {
            path = Some(rest.trim().to_string());
        } else if let Some(rest) = t.strip_prefix("Fuzz:") {
            fuzz = rest.trim().parse::<f64>().unwrap_or(0.85);
        } else if let Some(rest) = t.strip_prefix("Encoding:") {
            encoding = rest.trim().to_lowercase();
        }
        lines.next();
    }

    let file = path.ok_or_else(|| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Armored block missing 'Path:' header".to_string(),
        context: "".to_string(),
    })?;

    // Expect From:
    match lines.next() {
        Some((_, l)) if l.trim() == "From:" => {}
        Some((_, other)) => return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Expected 'From:'".to_string(),
            context: other.to_string(),
        }),
        None => return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Unexpected end before 'From:'".to_string(),
            context: "".to_string(),
        }),
    }

    // Collect base64 until 'To:'
    let mut from_buf = String::new();
    while let Some((_, l)) = lines.peek().cloned() {
        if l.trim() == "To:" { lines.next(); break; }
        if l.trim() == "-----END APPLYDIFF AFB-1-----" {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Expected 'To:' in armored block".to_string(),
                context: file.clone(),
            });
        }
        from_buf.push_str(l);
        from_buf.push('\n');
        lines.next();
    }

    // Collect base64 until END
    let mut to_buf = String::new();
    let mut found_end = false;
    while let Some((_, l)) = lines.peek().cloned() {
        if l.trim() == "-----END APPLYDIFF AFB-1-----" { lines.next(); found_end = true; break; }
        to_buf.push_str(l);
        to_buf.push('\n');
        lines.next();
    }
    
    if !found_end {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Armored block missing end marker".to_string(),
            context: file.clone(),
        });
    }

    if encoding != "base64" {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: format!("Unsupported Encoding: {}", encoding),
            context: file.clone(),
        });
    }

    let from = String::from_utf8(decode_base64_lossy(&from_buf)).map_err(|_| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Armored 'From' is not valid UTF-8 after base64 decode".to_string(),
        context: file.clone(),
    })?;

    let to = String::from_utf8(decode_base64_lossy(&to_buf)).map_err(|_| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Armored 'To' is not valid UTF-8 after base64 decode".to_string(),
        context: file.clone(),
    })?;

    Ok(PatchBlock {
        file: PathBuf::from(file),
        from,
        to,
        fuzz: fuzz.clamp(0.0, 1.0),
    })
}