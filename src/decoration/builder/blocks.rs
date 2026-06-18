use ratatui::style::Style;

use crate::decoration::spans::{byte_to_line_char, line_char_len, make_span, push_span};
use crate::decoration::types::StyledSpan;
use crate::decoration::words::count_chars_in;

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- f. Blockquotes ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_blockquote_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (start_line, _) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, _) = byte_to_line_char(
        &s.line_starts,
        s.text,
        range.end.saturating_sub(1).max(range.start),
    );

    let indicator_style = Style::default().fg(s.theme.muted);

    for line in start_line..=end_line {
        let line_len = line_char_len(&s.line_starts, s.text, line);
        if line_len == 0 {
            continue;
        }
        // ▌ indicator at char 0 (covers the `>` char visually).
        //
        // The rest of the line is intentionally left unspanned so that inline
        // decorations (bold, italic, code, links) emit their own spans without
        // a wide content span blocking `emit_content_around_existing`.
        //
        // The renderer detects `is_blockquote` on this indicator span and
        // applies `theme.blockquote_color` as the default fg for any
        // undecorated text on the line, preserving the blockquote visual style
        // while letting inline markup render at its own correct colors.
        push_span(
            &mut s.map,
            line,
            StyledSpan {
                char_start: 0,
                char_end: 1,
                style: indicator_style,
                is_blockquote: true,
                // Continuation visual rows indent 2 cols to align
                // with text start after `> `.
                continuation_indent: 2,
                ..Default::default()
            },
        );
    }
}

// ---------------------------------------------------------------------------
// ---- h. List items ----
// ---------------------------------------------------------------------------

pub(super) fn on_list_start(s: &mut BuildState, kind: Option<u64>) {
    s.in_ordered_list = kind.is_some();
}

pub(super) fn on_list_end(s: &mut BuildState) {
    s.in_ordered_list = false;
}

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_item_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (item_line, item_char) = byte_to_line_char(&s.line_starts, s.text, range.start);

    let bullet_style = Style::default().fg(s.theme.accent);
    let bullet_end = if s.in_ordered_list {
        let line_bytes_start = s.line_starts[item_line];
        let scan_start = range.start.saturating_sub(line_bytes_start);
        let line_text = &s.text[s.line_starts[item_line]..];
        line_text[scan_start..]
            .find(['.', ')'])
            .map(|i| item_char + count_chars_in(&line_text[scan_start..scan_start + i + 1]))
            .unwrap_or(item_char + 2)
    } else {
        item_char + 1
    };
    // continuation_indent = bullet_end + 1 so that soft-wrapped
    // continuation rows align with the item text (past bullet + space).
    let ci = (bullet_end + 1).min(255) as u8;
    push_span(
        &mut s.map,
        item_line,
        StyledSpan {
            char_start: item_char,
            char_end: bullet_end,
            style: bullet_style,
            continuation_indent: ci,
            ..Default::default()
        },
    );
}

// ---------------------------------------------------------------------------
// ---- i. Task-list markers ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_task_marker(s: &mut BuildState, checked: bool, range: std::ops::Range<usize>) {
    let (marker_line, marker_char) = byte_to_line_char(&s.line_starts, s.text, range.start);

    // The full task-list glyph is `- [ ] ` / `- [x] ` (marker_char chars
    // before `[`, then `[`, one char, `]`, space = 4 more chars).
    // Upgrade the bullet span's continuation_indent so that soft-wrapped
    // continuation rows align with the item text, not just the `- ` prefix.
    let task_ci = (marker_char + 4).min(255) as u8;
    if let Some(spans) = s.map.get_mut(&marker_line) {
        for span in spans.iter_mut() {
            if span.continuation_indent > 0 {
                span.continuation_indent = task_ci;
            }
        }
    }

    if checked {
        let line_len = line_char_len(&s.line_starts, s.text, marker_line);
        // [x] is 3 chars at marker_char: [ x ]
        let bracket_end = (marker_char + 3).min(line_len);
        let muted = Style::default().fg(s.theme.muted);
        let x_style = Style::default().fg(s.theme.text);
        // `[`
        push_span(
            &mut s.map,
            marker_line,
            make_span(marker_char, (marker_char + 1).min(bracket_end), muted),
        );
        // `x`
        if marker_char + 1 < bracket_end {
            push_span(
                &mut s.map,
                marker_line,
                make_span(marker_char + 1, (marker_char + 2).min(bracket_end), x_style),
            );
        }
        // `]`
        if marker_char + 2 < bracket_end {
            push_span(
                &mut s.map,
                marker_line,
                make_span(marker_char + 2, bracket_end, muted),
            );
        }
        // Item text after the bracket
        if bracket_end < line_len {
            push_span(
                &mut s.map,
                marker_line,
                make_span(
                    bracket_end,
                    line_len,
                    Style::default().fg(s.theme.todo_done),
                ),
            );
        }
    } else {
        // Style [ and ] in accent, leave space between as normal
        let accent = Style::default().fg(s.theme.accent);
        // `[ ]` is 3 chars: [, space, ]
        push_span(
            &mut s.map,
            marker_line,
            make_span(marker_char, marker_char + 1, accent),
        );
        push_span(
            &mut s.map,
            marker_line,
            make_span(marker_char + 2, marker_char + 3, accent),
        );
    }
}
