use pulldown_cmark::CowStr;
use ratatui::style::{Modifier, Style};

use crate::config::blend_colors;
use crate::decoration::emit::{
    add_modifier_to_existing, emit_bold_italic_spans, emit_content_around_existing,
    is_strong_outer_adjacent_to_emphasis,
};
use crate::decoration::spans::{
    SpanParams, add_byte_range_span, byte_to_line_char, make_span, push_span,
};

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- b. Bold — record range; emit on End(Strong) ----
// ---------------------------------------------------------------------------

pub(super) fn on_strong_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    s.in_strong = Some(range);
}

// ---------------------------------------------------------------------------
// ---- c. Italic — record range; emit on End(Emphasis) ----
// ---------------------------------------------------------------------------

pub(super) fn on_emphasis_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    s.in_emphasis = Some(range);
}

// ---------------------------------------------------------------------------
// ---- b/c end: emit bold, italic, or combined bold+italic ----
//
// Combined bold+italic is only triggered when the two tags are
// *directly adjacent* (delimiters touch with no intervening text).
// For `***text***`: Emphasis(0..N) wraps Strong(1..N-1) →
//   emph.start + 1 == strong.start  AND  strong.end + 1 == emph.end.
// For `**_text_**`: Strong(0..N) wraps Emphasis(2..N-2) →
//   strong.start + 2 == emph.start  AND  emph.end + 2 == strong.end.
//
// Non-adjacent nesting (`*italic **bold** rest*`) is two independent
// decorations; the outer tag's state is left in place for its own End.
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_strong_end(s: &mut BuildState, _range: std::ops::Range<usize>) {
    if let Some(strong_range) = s.in_strong.take() {
        // Peek at in_emphasis to check adjacency without consuming it.
        let adjacent = s.in_emphasis.as_ref().is_some_and(|emph| {
            emph.start + 1 == strong_range.start && strong_range.end + 1 == emph.end
        });
        if adjacent {
            // Emphasis(outer) wraps Strong(inner) with touching delimiters.
            let outer = s.in_emphasis.take().unwrap();
            emit_bold_italic_spans(
                &mut s.map,
                &s.line_starts,
                s.text,
                outer,
                strong_range,
                true, // inner_is_strong
                s.theme,
                s.italic_support,
            );
        } else {
            // Plain bold — non-adjacent or no Emphasis context at all.
            // Leave in_emphasis in place so its own End(Emphasis) fires later.
            let (start_line, start_char) =
                byte_to_line_char(&s.line_starts, s.text, strong_range.start);
            let (end_line, end_char_excl) =
                byte_to_line_char(&s.line_starts, s.text, strong_range.end);
            if start_line == end_line {
                let span_len = end_char_excl.saturating_sub(start_char);
                if span_len >= 4 {
                    let delim_style = Style::default()
                        .fg(blend_colors(
                            s.theme.text,
                            s.theme.muted,
                            s.theme.delimiter_blend,
                        ))
                        .add_modifier(Modifier::BOLD);
                    let content_style = Style::default()
                        .fg(s.theme.bold_color)
                        .add_modifier(Modifier::BOLD);
                    push_span(
                        &mut s.map,
                        start_line,
                        make_span(start_char, start_char + 2, delim_style),
                    );
                    emit_content_around_existing(
                        &mut s.map,
                        start_line,
                        start_char + 2,
                        end_char_excl.saturating_sub(2),
                        content_style,
                    );
                    push_span(
                        &mut s.map,
                        end_line,
                        make_span(end_char_excl - 2, end_char_excl, delim_style),
                    );
                    // Layer BOLD onto any inner spans (e.g. italic) in the
                    // bold content region so the overlap has both modifiers.
                    add_modifier_to_existing(
                        &mut s.map,
                        start_line,
                        start_char + 2,
                        end_char_excl.saturating_sub(2),
                        Modifier::BOLD,
                    );
                }
            }
        }
    }
}

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_emphasis_end(s: &mut BuildState, _range: std::ops::Range<usize>) {
    if let Some(emph_range) = s.in_emphasis.take() {
        // Peek at in_strong to check adjacency without consuming it.
        let adjacent = s
            .in_strong
            .as_ref()
            .is_some_and(|strong| is_strong_outer_adjacent_to_emphasis(strong, &emph_range));
        if adjacent {
            // Strong(outer) wraps Emphasis(inner) with touching delimiters.
            let outer = s.in_strong.take().unwrap();
            emit_bold_italic_spans(
                &mut s.map,
                &s.line_starts,
                s.text,
                outer,
                emph_range,
                false, // inner_is_strong = false (inner is Emphasis, 1-char delim)
                s.theme,
                s.italic_support,
            );
        } else {
            // Plain italic — non-adjacent or no Strong context at all.
            let (start_line, start_char) =
                byte_to_line_char(&s.line_starts, s.text, emph_range.start);
            let (end_line, end_char_excl) =
                byte_to_line_char(&s.line_starts, s.text, emph_range.end);
            if start_line == end_line {
                let span_len = end_char_excl.saturating_sub(start_char);
                if span_len >= 2 {
                    let delim_style = Style::default().fg(blend_colors(
                        s.theme.italic_color,
                        s.theme.muted,
                        s.theme.delimiter_blend,
                    ));
                    let mut content_style = Style::default().fg(s.theme.italic_color);
                    if s.italic_support {
                        content_style = content_style.add_modifier(Modifier::ITALIC);
                    }
                    push_span(
                        &mut s.map,
                        start_line,
                        make_span(start_char, start_char + 1, delim_style),
                    );
                    emit_content_around_existing(
                        &mut s.map,
                        start_line,
                        start_char + 1,
                        end_char_excl.saturating_sub(1),
                        content_style,
                    );
                    push_span(
                        &mut s.map,
                        end_line,
                        make_span(end_char_excl - 1, end_char_excl, delim_style),
                    );
                    // Layer ITALIC onto any inner spans (e.g. bold) in the
                    // italic content region so the overlap has both modifiers.
                    if s.italic_support {
                        add_modifier_to_existing(
                            &mut s.map,
                            start_line,
                            start_char + 1,
                            end_char_excl.saturating_sub(1),
                            Modifier::ITALIC,
                        );
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ---- d. Inline code ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_code(s: &mut BuildState, val: &CowStr, range: std::ops::Range<usize>) {
    s.word_count += val.split_whitespace().count();
    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, end_char_excl) = byte_to_line_char(&s.line_starts, s.text, range.end);
    let bg = s.in_heading_bg.unwrap_or(s.theme.code_bg);
    let code_style = Style::default().fg(s.theme.code_color).bg(bg);
    // Backtick delimiters blend toward muted (same standard as `*`, `[]()` etc.)
    let delim_style = Style::default()
        .fg(blend_colors(
            s.theme.code_color,
            s.theme.muted,
            s.theme.delimiter_blend,
        ))
        .bg(bg);

    if start_line == end_line {
        // Count the opening backtick run so we can split delimiters from content.
        let bt = s.text[range.start..range.end]
            .chars()
            .take_while(|&c| c == '`')
            .count()
            .max(1);
        let open_end = (start_char + bt).min(end_char_excl);
        let close_start = end_char_excl.saturating_sub(bt).max(open_end);

        // Opening backtick(s)
        push_span(
            &mut s.map,
            start_line,
            make_span(start_char, open_end, delim_style),
        );
        // Content between the backticks
        if open_end < close_start {
            push_span(
                &mut s.map,
                start_line,
                make_span(open_end, close_start, code_style),
            );
        }
        // Closing backtick(s)
        if close_start < end_char_excl {
            push_span(
                &mut s.map,
                start_line,
                make_span(close_start, end_char_excl, delim_style),
            );
        }
    } else {
        // Multi-line fallback (rare in practice — treat whole span uniformly).
        add_byte_range_span(
            &mut s.map,
            &s.line_starts,
            s.text,
            range.start,
            range.end,
            SpanParams {
                style: code_style,
                full_line_bg: None,
                is_blockquote: false,
            },
        );
    }
}
