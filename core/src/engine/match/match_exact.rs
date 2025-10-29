use super::{MatchResult, Logger};

/// Tier 1: Fast path for a unique, exact substring match.
pub fn try_exact_match(haystack: &str, needle: &str, logger: &Logger) -> Option<MatchResult> {
    let matches: Vec<_> = haystack.match_indices(needle).collect();
    if matches.len() == 1 {
        let (idx, matched_str) = matches[0];
        logger.info(
            "matcher",
            "exact_match_unique",
            &format!("Found unique exact match at byte {}", idx),
        );
        return Some(MatchResult {
            start_byte: idx,
            end_byte: idx + matched_str.len(),
            score: 1.0,
        });
    }

    if matches.len() > 1 {
        logger.info(
            "matcher",
            "exact_match_ambiguous",
            &format!("Found {} exact matches; fallback to fuzzy", matches.len()),
        );
    }
    
    None
}