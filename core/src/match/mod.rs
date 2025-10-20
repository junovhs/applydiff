use crate::logger::Logger;

mod match_exact;
mod match_fuzzy;
mod match_normalize;

pub use match_exact::try_exact_match;
pub use match_fuzzy::find_fuzzy_match;
pub use match_normalize::{normalize_newlines, normalize_ws_preserve_newlines, normalize_relative_indent};

/// Result of locating the best match of `needle` within `haystack`
pub struct MatchResult {
    pub start: usize,
    pub end: usize,
    pub score: f64,
}

/// Top-level matching strategy (layered):
/// 1) Exact substring
/// 2) Whitespace-normalized equality
/// 3) Relative-indentation-normalized equality
/// 4) Fuzzy window search with ambiguity guard
pub fn find_best_match(
    haystack: &str,
    needle: &str,
    min_score: f64,
    logger: &Logger,
) -> Option<MatchResult> {
    if needle.is_empty() {
        return Some(MatchResult { start: haystack.len(), end: haystack.len(), score: 1.0 });
    }

    // Fast path: exact match
    if let Some(result) = try_exact_match(haystack, needle, logger) {
        return Some(result);
    }

    logger.info("matcher", "search_start", &format!("no exact match; layered search (needle_len={})", needle.len()));

    // Prepare line ranges
    let ranges = line_ranges(haystack);
    if ranges.is_empty() {
        logger.info("matcher", "empty_haystack", "no lines to search");
        return None;
    }

    // Calculate window sizes
    let needle_lines_norm = normalize_newlines(needle);
    let n_lines = count_lines(&needle_lines_norm).max(1);
    let win_min = n_lines.saturating_sub(1);
    let win_max = n_lines + 1;

    // Try fuzzy matching
    find_fuzzy_match(haystack, needle, &ranges, win_min, win_max, min_score, logger)
}

fn line_ranges(s: &str) -> Vec<(usize, usize)> {
    match_normalize::line_ranges(s)
}

fn count_lines(s: &str) -> usize {
    if s.is_empty() { 0 } else { s.lines().count().max(1) }
}