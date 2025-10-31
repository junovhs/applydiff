/// Returns a vector of (`start_byte`, `end_byte`) for each line in the string.
/// The `end_byte` includes the newline characters.
pub fn line_ranges(s: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c == '\n' {
            ranges.push((start, i + 1));
            start = i + 1;
        }
    }
    if start < s.len() {
        ranges.push((start, s.len()));
    }
    ranges
}

/// Normalizes all line endings to a single `\n` (LF).
pub fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Normalizes horizontal whitespace (spaces, tabs) on each line.
pub fn normalize_horizontal_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| {
            line.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Normalizes indentation by removing the common leading whitespace from all non-empty lines.
pub fn normalize_relative_indent(s: &str) -> String {
    let lines: Vec<_> = s.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return s.to_string();
    }

    lines
        .iter()
        .map(|line| {
            if line.len() > min_indent {
                &line[min_indent..]
            } else {
                line.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}