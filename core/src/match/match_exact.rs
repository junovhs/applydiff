use crate::logger::Logger;
use super::MatchResult;

/// Fast path: check for a UNIQUE exact substring
pub fn try_exact_match(haystack: &str, needle: &str, logger: &Logger) -> Option<MatchResult> {
    let exact_matches: Vec<_> = haystack.match_indices(needle).collect();
    
    if exact_matches.len() == 1 {
        let (idx, _) = exact_matches[0];
        logger.info(
            "matcher",
            "fast_path_match",
            &format!("unique exact substring (len={})", needle.len())
        );
        return Some(MatchResult {
            start: idx,
            end: idx + needle.len(),
            score: 1.0,
        });
    }
    
    None
}