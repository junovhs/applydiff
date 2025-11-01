use super::{MatchResult, Logger};

pub enum ExactMatch {
    None,
    Unique(MatchResult),
    Ambiguous,
}

/// Tier 1: Fast path for an exact substring match.
pub fn try_exact_match(haystack: &str, needle: &str, logger: &Logger) -> ExactMatch {
    let matches: Vec<_> = haystack.match_indices(needle).collect();
    if matches.len() == 1 {
        let (idx, matched_str) = matches[0];
        logger.info(
            "matcher",
            "exact_match_unique",
            &format!("Found unique exact match at byte {idx}"),
        );
        return ExactMatch::Unique(MatchResult {
            start_byte: idx,
            end_byte: idx + matched_str.len(),
            score: 1.0,
        });
    }

    if matches.len() > 1 {
        logger.info(
            "matcher",
            "exact_match_ambiguous",
            &format!("Found {} exact matches; forcing ambiguity error", matches.len()),
        );
        return ExactMatch::Ambiguous;
    }
    
    ExactMatch::None
}