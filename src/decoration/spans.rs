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
pub(super) fn line_char_len(line_starts: &[usize], text: &str, line_idx: usize) -> usize {
    let ls = line_starts[line_idx];
    let le = if line_idx + 1 < line_starts.len() {
        line_starts[line_idx + 1].saturating_sub(1) // trim the \n
    } else {
        text.len()
    };
    text[ls..le].chars().count()
}

pub(super) fn push_span(map: &mut DecorationMap, line: usize, span: StyledSpan) {
    map.entry(line).or_default().push(span);
}

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
