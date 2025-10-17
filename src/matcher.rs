use crate::logger::Logger;
use strsim::normalized_damerau_levenshtein;

/// Result of locating the best match of `needle` within `haystack`
pub struct MatchResult {
    pub start: usize,
    pub end: usize,
    pub score: f32, // 0..1
}

/// Normalize line endings to '\n' for scoring, but compute byte offsets
/// on the original haystack so replacements write correctly on Windows (CRLF) too.
pub fn find_best_match(haystack: &str, needle: &str, min_score: f32, logger: &Logger) -> Option<MatchResult> {
    if needle.is_empty() {
        return Some(MatchResult { start: haystack.len(), end: haystack.len(), score: 1.0 });
    }

    // Fast path: exact substring (works for single-line or exact EOL matches)
    if let Some(idx) = haystack.find(needle) {
        logger.info("matcher", "fast_path_match", &format!("Found exact match for needle of length {}", needle.len()));
        return Some(MatchResult { start: idx, end: idx + needle.len(), score: 1.0 });
    }

    logger.info("matcher", "fuzzy_search_start", &format!("No exact match. Starting fuzzy search for needle of length {}", needle.len()));
    
    // Prepare line ranges with byte indices in the ORIGINAL haystack.
    let ranges = line_ranges(haystack); // each range includes its newline(s)
    if ranges.is_empty() {
        return None;
    }

    // Normalize the needle once for fuzzy scoring.
    let needle_norm = normalize_newlines(needle);

    // Determine "needle length" in lines for windowing.
    let n_lines = count_lines(&needle_norm).max(1);

    // Track best & second-best to detect ambiguous matches.
    let mut best_score: f32 = -1.0;
    let mut second_score: f32 = -1.0;
    let mut best_range: Option<(usize, usize)> = None;

    // Try windows of size n-1 ..= n+1 to tolerate +/- a line
    let win_min = n_lines.saturating_sub(1);
    let win_max = n_lines + 1;

    for win in win_min..=win_max {
        if win == 0 || ranges.len() < win { continue; }
        for i in 0..=ranges.len() - win {
            let start = ranges[i].0;
            let end   = ranges[i + win - 1].1;
            let slice = &haystack[start..end];

            // Score on normalized strings so CRLF/LF differences don't matter.
            let slice_norm = normalize_newlines(slice);
            let score = normalized_damerau_levenshtein(&slice_norm, &needle_norm) as f32;

            if score > best_score {
                second_score = best_score;
                best_score = score;
                best_range = Some((start, end));
            } else if score > second_score {
                second_score = score;
            }
        }
    }

    if let Some((start, end)) = best_range {
        if best_score >= min_score {
            // Treat near-ties as ambiguous instead of guessing wrong.
            if second_score >= 0.0 && (best_score - second_score) < 0.02 && second_score >= min_score {
                logger.info("matcher", "ambiguous_match", &format!("best={:.3}, second={:.3}", best_score, second_score));
                return None;
            }
            return Some(MatchResult { start, end, score: best_score });
        }
    }
    None
}

/// Return a vector of (start_byte, end_byte) for each logical line,
/// where end includes the newline if present. Handles both LF and CRLF.
fn line_ranges(s: &str) -> Vec<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut line_start = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\n' {
            // include '\n' in the line; also include '\r' if present before it
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

    // Special case: empty string -> no ranges
    out
}

/// Replace CRLF with LF for comparison/scoring.
fn normalize_newlines(s: &str) -> String {
    // This avoids allocating if there's no '\r'
    if s.as_bytes().contains(&b'\r') {
        s.replace("\r\n", "\n")
    } else {
        s.to_string()
    }
}

fn count_lines(s: &str) -> usize {
    // count '\n', but ensure at least 1 line for non-empty strings
    if s.is_empty() { 0 } else { s.lines().count().max(1) }
}
