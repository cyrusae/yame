use pulldown_cmark::{Event, Parser};

/// Count words in Markdown text, excluding syntax characters.
pub fn count_words(text: &str) -> usize {
    Parser::new(text)
        .filter_map(|e| match e {
            Event::Text(s) | Event::Code(s) => Some(s.split_whitespace().count()),
            _ => None,
        })
        .sum()
}

/// Find the `](` split point in a `[text](url)` char slice.
/// Returns the char index of `]`.
pub(super) fn link_split_char_idx(chars: &[char]) -> Option<usize> {
    let mut bracket_depth = 0usize;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => bracket_depth += 1,
            ']' if i + 1 < chars.len() && chars[i + 1] == '(' => {
                if bracket_depth <= 1 {
                    return Some(i);
                }
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Count chars in a `&str` slice (for ordered list marker scanning).
pub(super) fn count_chars_in(s: &str) -> usize {
    s.chars().count()
}
