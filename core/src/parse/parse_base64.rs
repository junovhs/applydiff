use crate::error::{ErrorCode, PatchError, Result};

/// Sensible default cap for decoded output when callers don't have a custom cap.
/// 1 MiB keeps the UI responsive and prevents runaway pastes.
pub const MAX_BASE64_DECODED_DEFAULT: usize = 1_048_576;

/// Strict Base64 decoder:
/// - Ignores ASCII whitespace
/// - **Rejects** any non-alphabet bytes
/// - **Requires** correct padding ('=' only allowed in the final quartet)
/// - Fails if the estimated decoded size exceeds `max_decoded_len`
///
/// Returns raw bytes on success.
pub fn decode_base64_checked(s: &str, max_decoded_len: usize) -> Result<Vec<u8>> {
    // Build mapping table: 255 = invalid, 254 = padding '='
    let mut map = [255u8; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
    {
        map[*c as usize] = i as u8;
    }
    map[b'=' as usize] = 254;

    // Collect only base64 alphabet and '='; reject any other non-whitespace bytes.
    let mut clean: Vec<u8> = Vec::with_capacity(s.len());
    for (idx, b) in s.bytes().enumerate() {
        if b.is_ascii_whitespace() {
            continue;
        }
        let m = map[b as usize];
        if m == 255 {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: format!("Invalid base64 character 0x{b:02X} at byte offset {idx}"),
                context: "".to_string(),
            });
        }
        clean.push(b);
    }

    if clean.is_empty() {
        return Ok(Vec::new());
    }

    if clean.len() % 4 != 0 {
        return Err(PatchError::Parse {
            code: ErrorCode::ParseFailed,
            message: "Base64 length (after removing whitespace) is not a multiple of 4".to_string(),
            context: "".to_string(),
        });
    }

    // Count trailing padding and ensure '=' only appears at the tail.
    let mut pad = 0usize;
    if let Some(&last) = clean.last() {
        if last == b'=' {
            pad += 1;
            if clean.len() >= 2 && clean[clean.len() - 2] == b'=' {
                pad += 1;
            }
        }
    }
    // No '=' allowed before the trailing padding section.
    for (i, &ch) in clean[..clean.len() - pad].iter().enumerate() {
        if ch == b'=' {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: format!("Unexpected '=' padding at position {} (only allowed at the end)", i),
                context: "".to_string(),
            });
        }
    }

    // Estimate decoded size and bound it.
    let decoded_len = (clean.len() / 4) * 3 - pad;
    if decoded_len > max_decoded_len {
        return Err(PatchError::Validation {
            code: ErrorCode::BoundsExceeded,
            message: format!(
                "Decoded base64 would be {} bytes, which exceeds the limit of {} bytes",
                decoded_len, max_decoded_len
            ),
            context: "base64".to_string(),
        });
    }

    let mut out = Vec::with_capacity(decoded_len);
    let last_block_start = clean.len() - 4;

    // map again in a closure (safe, fast)
    let sext = |b: u8| -> u8 { map[b as usize] };

    let mut i = 0usize;
    while i <= last_block_start {
        let a = clean[i];
        let b = clean[i + 1];
        let c = clean[i + 2];
        let d = clean[i + 3];
        let is_last = i == last_block_start;

        let av = sext(a);
        let bv = sext(b);
        let cv = sext(c);
        let dv = sext(d);

        // '=' is only allowed in the final quartet; validate pattern.
        if !is_last && (av == 254 || bv == 254 || cv == 254 || dv == 254) {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Padding '=' encountered before the final quartet".to_string(),
                context: "".to_string(),
            });
        }
        if is_last {
            // Legal last-quartet patterns:
            // [v v v v] -> 3 bytes
            // [v v v =] -> 2 bytes
            // [v v = =] -> 1 byte
            if cv == 254 && dv != 254 {
                return Err(PatchError::Parse {
                    code: ErrorCode::ParseFailed,
                    message: "Invalid base64 padding: single '=' in 3rd position must be followed by '='".to_string(),
                    context: "".to_string(),
                });
            }
        }

        if av >= 64 || bv >= 64 || (cv != 254 && cv >= 64) || (dv != 254 && dv >= 64) {
            return Err(PatchError::Parse {
                code: ErrorCode::ParseFailed,
                message: "Invalid base64 sextet value".to_string(),
                context: "".to_string(),
            });
        }

        let x = ((av as u32) << 18)
            | ((bv as u32) << 12)
            | (if cv == 254 { 0 } else { (cv as u32) << 6 })
            | (if dv == 254 { 0 } else { dv as u32 });

        out.push(((x >> 16) & 0xFF) as u8);
        if cv != 254 {
            out.push(((x >> 8) & 0xFF) as u8);
        }
        if dv != 254 {
            out.push((x & 0xFF) as u8);
        }

        i += 4;
    }

    debug_assert_eq!(out.len(), decoded_len);
    Ok(out)
}

/// Legacy *lossy* decoder retained for compatibility (not re-exported).
/// It ignores invalid bytes and treats any padding loosely.
pub fn decode_base64_lossy(s: &str) -> Vec<u8> {
    // Mapping table: 255 = invalid, 254 = padding '='
    let mut map = [255u8; 256];
    for (i, c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        map[*c as usize] = i as u8;
    }
    map[b'=' as usize] = 254;

    // Collect values, ignoring whitespace
    let mut vals: Vec<u8> = Vec::new();
    for b in s.bytes() {
        if b.is_ascii_whitespace() { continue; }
        let m = map[b as usize];
        if m == 255 { continue; } // Ignore invalid bytes
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

        if a == 254 || b == 254 { break; }

        let x = ((a as u32) << 18) | ((b as u32) << 12) |
                (if c == 254 { 0 } else { (c as u32) << 6 }) |
                (if d == 254 { 0 } else { d as u32 });

        out.push(((x >> 16) & 0xFF) as u8);
        if c != 254 { out.push(((x >> 8) & 0xFF) as u8); }
        if d != 254 { out.push((x & 0xFF) as u8); }

        if c == 254 || d == 254 { break; }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_valid_string() {
        let good = "SGVsbG8sIFdvcmxkIQ=="; // "Hello, World!"
        let decoded = decode_base64_checked(good, 1024).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello, World!");
    }

    #[test]
    fn rejects_invalid_characters() {
        let bad = "abcd#efgh"; // '#' is not allowed
        assert!(decode_base64_checked(bad, 1024).is_err());
    }

    #[test]
    fn rejects_bad_padding_single_equals_in_third_position() {
        // Invalid: '=' in 3rd position must be followed by '=' in the 4th.
        let bad = "T==="; // made-up invalid quartet
        assert!(decode_base64_checked(bad, 1024).is_err());
    }

    #[test]
    fn enforces_size_cap() {
        // "AAAA" -> 3 zero bytes. Create enough quartets to exceed the cap.
        let quartets = (MAX_BASE64_DECODED_DEFAULT / 3) + 1;
        let huge = "AAAA".repeat(quartets);
        let err = decode_base64_checked(&huge, MAX_BASE64_DECODED_DEFAULT).unwrap_err();
        // It's enough that it errs; message/variant may vary.
        let _ = err; // don't assert variant to keep this stable
    }
}
