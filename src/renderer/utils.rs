use ratatui::{style::Style, text::Span};

use crate::decoration::StyledSpan;

/// Shorten a file path to at most `max_components` trailing components.
/// e.g. "/home/user/docs/notes/foo.md" → "notes/foo.md" with max_components=2
pub fn shorten_path(path: &std::path::Path, max_components: usize) -> String {
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let start = components.len().saturating_sub(max_components);
    components[start..].join("/")
}

/// Format a usize with thousands separators.
/// e.g. 1204 → "1,204", 999 → "999"
pub fn format_thousands(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

/// Split a raw line string into ratatui `Span`s at decoration span boundaries.
///
/// `spans` need not be sorted or non-overlapping; this function sorts by
/// `char_start` and clips any later span that overlaps a prior one.
/// All indices are char indices (not byte indices).
pub fn split_into_spans(
    line: &str,
    spans: &[StyledSpan],
    default_style: Style,
) -> Vec<Span<'static>> {
    // Fast path — no decoration spans; skip the char_indices allocation entirely.
    if spans.is_empty() {
        return vec![Span::styled(line.to_owned(), default_style)];
    }

    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let char_count = chars.len();

    let mut sorted = spans.to_vec();
    sorted.sort_by_key(|s| s.char_start);

    let mut result: Vec<Span<'static>> = Vec::with_capacity(sorted.len() * 2 + 1);
    let mut char_pos = 0usize;

    for span in &sorted {
        let s_start = span.char_start.min(char_count);
        let s_end = span.char_end.min(char_count);
        let s_start = s_start.max(char_pos);

        if s_start >= s_end {
            continue;
        }

        // Unstyled gap before this span
        if char_pos < s_start {
            let byte_start = chars[char_pos].0;
            let byte_end = chars[s_start].0;
            result.push(Span::styled(
                line[byte_start..byte_end].to_owned(),
                default_style,
            ));
        }

        // Styled span content
        let byte_start = chars[s_start].0;
        let byte_end = if s_end < char_count {
            chars[s_end].0
        } else {
            line.len()
        };
        result.push(Span::styled(
            line[byte_start..byte_end].to_owned(),
            span.style,
        ));

        char_pos = s_end;
    }

    // Trailing unstyled tail
    if char_pos < char_count {
        let byte_start = chars[char_pos].0;
        result.push(Span::styled(line[byte_start..].to_owned(), default_style));
    }

    if result.is_empty() {
        result.push(Span::styled(String::new(), default_style));
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Style};

    use crate::decoration::StyledSpan;

    use super::{format_thousands, split_into_spans};

    // ── format_thousands ─────────────────────────────────────────────────────
    //
    // Kills renderer/utils.rs:24:26 `replace - with + in format_thousands`.
    //
    // The condition `(len - i).is_multiple_of(3)` places a comma every three
    // digits from the right.  With `+` it becomes `(len + i)`, which puts
    // commas at wrong positions (e.g. "1234" → "12,34" instead of "1,234").

    #[test]
    fn format_thousands_no_separator_below_1000() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(999), "999");
    }

    #[test]
    fn format_thousands_one_separator_at_1000() {
        // "1000": len=4, comma at i=1 (4-1=3, multiple of 3).
        // Mutation (len+i): 4+1=5, not multiple of 3 → no comma → "1000" ≠ "1,000".
        assert_eq!(format_thousands(1000), "1,000");
        assert_eq!(format_thousands(1234), "1,234");
    }

    #[test]
    fn format_thousands_two_separators() {
        assert_eq!(format_thousands(1_234_567), "1,234,567");
    }

    // ── split_into_spans ─────────────────────────────────────────────────────
    //
    // Kills renderer/utils.rs:61:20 `replace >= with < in split_into_spans`.
    //
    // `if s_start >= s_end { continue }` skips zero-width or reversed spans.
    // With `<`, the condition is true for every *valid* (s_start < s_end) span,
    // causing all of them to be skipped and the whole line to be returned unstyled.

    fn styled_span(char_start: usize, char_end: usize, color: Color) -> StyledSpan {
        StyledSpan {
            char_start,
            char_end,
            style: Style::default().fg(color),
            ..Default::default()
        }
    }

    #[test]
    fn split_into_spans_applies_single_span() {
        // "hello" with a Red span on chars 1..4 ("ell").
        // Mutation would skip the span → only one unstyled "hello" span returned.
        let spans = vec![styled_span(1, 4, Color::Red)];
        let result = split_into_spans("hello", &spans, Style::default());

        // Expect: "h" (default), "ell" (red), "o" (default) = 3 spans.
        assert_eq!(result.len(), 3, "must produce 3 segments: prefix, styled, suffix");
        assert_eq!(result[1].style.fg, Some(Color::Red), "middle span must be Red");
        assert_eq!(result[0].content, "h");
        assert_eq!(result[1].content, "ell");
        assert_eq!(result[2].content, "o");
    }

    #[test]
    fn split_into_spans_no_spans_returns_whole_line() {
        // Fast path: empty spans slice → single span with default style.
        let result = split_into_spans("hello", &[], Style::default());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "hello");
    }

    #[test]
    fn split_into_spans_zero_width_span_skipped() {
        // A span where char_start == char_end is zero-width and must be skipped.
        // This verifies the `s_start >= s_end` guard fires correctly.
        let spans = vec![styled_span(2, 2, Color::Green)]; // zero-width at char 2
        let result = split_into_spans("hello", &spans, Style::default());
        // Zero-width span is skipped → whole line returned as one unstyled span.
        assert_eq!(result.len(), 1, "zero-width span must be skipped");
        assert_eq!(result[0].content, "hello");
    }
}
