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
        i += 1; // mutants::skip — i *= 1 causes an infinite loop (timeout, not a logic error)
    }
    None
}

/// Count chars in a `&str` slice (for ordered list marker scanning).
pub(super) fn count_chars_in(s: &str) -> usize {
    s.chars().count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    // ---- link_split_char_idx ----

    // Simple `[text](url)` — ']' is at index 5.
    // Kills: `i * 1` (second occurrence, col ~38) — `chars[5] = ']' ≠ '('` so guard fails → None.
    #[test]
    fn split_simple_link() {
        assert_eq!(link_split_char_idx(&ch("[text](url)")), Some(5));
    }

    // No `](` anywhere — loop exhausts all chars and returns None.
    // Kills: `while i <= len` (line 18) — at i == len, `chars[len]` panics.
    #[test]
    fn split_no_link_returns_none() {
        assert_eq!(link_split_char_idx(&ch("hello")), None);
    }

    // Nested brackets: `[[a](b)](c)` — inner `](` must be skipped; outer `]` at index 7.
    // Kills: `delete '[' arm` and `bracket_depth += 1` → `*= 1` (depth stays 0 → wrong early return).
    #[test]
    fn split_nested_brackets() {
        // chars: [ [ a ] ( b ) ] ( c )
        //        0 1 2 3 4 5 6 7 8 9
        assert_eq!(link_split_char_idx(&ch("[[a](b)](c)")), Some(7));
    }

    // `]` not followed by `(` appears before the real `](`.
    // Kills: `guard → true` — bare `]` at index 3 would fire, returning Some(3) instead of Some(6).
    #[test]
    fn split_bare_bracket_before_real_link() {
        // chars: [ t e ] x t ] (  u  r  l  )
        //        0 1 2 3 4 5 6 7  8  9 10 11
        assert_eq!(link_split_char_idx(&ch("[te]xt](url)")), Some(6));
    }

    // `]` is the last character — no char follows it.
    // Kills: `&&` → `||` (evaluates `chars[len]` → panic) and
    //        `i + 1 < len` → `i + 1 <= len` (same OOB access) and
    //        first `i + 1` → `i * 1` (then `chars[i+1]` is OOB → panic).
    #[test]
    fn split_bracket_at_end_no_url() {
        assert_eq!(link_split_char_idx(&ch("[text]")), None);
    }

    // `]` is the very first character (i == 0).
    // Kills: first `i + 1` → `i - 1` (`0usize - 1` underflows in debug → panic).
    #[test]
    fn split_bracket_paren_at_start() {
        // chars: ] ( u r l )  — bracket_depth starts at 0, which satisfies <= 1.
        //        0 1 2 3 4 5
        assert_eq!(link_split_char_idx(&ch("](url)")), Some(0));
    }

    // Empty slice — trivially returns None without entering the loop.
    #[test]
    fn split_empty() {
        assert_eq!(link_split_char_idx(&[]), None);
    }
}
