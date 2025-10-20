/// Return a vector of (start_byte, end_byte) for each logical line,
/// where end includes the newline if present.
pub fn line_ranges(s: &str) -> Vec<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut line_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let end = i + 1;
            out.push((line_start, end));
            i += 1;
            line_start = i;
        } else {
            i += 1;
        }
    }

    // Last line without trailing newline
    if line_start < bytes.len() {
        out.push((line_start, bytes.len()));
    }
    out
}

/// Replace CRLF with LF for comparison/scoring
pub fn normalize_newlines(s: &str) -> String {
    if s.as_bytes().contains(&b'\r') {
        s.replace("\r\n", "\n")
    } else {
        s.to_string()
    }
}

/// Collapse horizontal whitespace runs to single space per line,
/// strip trailing whitespace. Newlines are preserved.
pub fn normalize_ws_preserve_newlines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.split_inclusive('\n') {
        let (body, nl) = if let Some(stripped) = line.strip_suffix('\n') {
            (stripped, "\n")
        } else {
            (line, "")
        };

        let mut last_was_ws = false;
        for ch in body.chars() {
            if ch == ' ' || ch == '\t' {
                if !last_was_ws {
                    out.push(' ');
                    last_was_ws = true;
                }
            } else {
                out.push(ch);
                last_was_ws = false;
            }
        }
        
        // Trim trailing single space
        if out.ends_with(' ') {
            out.pop();
        }
        out.push_str(nl);
    }
    out
}

/// Remove uniform leading indentation (min leading spaces/tabs across non-empty lines)
pub fn normalize_relative_indent(s: &str) -> String {
    // Compute min leading whitespace
    let mut min_ws: Option<usize> = None;
    for line in s.lines() {
        if line.trim().is_empty() { continue; }
        let ws = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        min_ws = Some(match min_ws {
            Some(m) => m.min(ws),
            None => ws,
        });
    }
    let take = min_ws.unwrap_or(0);
    if take == 0 { return s.to_string(); }

    // Strip exactly `take` leading whitespace chars
    let mut out = String::with_capacity(s.len());
    for line in s.split_inclusive('\n') {
        let (body, nl) = if let Some(stripped) = line.strip_suffix('\n') {
            (stripped, "\n")
        } else {
            (line, "")
        };
        
        if body.trim().is_empty() {
            out.push_str(body);
            out.push_str(nl);
            continue;
        }
        
        let mut removed = 0usize;
        let mut chars = body.chars();
        while removed < take {
            if let Some(c) = chars.next() {
                if c == ' ' || c == '\t' {
                    removed += 1;
                } else {
                    out.push(c);
                    break;
                }
            } else {
                break;
            }
        }
        out.push_str(chars.as_str());
        out.push_str(nl);
    }
    out
}