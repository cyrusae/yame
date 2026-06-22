use pulldown_cmark::CowStr;
use ratatui::style::{Modifier, Style};

use crate::config::blend_colors;
use crate::decoration::spans::{byte_to_line_char, line_char_len, make_span, push_span};
use crate::decoration::types::StyledSpan;
use crate::decoration::words::link_split_char_idx;

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- g. Links ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_link(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, end_char_excl) = byte_to_line_char(&s.line_starts, s.text, range.end);

    // Only handle single-line links in v1
    if start_line == end_line {
        let link_text_slice = &s.text[range.start..range.end];
        let link_chars: Vec<char> = link_text_slice.chars().collect();

        if let Some(split_idx) = link_split_char_idx(&link_chars) {
            let delim_style = Style::default().fg(blend_colors(
                s.theme.link_text,
                s.theme.muted,
                s.theme.delimiter_blend,
            ));
            let text_style = Style::default()
                .fg(s.theme.link_text)
                .add_modifier(Modifier::UNDERLINED);
            let mut url_style = Style::default().fg(s.theme.link_url);
            if s.italic_support {
                url_style = url_style.add_modifier(Modifier::ITALIC);
            }

            // [ at start_char
            push_span(
                &mut s.map,
                start_line,
                make_span(start_char, start_char + 1, delim_style),
            );
            // text content
            if split_idx > 1 {
                push_span(
                    &mut s.map,
                    start_line,
                    make_span(start_char + 1, start_char + split_idx, text_style),
                );
            }
            // ] and ( around split
            push_span(
                &mut s.map,
                start_line,
                make_span(
                    start_char + split_idx,
                    start_char + split_idx + 2,
                    delim_style,
                ),
            );
            // url content
            let url_start = start_char + split_idx + 2;
            let url_end = end_char_excl.saturating_sub(1);
            if url_end > url_start {
                push_span(
                    &mut s.map,
                    start_line,
                    make_span(url_start, url_end, url_style),
                );
            }
            // closing )
            if end_char_excl > 0 {
                push_span(
                    &mut s.map,
                    end_line,
                    make_span(end_char_excl - 1, end_char_excl, delim_style),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ---- h. Images ----
// ---------------------------------------------------------------------------

// `![alt](path)` — the range from pulldown-cmark covers the full `![alt](path)` string,
// including the leading `!`.  We re-use `link_split_char_idx` which finds `](` at index 4
// for `![hi](img)` (skipping the leading `!` and `[`).
#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_image(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, end_char_excl) = byte_to_line_char(&s.line_starts, s.text, range.end);

    if start_line == end_line {
        let image_slice = &s.text[range.start..range.end];
        let image_chars: Vec<char> = image_slice.chars().collect();

        if let Some(split_idx) = link_split_char_idx(&image_chars) {
            let delim_style = Style::default().fg(blend_colors(
                s.theme.link_text,
                s.theme.muted,
                s.theme.delimiter_blend,
            ));
            let alt_style = Style::default().fg(s.theme.link_text);
            let mut path_style = Style::default()
                .fg(s.theme.link_url)
                .add_modifier(Modifier::UNDERLINED);
            if s.italic_support {
                path_style = path_style.add_modifier(Modifier::ITALIC);
            }

            // ![ opening (2-char delimiter)
            push_span(
                &mut s.map,
                start_line,
                make_span(start_char, start_char + 2, delim_style),
            );
            // alt text between [ and ]
            if split_idx > 2 {
                push_span(
                    &mut s.map,
                    start_line,
                    make_span(start_char + 2, start_char + split_idx, alt_style),
                );
            }
            // ]( delimiter
            push_span(
                &mut s.map,
                start_line,
                make_span(
                    start_char + split_idx,
                    start_char + split_idx + 2,
                    delim_style,
                ),
            );
            // path content
            let path_start = start_char + split_idx + 2;
            let path_end = end_char_excl.saturating_sub(1);
            if path_end > path_start {
                push_span(
                    &mut s.map,
                    start_line,
                    make_span(path_start, path_end, path_style),
                );
            }
            // closing )
            if end_char_excl > 0 {
                push_span(
                    &mut s.map,
                    end_line,
                    make_span(end_char_excl - 1, end_char_excl, delim_style),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ---- k. Strikethrough ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_strikethrough(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, end_char_excl) = byte_to_line_char(&s.line_starts, s.text, range.end);

    if start_line == end_line {
        let span_len = end_char_excl.saturating_sub(start_char);
        if span_len >= 4 {
            // ~~ delimiters use plain muted — blending toward text made them
            // brighter than the struck-through content they surround.
            let delim_style = Style::default().fg(s.theme.muted);
            let content_style = Style::default()
                .fg(s.theme.strikethrough_color)
                .add_modifier(Modifier::CROSSED_OUT);

            // opening ~~
            push_span(
                &mut s.map,
                start_line,
                make_span(start_char, start_char + 2, delim_style),
            );
            // content
            if start_char + 2 < end_char_excl.saturating_sub(2) {
                push_span(
                    &mut s.map,
                    start_line,
                    make_span(start_char + 2, end_char_excl - 2, content_style),
                );
            }
            // closing ~~
            push_span(
                &mut s.map,
                end_line,
                make_span(end_char_excl - 2, end_char_excl, delim_style),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ---- l. Horizontal rule ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_rule(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (rule_line, _) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let line_len = line_char_len(&s.line_starts, s.text, rule_line).max(1);
    push_span(
        &mut s.map,
        rule_line,
        StyledSpan {
            char_start: 0,
            char_end: line_len,
            style: Style::default().fg(s.theme.rule_color),
            is_rule: true,
            ..Default::default()
        },
    );
}

// ---------------------------------------------------------------------------
// ---- m. Word count — accumulate plain text events ----
// ---------------------------------------------------------------------------

pub(super) fn on_text(s: &mut BuildState, val: &CowStr, _range: std::ops::Range<usize>) {
    s.word_count += val.split_whitespace().count();
}
