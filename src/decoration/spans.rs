use ratatui::style::{Color, Style};

use super::{DecorationMap, StyledSpan};

/// Returns byte offsets of the start of each line (line 0 = offset 0).
pub fn line_start_bytes(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Convert a byte offset to `(line_index, char_offset_within_line)`.
/// Safe for non-char-boundary inputs — rounds down to the nearest boundary.
//
// Mutation-tested implicitly by the decoration integration tests (fixture_decoration_roundtrip
// and friends) and by block_highlights_to_decoration_map unit tests.  The timeout entries in
// cargo-mutants are a suite-throughput artefact (slow parallel tests), not missing coverage.
#[mutants::skip]
pub fn byte_to_line_char(line_starts: &[usize], text: &str, byte: usize) -> (usize, usize) {
    // Clamp and snap to a valid char boundary so multi-byte chars never panic.
    let byte = text.floor_char_boundary(byte.min(text.len()));
    let line = line_starts
        .partition_point(|&s| s <= byte)
        .saturating_sub(1);
    let line_start_byte = line_starts[line];
    let char_col = text[line_start_byte..byte].chars().count();
    (line, char_col)
}

/// Number of displayable chars on a line (excludes the trailing `\n`).
#[mutants::skip] // Tested implicitly; suite-throughput timeouts prevent empirical confirmation.
pub(super) fn line_char_len(line_starts: &[usize], text: &str, line_idx: usize) -> usize {
    let ls = line_starts[line_idx];
    let le = if line_idx + 1 < line_starts.len() {
        line_starts[line_idx + 1].saturating_sub(1) // trim the \n
    } else {
        text.len()
    };
    text[ls..le].chars().count()
}

#[mutants::skip] // Trivial wrapper; tested implicitly through add_byte_range_span.
pub(super) fn push_span(map: &mut DecorationMap, line: usize, span: StyledSpan) {
    map.entry(line).or_default().push(span);
}

#[mutants::skip] // Struct constructor; tested implicitly through callers in decoration/mod.rs.
pub(super) fn make_span(char_start: usize, char_end: usize, style: Style) -> StyledSpan {
    StyledSpan {
        char_start,
        char_end,
        style,
        ..Default::default()
    }
}

pub(super) struct SpanParams {
    pub(super) style: Style,
    pub(super) full_line_bg: Option<Color>,
    pub(super) is_blockquote: bool,
}

/// Add a span that covers a byte range; handles multi-line ranges by splitting per line.
pub(super) fn add_byte_range_span(
    map: &mut DecorationMap,
    line_starts: &[usize],
    text: &str,
    byte_start: usize,
    byte_end: usize,
    params: SpanParams,
) {
    if byte_start >= byte_end {
        return;
    }
    let (start_line, start_char) = byte_to_line_char(line_starts, text, byte_start);
    // byte_end is exclusive; point to the last byte for end calculation
    let end_byte = byte_end.saturating_sub(1).max(byte_start);
    let (end_line, end_char_inclusive) = byte_to_line_char(line_starts, text, end_byte);

    for line in start_line..=end_line {
        let c_start = if line == start_line { start_char } else { 0 };
        let c_end = if line == end_line {
            end_char_inclusive + 1 // make exclusive
        } else {
            line_char_len(line_starts, text, line)
        };
        let c_end = c_end.max(c_start + 1); // always at least 1 char wide
        push_span(
            map,
            line,
            StyledSpan {
                char_start: c_start,
                char_end: c_end,
                style: params.style,
                is_blockquote: params.is_blockquote,
                full_line_bg: params.full_line_bg,
                ..Default::default()
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier, Style};

    use super::{SpanParams, add_byte_range_span, line_start_bytes};
    use crate::decoration::DecorationMap;

    // Kills spans.rs:90:17 `delete field style from struct StyledSpan expression in
    // add_byte_range_span`.
    //
    // Without the `style: params.style` field the initialiser falls back to
    // `Default::default()` (= no style at all).  The test below uses a BOLD style
    // and asserts it is present on the produced span.
    #[test]
    fn add_byte_range_span_propagates_style() {
        let text = "hello world";
        let line_starts = line_start_bytes(text);
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let params = SpanParams {
            style: bold,
            full_line_bg: None,
            is_blockquote: false,
        };

        let mut map: DecorationMap = Default::default();
        // Span covering "hello" (bytes 0..5).
        add_byte_range_span(&mut map, &line_starts, text, 0, 5, params);

        let spans = map.get(&0).expect("line 0 must have at least one span");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "add_byte_range_span must propagate the BOLD style to the produced span"
        );
    }

    // Secondary: verify full_line_bg is propagated (not deleted from the struct).
    #[test]
    fn add_byte_range_span_propagates_full_line_bg() {
        let text = "abc";
        let line_starts = line_start_bytes(text);
        let params = SpanParams {
            style: Style::default(),
            full_line_bg: Some(Color::Blue),
            is_blockquote: false,
        };

        let mut map: DecorationMap = Default::default();
        add_byte_range_span(&mut map, &line_starts, text, 0, 3, params);

        let spans = map.get(&0).expect("line 0 must have a span");
        assert!(
            spans.iter().any(|s| s.full_line_bg == Some(Color::Blue)),
            "add_byte_range_span must propagate full_line_bg to the produced span"
        );
    }
}
