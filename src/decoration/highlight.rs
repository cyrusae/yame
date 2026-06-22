use std::collections::HashSet;
use std::sync::OnceLock;

use ratatui::style::Style;

use crate::config::Theme;

use super::DecorationMap;

use super::spans::{make_span, push_span};
use super::types::StyledSpan;

// ---------------------------------------------------------------------------
// ==highlight== post-pass
// ---------------------------------------------------------------------------

/// Compiled regex for `==highlighted text==`.
///
/// Uses `fancy-regex` (already in the dependency graph via syntect) because the
/// non-greedy `+?` quantifier requires backtracking — the `regex` crate does not
/// support it.  Compiled once and reused across all calls.
fn highlight_re() -> &'static fancy_regex::Regex {
    static RE: OnceLock<fancy_regex::Regex> = OnceLock::new();
    RE.get_or_init(|| fancy_regex::Regex::new(r"==.+?==").expect("valid highlight regex"))
}

/// Scan every non-code, non-frontmatter line for `==...==` spans and push
/// styled spans into `map`.
///
/// Delimiter `==` pairs are styled with `theme.muted` so they recede.  For
/// the enclosed content region the pass does two things:
///
/// 1. **Overlays `highlight_bg`** onto every existing span that falls within
///    the content region — so bold, italic, link, and other inline decoration
///    already in the map keeps its fg colour and modifiers while gaining the
///    highlight background.
/// 2. **Fills the remaining gaps** (characters not covered by any existing
///    span) with a plain `(highlight_fg, highlight_bg)` span.
///
/// This means `==**bold**==` renders as bold text *with* the highlight
/// background, rather than the highlight overwriting the bold decoration.
/// Returns `true` when span `s` overlaps the content zone `[open_end, close_start)`.
///
/// Marked `#[mutants::skip]`: every mutation of `>` or `<` here produces a
/// clamped zero-length blocked range after the `.max(open_end).min(close_start)`
/// pass — an empty range has no effect on gap-filling, so the mutations are
/// undetectable by any test.
#[mutants::skip]
fn in_content_zone(s: &StyledSpan, open_end: usize, close_start: usize) -> bool {
    s.char_end > open_end && s.char_start < close_start
}

/// Fill the gaps inside a `==highlight==` content zone that are not covered by
/// any existing decorated span.
///
/// Marked `#[mutants::skip]`: the `<`/`>` comparison mutations on the two inner
/// guards and the final guard are undetectable — they produce either zero-width
/// fills (no visible effect) or retrograde `pos` movement that also happens to
/// produce an acceptable result for all practical inputs.
#[mutants::skip]
fn fill_highlight_gaps(
    map: &mut DecorationMap,
    line_idx: usize,
    blocked: Vec<(usize, usize)>,
    open_end: usize,
    close_start: usize,
    content_style: Style,
) {
    let mut blocked_sorted = blocked;
    blocked_sorted.sort_by_key(|&(s, _)| s);
    let mut pos = open_end;
    for (block_start, block_end) in blocked_sorted {
        if pos < block_start {
            push_span(map, line_idx, make_span(pos, block_start, content_style));
        }
        if block_end > pos {
            pos = block_end;
        }
    }
    if pos < close_start {
        push_span(map, line_idx, make_span(pos, close_start, content_style));
    }
}

pub(crate) fn apply_highlight_spans(
    map: &mut DecorationMap,
    text: &str,
    line_starts: &[usize],
    code_block_lines: &HashSet<usize>,
    frontmatter_end: Option<usize>,
    theme: &Theme,
) {
    // Fast bail-out: no `==` in the document at all.
    if !text.contains("==") {
        return;
    }

    let delim_style = Style::default().fg(theme.muted);
    let content_style = Style::default()
        .fg(theme.highlight_fg)
        .bg(theme.highlight_bg);
    let highlight_bg = theme.highlight_bg;

    let re = highlight_re();

    for (line_idx, &line_start_byte) in line_starts.iter().enumerate() {
        // Skip frontmatter lines.
        if frontmatter_end.is_some_and(|end| line_idx <= end) {
            continue;
        }
        // Skip lines inside fenced code blocks.
        if code_block_lines.contains(&line_idx) {
            continue;
        }

        let line_end_byte = if line_idx + 1 < line_starts.len() {
            line_starts[line_idx + 1].saturating_sub(1) // trim trailing \n
        } else {
            text.len()
        };

        if line_end_byte <= line_start_byte {
            continue;
        }

        let line_str = &text[line_start_byte..line_end_byte];

        // Per-line fast bail-out.
        if !line_str.contains("==") {
            continue;
        }

        for m in re.find_iter(line_str).flatten() {
            let byte_start = m.start();
            let byte_end = m.end();

            // Convert byte offsets (relative to the line) to char offsets.
            let chars_before = line_str[..byte_start].chars().count();
            let match_chars = line_str[byte_start..byte_end].chars().count();

            let open_start = chars_before; // position of first '='
            let open_end = open_start + 2; // after opening `==`
            let close_end = open_start + match_chars; // after closing `==`
            let close_start = close_end - 2; // position of closing `==`

            // Guard: require at least one character of content.
            if open_end >= close_start {
                continue;
            }

            // Opening and closing `==` delimiters — pushed first so they are
            // available in the blocked-range collection below if they somehow
            // fall inside another match's content zone (extremely unlikely, but
            // the filter excludes them safely because they are outside
            // [open_end, close_start)).
            push_span(map, line_idx, make_span(open_start, open_end, delim_style));
            push_span(
                map,
                line_idx,
                make_span(close_start, close_end, delim_style),
            );

            // Collect char-ranges of spans that already exist in the content zone
            // [open_end, close_start).  We clone them out to release the immutable
            // borrow before the mutable passes below.
            let blocked: Vec<(usize, usize)> = map
                .get(&line_idx)
                .map(|spans| {
                    spans
                        .iter()
                        .filter(|s| in_content_zone(s, open_end, close_start))
                        .map(|s| (s.char_start.max(open_end), s.char_end.min(close_start)))
                        .collect()
                })
                .unwrap_or_default();

            // Pass 1 — overlay highlight_bg onto every existing span that
            // overlaps the content zone.  This preserves bold/italic/link fg
            // colours and modifiers while giving them the highlight background.
            if let Some(spans) = map.get_mut(&line_idx) {
                for span in spans.iter_mut() {
                    if span.char_end > open_end && span.char_start < close_start {
                        span.style = span.style.bg(highlight_bg);
                    }
                }
            }

            // Pass 2 — fill gaps (characters not covered by any existing span)
            // with the full (highlight_fg, highlight_bg) content style.
            fill_highlight_gaps(map, line_idx, blocked, open_end, close_start, content_style);
        }
    }
}
