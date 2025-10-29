use crate::error::Result;
use crate::logger::Logger;

mod match_exact;
mod match_fuzzy;
mod match_normalize;

/// The result of a successful match, containing the location and confidence score.
#[derive(Debug)]
pub struct MatchResult {
    pub start_byte: usize,
    pub end_byte: usize,
    pub score: f64,
}

/// Top-level matching strategy. Implements the progressive fallback logic.
pub fn find_best_match(
    haystack: &str,
    needle: &str,
    min_score: f64,
    logger: &Logger,
) -> Result<MatchResult> {
    assert!(
        (0.1..=1.0).contains(&min_score),
        "min_score must be between 0.1 and 1.0"
    );
    logger.info(
        "matcher",
        "search_start",
        &format!("needle_len={}, min_score={}", needle.len(), min_score),
    );

    // An empty "from" block means we are creating or appending to a file.
    if needle.is_empty() {
        logger.info("matcher", "empty_needle", "Append/create mode");
        return Ok(MatchResult {
            start_byte: haystack.len(),
            end_byte: haystack.len(),
            score: 1.0,
        });
    }

    // Tier 1: Exact Substring Match (Fast Path)
    if let Some(result) = match_exact::try_exact_match(haystack, needle, logger) {
        return Ok(result);
    }

    // Prepare line ranges for windowed search
    let line_ranges = match_normalize::line_ranges(haystack);
    if line_ranges.is_empty() && !haystack.is_empty() {
        logger.error("matcher", "range_fail", "Failed to calculate line ranges for non-empty haystack");
        // Fallback for files without newlines: treat the whole file as one line.
        let line_ranges = vec![(0, haystack.len())];
        return match_fuzzy::find_fuzzy_match(haystack, needle, &line_ranges, min_score, logger);
    }

    // Tiers 2, 3, and 4
    match_fuzzy::find_fuzzy_match(haystack, needle, &line_ranges, min_score, logger)
}