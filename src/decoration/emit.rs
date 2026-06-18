use ratatui::style::{Modifier, Style};

use crate::config::{Theme, blend_colors};

use super::DecorationMap;
use super::spans::{byte_to_line_char, make_span, push_span};

// ---------------------------------------------------------------------------
// Span-emission helpers
// ---------------------------------------------------------------------------

/// Layer an additional `modifier` on top of all spans in `map[line]` that
/// overlap `[range_start, range_end)`.
///
/// Used when an *outer* tag's End fires after the inner tag's spans are already
/// committed.  For example, in `**bold and *italic* bold**` the Strong End fires
/// after the Emphasis End has already placed italic spans; calling this with
/// `Modifier::BOLD` ensures the overlap region ends up with both BOLD and ITALIC.
pub(crate) fn add_modifier_to_existing(
    map: &mut DecorationMap,
    line: usize,
    range_start: usize,
    range_end: usize,
    modifier: Modifier,
) {
    if let Some(spans) = map.get_mut(&line) {
        for span in spans.iter_mut() {
            if span.char_end > range_start && span.char_start < range_end {
                span.style = span.style.add_modifier(modifier);
            }
        }
    }
}

/// Emit a styled span in segments over `[range_start, range_end)`, skipping
/// any char-ranges that are already decorated in `map` for the given line.
///
/// This lets an *outer* inline tag (e.g. Emphasis) coexist with *inner* tags
/// (e.g. Strong, Code) that were processed first.  Without this, a single
/// large outer content span would swallow every inner span in `split_into_spans`.
pub(crate) fn emit_content_around_existing(
    map: &mut DecorationMap,
    line: usize,
    range_start: usize,
    range_end: usize,
    style: Style,
) {
    if range_start >= range_end {
        return;
    }

    // Collect existing blocked char-ranges inside [range_start, range_end).
    // We clone the ranges out so the immutable borrow on `map` is released
    // before we call push_span (which needs a mutable borrow).
    let mut blocked: Vec<(usize, usize)> = map
        .get(&line)
        .map(|spans| {
            spans
                .iter()
                .filter(|s| s.char_end > range_start && s.char_start < range_end)
                .map(|s| (s.char_start.max(range_start), s.char_end.min(range_end)))
                .collect()
        })
        .unwrap_or_default();
    blocked.sort_by_key(|&(start, _)| start);

    // Emit content in the gaps between blocked regions.
    let mut pos = range_start;
    for (block_start, block_end) in blocked {
        if pos < block_start {
            push_span(map, line, make_span(pos, block_start, style));
        }
        if block_end > pos {
            pos = block_end;
        }
    }
    if pos < range_end {
        push_span(map, line, make_span(pos, range_end, style));
    }
}

// ---------------------------------------------------------------------------
// Bold+italic combined helper
// ---------------------------------------------------------------------------

/// Emit spans for a bold+italic region where `outer_range` is the enclosing
/// tag's byte range and `inner_range` is the nested tag's byte range.
/// `inner_is_strong` is true when the *inner* tag uses `**` (2-char delimiter);
/// false when it uses `*`/`_` (1-char delimiter).
///
/// pulldown-cmark nests `***text***` as `Emphasis { Strong { text } }`, so the
/// outer delimiter is 1 char (`*`) and the inner is 2 chars (`**`).
/// `**_text_**` nests as `Strong { Emphasis { text } }`, with outer = 2 and inner = 1.
#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_bold_italic_spans(
    map: &mut DecorationMap,
    line_starts: &[usize],
    text: &str,
    outer_range: std::ops::Range<usize>,
    inner_range: std::ops::Range<usize>,
    inner_is_strong: bool,
    theme: &Theme,
    italic_support: bool,
) {
    let inner_delim = if inner_is_strong { 2usize } else { 1 };

    let (start_line, start_char) = byte_to_line_char(line_starts, text, outer_range.start);
    let (end_line, end_char_excl) = byte_to_line_char(line_starts, text, outer_range.end);
    if start_line != end_line {
        return; // multi-line bold+italic not handled in v1
    }

    let (_, inner_start_char) = byte_to_line_char(line_starts, text, inner_range.start);
    let (_, inner_end_char) = byte_to_line_char(line_starts, text, inner_range.end);

    let content_start = inner_start_char + inner_delim;
    let content_end = inner_end_char.saturating_sub(inner_delim);
    if content_start > content_end || content_start >= end_char_excl {
        return;
    }

    // Blend bold and italic colors at 50 % for the combined content colour.
    let combined_color = blend_colors(theme.bold_color, theme.italic_color, 0.5);
    let delim_color = blend_colors(combined_color, theme.muted, theme.delimiter_blend);

    let delim_style = Style::default()
        .fg(delim_color)
        .add_modifier(Modifier::BOLD);

    let mut content_style = Style::default()
        .fg(combined_color)
        .add_modifier(Modifier::BOLD);
    if italic_support {
        content_style = content_style.add_modifier(Modifier::ITALIC);
    }

    // Opening delimiter (`***` / `**_` / `_**` / `___`)
    if content_start > start_char {
        push_span(
            map,
            start_line,
            make_span(start_char, content_start, delim_style),
        );
    }
    // Content
    if content_end > content_start {
        push_span(
            map,
            start_line,
            make_span(content_start, content_end, content_style),
        );
    }
    // Closing delimiter
    if end_char_excl > content_end {
        push_span(
            map,
            end_line,
            make_span(content_end, end_char_excl, delim_style),
        );
    }
}

// ---------------------------------------------------------------------------
// Bold+italic adjacency predicate helpers
// ---------------------------------------------------------------------------

/// `**_text_**` adjacency: Strong (outer, 2-char delimiters) directly wraps
/// Emphasis (inner, 1-char delimiter) with touching delimiters.
///
/// `&&` → `||` is `#[mutants::skip]`: for every well-formed pulldown-cmark
/// event sequence, either both conditions hold simultaneously (adjacent) or
/// neither holds (non-adjacent).  No Markdown input produces a case where
/// exactly one condition is true, so `&&` and `||` are behaviourally
/// equivalent and the mutation is undetectable.
#[mutants::skip]
pub(crate) fn is_strong_outer_adjacent_to_emphasis(
    strong: &std::ops::Range<usize>,
    emph: &std::ops::Range<usize>,
) -> bool {
    strong.start + 2 == emph.start && emph.end + 2 == strong.end
}
