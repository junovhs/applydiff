use crate::error::Result;
use crate::logger::Logger;

mod match_exact;
mod match_fuzzy;
mod match_normalize;

#[derive(Debug)]
pub struct MatchResult {
    pub start_byte: usize,
    pub end_byte: usize,
    pub score: f64,
}

/// Top-level matching strategy. Implements the progressive fallback logic.
///
/// # Panics
///
/// Panics if `min_score` is not between 0.1 and 1.0.
///
/// # Errors
///
/// Returns an error if no suitable match can be found for the `needle`.
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

    if needle.is_empty() {
        logger.info("matcher", "empty_needle", "Append/create mode");
        return Ok(MatchResult {
            start_byte: haystack.len(),
            end_byte: haystack.len(),
            score: 1.0,
        });
    }

    if let Some(result) = match_exact::try_exact_match(haystack, needle, logger) {
        return Ok(result);
    }

    let line_ranges = match_normalize::line_ranges(haystack);
    if line_ranges.is_empty() && !haystack.is_empty() {
        logger.error("matcher", "range_fail", "Failed to calculate line ranges for non-empty haystack");
        let line_ranges = vec![(0, haystack.len())];
        return match_fuzzy::find_fuzzy_match(haystack, needle, &line_ranges, min_score, logger);
    }

    match_fuzzy::find_fuzzy_match(haystack, needle, &line_ranges, min_score, logger)
}