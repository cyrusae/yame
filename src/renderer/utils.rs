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
