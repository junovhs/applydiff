use super::{
    match_normalize::{
        normalize_horizontal_whitespace, normalize_newlines, normalize_relative_indent,
    },
    Logger, MatchResult,
};
use crate::error::{ErrorCode, PatchError, Result};
use strsim::normalized_damerau_levenshtein as ndl;

/// Tiers 2, 3, and 4: Finds the best fuzzy match for the needle in the haystack.
pub fn find_fuzzy_match(
    haystack: &str,
    needle: &str,
    line_ranges: &[(usize, usize)],
    min_score: f64,
    logger: &Logger,
) -> Result<MatchResult> {
    // Tier 2: Whitespace-Normalized Equality
    let needle_ws_norm = normalize_horizontal_whitespace(&normalize_newlines(needle));
    let mut ws_matches = Vec::new();
    for &(start, end) in line_ranges {
        let slice = &haystack[start..end];
        let slice_ws_norm = normalize_horizontal_whitespace(&normalize_newlines(slice));
        if slice_ws_norm == needle_ws_norm {
            ws_matches.push((start, end));
        }
    }
    if ws_matches.len() == 1 {
        let (start, end) = ws_matches[0];
        logger.info("matcher", "ws_normalized_match", &format!("Found unique whitespace-normalized match at bytes {}-{}", start, end));
        return Ok(MatchResult { start_byte: start, end_byte: end, score: 1.0 });
    }

    // Tier 3: Relative-Indentation-Normalized Equality
    let needle_indent_norm = normalize_relative_indent(needle);
    let mut indent_matches = Vec::new();
    for &(start, end) in line_ranges {
        let slice = &haystack[start..end];
        if normalize_relative_indent(slice) == needle_indent_norm {
            indent_matches.push((start, end));
        }
    }
    if indent_matches.len() == 1 {
        let (start, end) = indent_matches[0];
        logger.info("matcher", "indent_normalized_match", &format!("Found unique indent-normalized match at bytes {}-{}", start, end));
        return Ok(MatchResult { start_byte: start, end_byte: end, score: 1.0 });
    }
    
    // Tier 4: Damerau-Levenshtein Fuzzy Search
    let needle_lines = normalize_newlines(needle).lines().count().max(1);
    let mut best_match: Option<MatchResult> = None;
    let mut second_best_score = -1.0;

    // Iterate through windows of lines in the haystack.
    // The window size is +/- 1 line from the needle's line count.
    for window_size in (needle_lines.saturating_sub(1))..=(needle_lines + 1) {
        if window_size == 0 || window_size > line_ranges.len() { continue; }

        for window in line_ranges.windows(window_size) {
            let start_byte = window[0].0;
            let end_byte = window[window_size - 1].1;
            let slice = &haystack[start_byte..end_byte];

            // Use normalized Damerau-Levenshtein for scoring.
            let score = ndl(&normalize_newlines(slice), &normalize_newlines(needle));

            if best_match.is_none() || score > best_match.as_ref().unwrap().score {
                if let Some(prev_best) = best_match.as_ref() {
                    second_best_score = prev_best.score;
                }
                best_match = Some(MatchResult { start_byte, end_byte, score });
            } else if score > second_best_score {
                second_best_score = score;
            }
        }
    }

    if let Some(bm) = best_match {
        // Ambiguity Guard: If the best and second-best scores are too close,
        // it's an ambiguous match, which is a Prediction Error.
        if (bm.score - second_best_score) < 0.02 && second_best_score > 0.0 {
            logger.error("matcher", "ambiguous_match", &format!("Ambiguous match detected. Best score: {:.2}, Second best: {:.2}", bm.score, second_best_score));
            return Err(PatchError::Apply {
                code: ErrorCode::AmbiguousMatch,
                message: "Ambiguous match detected. Multiple locations matched with similar confidence.".to_string(),
                file: Default::default(), // File path will be added by the Applier
            });
        }

        if bm.score >= min_score {
            logger.info("matcher", "fuzzy_match_success", &format!("Found fuzzy match with score {:.2} at bytes {}-{}", bm.score, bm.start_byte, bm.end_byte));
            return Ok(bm);
        }
    }

    logger.error("matcher", "no_match_found", "No suitable match found for the block.");
    Err(PatchError::Apply {
        code: ErrorCode::NoMatch,
        message: "No suitable match found for the block.".to_string(),
        file: Default::default(), // File path will be added by the Applier
    })
}