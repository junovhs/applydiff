use crate::logger::Logger;
use super::{MatchResult, normalize_newlines, normalize_ws_preserve_newlines, normalize_relative_indent};
use strsim::normalized_damerau_levenshtein;

pub fn find_fuzzy_match(
    haystack: &str,
    needle: &str,
    ranges: &[(usize, usize)],
    win_min: usize,
    win_max: usize,
    min_score: f64,
    logger: &Logger,
) -> Option<MatchResult> {
    // 1) Whitespace-normalized equality
    let needle_ws = normalize_ws_preserve_newlines(needle);
    let matches = scan_windows_equal(ranges, haystack, &needle_ws, win_min, win_max, |s| {
        normalize_ws_preserve_newlines(s)
    });
    if matches.len() == 1 {
        let (start, end) = matches[0];
        logger.info("matcher", "normalized_ws_match", &format!("start={}, end={}", start, end));
        return Some(MatchResult { start, end, score: 1.0 });
    }

    // 2) Relative-indentation-normalized equality
    let needle_rel = normalize_relative_indent(&normalize_ws_preserve_newlines(needle));
    let matches = scan_windows_equal(ranges, haystack, &needle_rel, win_min, win_max, |s| {
        normalize_relative_indent(&normalize_ws_preserve_newlines(s))
    });
    if matches.len() == 1 {
        let (start, end) = matches[0];
        logger.info("matcher", "relative_indent_match", &format!("start={}, end={}", start, end));
        return Some(MatchResult { start, end, score: 1.0 });
    }

    // 3) Fuzzy match with Damerau-Levenshtein
    let needle_norm = normalize_newlines(needle);
    let mut best_score: f64 = -1.0;
    let mut second_score: f64 = -1.0;
    let mut best_range: Option<(usize, usize)> = None;

    for win in win_min..=win_max {
        if win == 0 || ranges.len() < win { continue; }
        
        for i in 0..=ranges.len() - win {
            let start = ranges[i].0;
            let end = ranges[i + win - 1].1;
            let slice_with_nl = &haystack[start..end];

            // Trim trailing newline from slice
            let mut slice = slice_with_nl;
            if slice.ends_with('\n') {
                slice = &slice[..slice.len() - 1];
                if slice.ends_with('\r') {
                    slice = &slice[..slice.len() - 1];
                }
            }

            // CRLF-insensitive scoring
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

    None
}

fn scan_windows_equal(
    ranges: &[(usize, usize)],
    haystack: &str,
    needle_xfm: &str,
    win_min: usize,
    win_max: usize,
    mut xfm: impl FnMut(&str) -> String,
) -> Vec<(usize, usize)> {
    let mut hits = Vec::new();
    if ranges.is_empty() { return hits; }
    
    for win in win_min..=win_max {
        if win == 0 || ranges.len() < win { continue; }
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