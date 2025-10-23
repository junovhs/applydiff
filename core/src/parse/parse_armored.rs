use crate::error::{ErrorCode, PatchError, Result};
use crate::parse::{PatchBlock, decode_base64_checked};
use crate::parse::parse_base64::MAX_BASE64_DECODED_DEFAULT;
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

    // Collect until 'To:'
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

    // Collect until END
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

    // Strict, bounded decode (propagates precise errors: invalid char, bad padding, too large)
    let from_bytes = decode_base64_checked(&from_buf, MAX_BASE64_DECODED_DEFAULT)?;
    let to_bytes   = decode_base64_checked(&to_buf,   MAX_BASE64_DECODED_DEFAULT)?;

    let from = String::from_utf8(from_bytes).map_err(|_| PatchError::Parse {
        code: ErrorCode::ParseFailed,
        message: "Armored 'From' is not valid UTF-8 after base64 decode".to_string(),
        context: file.clone(),
    })?;

    let to = String::from_utf8(to_bytes).map_err(|_| PatchError::Parse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::Parser;

    fn make_block(from_b64: &str, to_b64: &str) -> String {
        format!(
"-----BEGIN APPLYDIFF AFB-1-----
Path: tmp.txt
Encoding: base64
From:
{from}
To:
{to}
-----END APPLYDIFF AFB-1-----
", from = from_b64, to = to_b64)
    }

    #[test]
    fn parses_valid_armored_block() {
        // "Foo" -> Rm9v, "Bar" -> QmFy
        let patch = make_block("Rm9v", "QmFy");
        let out = Parser::new().parse(&patch).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].file, PathBuf::from("tmp.txt"));
        assert_eq!(out[0].from, "Foo");
        assert_eq!(out[0].to, "Bar");
    }

    #[test]
    fn armored_rejects_invalid_character() {
        // Inject a bad character into base64.
        let patch = make_block("Rm9v#", "QmFy");
        let err = Parser::new().parse(&patch);
        assert!(err.is_err(), "should reject invalid base64 char");
    }

    #[test]
    fn armored_rejects_too_large() {
        // "AAAA" -> 3 zero bytes. Make From exceed the 1 MiB default cap.
        let quartets = (crate::parse::parse_base64::MAX_BASE64_DECODED_DEFAULT / 3) + 1;
        let huge = "AAAA".repeat(quartets);
        let patch = make_block(&huge, "QmFy");
        let err = Parser::new().parse(&patch);
        assert!(err.is_err(), "should reject oversized base64 payload");
    }
}
