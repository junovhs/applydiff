// ===== PASTE THIS INTO matcher.rs - FINAL COMPILABLE VERSION =====
use crate::logger::Logger;
use strsim::normalized_damerau_levenshtein;

/// Result of locating the best match of `needle` within `haystack`
pub struct MatchResult {
    pub start: usize,
    pub end: usize,
    pub score: f64, // 0..1
}

/// Top-level matching strategy (layered):
/// 1) Exact substring
/// 2) Whitespace-normalized equality on windows of ~needle line length
/// 3) Relative-indentation-normalized equality on windows
/// 4) Fuzzy window search (Damerau–Levenshtein) with ambiguity guard
///
/// Notes:
/// - We always return byte offsets into the ORIGINAL haystack.
/// - All normalization is for comparison only; offsets come from the unmodified window range.
pub fn find_best_match(
    haystack: &str,
    needle: &str,
    min_score: f64,
    logger: &Logger,
) -> Option<MatchResult> {
    if needle.is_empty() {
        return Some(MatchResult { start: haystack.len(), end: haystack.len(), score: 1.0 });
    }

    // 0) Fast path: check for a UNIQUE exact substring
    let exact_matches: Vec<_> = haystack.match_indices(needle).collect();
    if exact_matches.len() == 1 {
        let (idx, _) = exact_matches[0];
        logger.info("matcher", "fast_path_match", &format!("unique exact substring (len={})", needle.len()));
        return Some(MatchResult { start: idx, end: idx + needle.len(), score: 1.0 });
    }
    // If 0 or >1 exact matches, we fall through to the layered search.
    // The layered search will correctly handle no-match or ambiguity.

    logger.info("matcher", "search_start", &format!("no exact match; layered search (needle_len={})", needle.len()));

    // Prepare line ranges with byte indices (end includes newline if present)
    let ranges = line_ranges(haystack);
    if ranges.is_empty() {
        logger.info("matcher", "empty_haystack", "no lines to search");
        return None;
    }

    // Compute nominal needle length in lines for window sizing
    let needle_lines_norm = normalize_newlines(needle);
    let n_lines = count_lines(&needle_lines_norm).max(1);
    let win_min = n_lines.saturating_sub(1);
    let win_max = n_lines + 1;

    // 1) Whitespace-normalized equality (tabs/spaces/extra spaces differences)
    {
        let needle_ws = normalize_ws_preserve_newlines(needle);
        let matches = scan_windows_equal(&ranges, haystack, &needle_ws, win_min, win_max, |s| {
            normalize_ws_preserve_newlines(s)
        });
        if matches.len() == 1 {
            let (start, end) = matches[0];
            logger.info("matcher", "normalized_ws_match", &format!("start={}, end={}", start, end));
            return Some(MatchResult { start, end, score: 1.0 });
        }
    }

    // 2) Relative-indentation-normalized equality (uniform outdent/indent)
    {
        let needle_rel = normalize_relative_indent(&normalize_ws_preserve_newlines(needle));
        let matches = scan_windows_equal(&ranges, haystack, &needle_rel, win_min, win_max, |s| {
            normalize_relative_indent(&normalize_ws_preserve_newlines(s))
        });
        if matches.len() == 1 {
            let (start, end) = matches[0];
            logger.info("matcher", "relative_indent_match", &format!("start={}, end={}", start, end));
            return Some(MatchResult { start, end, score: 1.0 });
        }
    }

    // 3) Fuzzy match (Damerau–Levenshtein) on windows with CRLF/LF insensitive scoring
    {
        let needle_norm = normalize_newlines(needle);
        let mut best_score: f64 = -1.0;
        let mut second_score: f64 = -1.0;
        let mut best_range: Option<(usize, usize)> = None;

        for win in win_min..=win_max {
            if win == 0 || ranges.len() < win {
                continue;
            }
            for i in 0..=ranges.len() - win {
                let start = ranges[i].0;
                let end = ranges[i + win - 1].1;
                let slice_with_nl = &haystack[start..end];

                // Trim trailing newline from slice to match how the parser prepares the needle.
                let mut slice = slice_with_nl;
                if slice.ends_with('\n') {
                    slice = &slice[..slice.len() - 1];
                    if slice.ends_with('\r') {
                        slice = &slice[..slice.len() - 1];
                    }
                }

                // CRLF-insensitive scoring (now on a correctly trimmed slice)
                let slice_norm = normalize_newlines(slice);
                let score = normalized_damerau_levenshtein(&slice_norm, &needle_norm);

                if score > best_score {
                    second_score = best_score;
                    best_score = score;
                    best_range = Some((start, end));
                } else if score > second_score {
                    second_score = score;
                }
            }
        }

        // Decide based on threshold and ambiguity
        if let Some((start, end)) = best_range {
            if best_score >= min_score {
                // Avoid wrong-place edits when two windows are nearly equal
                if second_score >= 0.0 && (best_score - second_score) < 0.02 && second_score >= min_score {
                    logger.info("matcher", "ambiguous_match", &format!("best={:.3}, second={:.3}", best_score, second_score));
                    return None;
                }
                logger.info("matcher", "fuzzy_match", &format!("start={}, end={}, score={:.3}", start, end, best_score));
                return Some(MatchResult { start, end, score: best_score });
            } else {
                logger.info("matcher", "no_match_threshold", &format!("best={:.3} < min={:.3}", best_score, min_score));
            }
        } else {
            logger.info("matcher", "no_candidates", "no windows produced a score");
        }
    }

    None
}

/* ============================== helpers ============================== */

/// Scan windows defined by `ranges`, transforming each slice with `xfm`,
/// and return ALL windows whose transformed text equals `needle_xfm`.
fn scan_windows_equal(
    ranges: &[(usize, usize)],
    haystack: &str,
    needle_xfm: &str,
    win_min: usize,
    win_max: usize,
    mut xfm: impl FnMut(&str) -> String,
) -> Vec<(usize, usize)> {
    let mut hits = Vec::new();
    if ranges.is_empty() {
        return hits;
    }
    for win in win_min..=win_max {
        if win == 0 || ranges.len() < win {
            continue;
        }
        for i in 0..=ranges.len() - win {
            let start = ranges[i].0;
            let end = ranges[i + win - 1].1;
            let slice = &haystack[start..end];
            if xfm(slice) == needle_xfm {
                hits.push((start, end));
            }
        }
    }
    hits
}


/// Return a vector of (start_byte, end_byte) for each logical line,
/// where end includes the newline if present. Handles both LF and CRLF inputs
/// (we look for '\n'; '\r' is carried within the preceding byte, if any).
fn line_ranges(s: &str) -> Vec<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut line_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\n' {
            // include '\n' in the line
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

/// Replace CRLF with LF for comparison/scoring (allocation-free fast path if no '\r').
fn normalize_newlines(s: &str) -> String {
    if s.as_bytes().contains(&b'\r') {
        s.replace("\r\n", "\n")
    } else {
        s.to_string()
    }
}

/// Collapse horizontal whitespace runs (space or tab) to a single space per line,
/// and strip trailing whitespace on each line. Newlines are preserved.
fn normalize_ws_preserve_newlines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.split_inclusive('\n') {
        // split_inclusive keeps the '\n' on the line; handle body without the final '\n'
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
        // trim trailing single space we may have appended
        if out.ends_with(' ') {
            out.pop();
        }
        out.push_str(nl);
    }
    out
}

/// Remove a uniform leading indentation (min leading spaces/tabs across non-empty lines),
/// then return the result. Combine with `normalize_ws_preserve_newlines` for tabs/spaces variance.
fn normalize_relative_indent(s: &str) -> String {
    // Compute min leading whitespace across non-empty lines
    let mut min_ws: Option<usize> = None;
    for line in s.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let ws = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        min_ws = Some(match min_ws {
            Some(m) => m.min(ws),
            None => ws,
        });
    }
    let take = min_ws.unwrap_or(0);
    if take == 0 {
        return s.to_string();
    }

    // Strip exactly `take` leading whitespace chars from each non-empty line
    let mut out = String::with_capacity(s.len());
    for (i, line) in s.split_inclusive('\n').enumerate() {
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
        // remove up to `take` of leading spaces/tabs
        let mut removed = 0usize;
        let mut chars = body.chars();
        while removed < take {
            if let Some(c) = chars.next() {
                if c == ' ' || c == '\t' {
                    removed += 1;
                } else {
                    // non-whitespace encountered early; push it and break
                    out.push(c);
                    break;
                }
            } else {
                break;
            }
        }
        // push the rest
        out.push_str(chars.as_str());
        out.push_str(nl);

        // handle lines that had fewer than `take` ws chars by appending remainder of original body
        if removed < take {
            // (no-op: the loop above already stopped at first non-ws and appended it)
            // kept for clarity; no extra work needed.
        }

        // avoid unused variable warning for i in certain builds
        let _ = i;
    }
    out
}

fn count_lines(s: &str) -> usize {
    if s.is_empty() { 0 } else { s.lines().count().max(1) }
}