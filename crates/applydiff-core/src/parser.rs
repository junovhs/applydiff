use crate::error::{ErrorCode, PatchError, Result};
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub file: PathBuf,
    pub from: String,
    pub to: String,
    pub fuzz: f64, // 0.0..=1.0
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self { Self::default() }

    /// Top-level parser. Supports two formats:
    /// 1) Classic sentinel format:
    ///
    /// >>> file: path/to/file | fuzz=0.90
    /// --- from
    /// (old text)
    /// --- to
    /// (new text)
    /// <<<
    ///
    /// 2) Armored Framed Blocks v1 (AFB-1), robust against chat UIs:
    ///
    /// -----BEGIN APPLYDIFF AFB-1-----
    /// Path: relative/path.txt
    /// Fuzz: 0.85
    /// Encoding: base64
    /// From:
    /// <base64-encoded UTF-8 of "from">
    /// To:
    /// <base64-encoded UTF-8 of "to">
    /// -----END APPLYDIFF AFB-1-----
    ///
    /// Notes:
    /// - Base64 may be arbitrarily wrapped; whitespace is ignored.
    /// - If Encoding is omitted, it defaults to base64.
    pub fn parse(&self, input: &str) -> Result<Vec<PatchBlock>> {
        let mut out: Vec<PatchBlock> = Vec::new();
        let mut lines = input.lines().enumerate().peekable();

        while let Some((_, line)) = lines.peek().cloned() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("-----BEGIN APPLYDIFF AFB-1-----") {
                let blk = Self::parse_one_armored(&mut lines)?;
                out.push(blk);
                continue;
            }

            if trimmed.starts_with(">>>") {
                let blk = Self::parse_one_classic(&mut lines)?;
                out.push(blk);
                continue;
            }

            // ignore noise
            lines.next();
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

    fn parse_one_classic(lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>) -> Result<PatchBlock> {
        let re_head = Regex::new(
            r#"^>>>\s*file:\s*(?P<file>[^|]+?)(?:\s*\|\s*fuzz=(?P<fuzz>[0-9.]+))?\s*$"#
        ).unwrap();

        // header
        let (_, header) = lines.next().ok_or_else(|| PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Unexpected end while reading header".to_string(),
            context: "".to_string(),
        })?;

        let caps = re_head.captures(header).ok_or_else(|| PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Invalid header; expected '>>> file: <path> [| fuzz=<0..1>]'" .to_string(),
            context: header.to_string(),
        })?;

        let file = caps["file"].trim().to_string();
        let fuzz = caps
            .name("fuzz")
            .map(|m| m.as_str().parse::<f64>().unwrap_or(0.85))
            .unwrap_or(0.85);

        // expect --- from
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

        // collect FROM until --- to
        let mut from = String::new();
        while let Some((_, l)) = lines.peek().cloned() {
            if l.trim() == "--- to" { lines.next(); break; }
            from.push_str(l);
            from.push('\n');
            lines.next();
        }

        // collect TO until <<< (required)
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

        // Trim trailing single newline for neatness
        if from.ends_with('\n') { from.pop(); }
        if to.ends_with('\n') { to.pop(); }

        Ok(PatchBlock {
            file: PathBuf::from(file),
            from,
            to,
            fuzz: fuzz.clamp(0.0, 1.0),
        })
    }

    fn parse_one_armored(lines: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Lines<'_>>>) -> Result<PatchBlock> {
        // consume BEGIN
        lines.next();

        let mut path: Option<String> = None;
        let mut fuzz: f64 = 0.85;
        let mut encoding = String::from("base64");

        // read headers until "From:"
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

        // expect From:
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

        // collect base64 until 'To:'
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

        // collect base64 until END
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

        let from = if encoding == "base64" {
            let v = decode_base64_lossy(&from_buf);
            String::from_utf8(v).map_err(|_| PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Armored 'From' is not valid UTF-8 after base64 decode".to_string(),
                context: file.clone(),
            })?
        } else {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: format!("Unsupported Encoding: {}", encoding),
                context: file.clone(),
            });
        };

        let to = if encoding == "base64" {
            let v = decode_base64_lossy(&to_buf);
            String::from_utf8(v).map_err(|_| PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Armored 'To' is not valid UTF-8 after base64 decode".to_string(),
                context: file.clone(),
            })?
        } else {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: format!("Unsupported Encoding: {}", encoding),
                context: file.clone(),
            });
        };

        Ok(PatchBlock {
            file: PathBuf::from(file),
            from,
            to,
            fuzz: fuzz.clamp(0.0, 1.0),
        })
    }
}

/// Minimal base64 decoder that ignores ASCII whitespace. Returns raw bytes.
/// This avoids adding external dependencies to the core crate.
fn decode_base64_lossy(s: &str) -> Vec<u8> {
    // mapping table: 255 = invalid, 254 = padding '='
    let mut map = [255u8; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        map[*c as usize] = i as u8;
    }
    map[b'=' as usize] = 254;

    // collect values, ignoring whitespace
    let mut vals: Vec<u8> = Vec::new();
    for b in s.bytes() {
        if b.is_ascii_whitespace() { continue; }
        let m = map[b as usize];
        if m == 255 {
            // ignore other non-base64 bytes silently to be tolerant of chat artifacts
            continue;
        }
        vals.push(m);
    }

    let mut out: Vec<u8> = Vec::with_capacity(vals.len() * 3 / 4 + 3);
    let mut i = 0usize;
    while i + 3 < vals.len() {
        let a = vals[i];
        let b = vals[i+1];
        let c = vals[i+2];
        let d = vals[i+3];
        i += 4;

        if a == 254 || b == 254 {
            break;
        }
        let x = ((a as u32) << 18) | ((b as u32) << 12) |
                (if c == 254 { 0 } else { (c as u32) << 6 }) |
                (if d == 254 { 0 } else { d as u32 });

        out.push(((x >> 16) & 0xFF) as u8);
        if c != 254 {
            out.push(((x >> 8) & 0xFF) as u8);
        }
        if d != 254 {
            out.push((x & 0xFF) as u8);
        }

        if c == 254 || d == 254 {
            break;
        }
    }

    out
}
