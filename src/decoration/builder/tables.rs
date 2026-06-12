use ratatui::style::{Modifier, Style};

use crate::decoration::emit::emit_content_around_existing;
use crate::decoration::spans::{byte_to_line_char, line_char_len, make_span, push_span};

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- j. Tables ----
// ---------------------------------------------------------------------------

pub(super) fn on_table_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    let (start_line, _) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, _) = byte_to_line_char(
        &s.line_starts,
        s.text,
        range.end.saturating_sub(1).max(range.start),
    );
    let pipe_style = Style::default().fg(s.theme.muted);
    // Separator row: dashes blend into background; colons highlight alignment.
    let sep_dash_style = Style::default().fg(s.theme.muted);
    let sep_colon_style = Style::default().fg(s.theme.accent);

    for line in start_line..=end_line {
        let ls = s.line_starts[line];
        let le = if line + 1 < s.line_starts.len() {
            s.line_starts[line + 1].saturating_sub(1)
        } else {
            s.text.len()
        };
        let line_text = &s.text[ls..le];

        // A separator row contains only `|`, `:`, `-`, and whitespace,
        // with at least one dash.  Colons mark column alignment (GFM spec).
        let is_sep = line_text.contains('-')
            && line_text
                .chars()
                .all(|c| matches!(c, '|' | ':' | '-' | ' ' | '\t'));

        for (char_idx, c) in line_text.chars().enumerate() {
            match c {
                '|' => {
                    push_span(
                        &mut s.map,
                        line,
                        make_span(char_idx, char_idx + 1, pipe_style),
                    );
                }
                ':' if is_sep => {
                    push_span(
                        &mut s.map,
                        line,
                        make_span(char_idx, char_idx + 1, sep_colon_style),
                    );
                }
                '-' if is_sep => {
                    push_span(
                        &mut s.map,
                        line,
                        make_span(char_idx, char_idx + 1, sep_dash_style),
                    );
                }
                _ => {}
            }
        }
    }
}

pub(super) fn on_table_head_start(s: &mut BuildState, range: std::ops::Range<usize>) {
    // Capture the byte range; defer span emission to End(TableHead) so
    // that inline formatting spans (bold, italic, code) are already in
    // the map and `emit_content_around_existing` can skip over them.
    s.in_table_head = Some(range);
}

pub(super) fn on_table_head_end(s: &mut BuildState, _range: std::ops::Range<usize>) {
    if let Some(head_range) = s.in_table_head.take() {
        let style = Style::default()
            .fg(s.theme.accent)
            .add_modifier(Modifier::BOLD);

        let (start_line, _) = byte_to_line_char(&s.line_starts, s.text, head_range.start);
        let (end_line, _) = byte_to_line_char(
            &s.line_starts,
            s.text,
            head_range.end.saturating_sub(1).max(head_range.start),
        );

        for line in start_line..=end_line {
            let line_len = line_char_len(&s.line_starts, s.text, line);
            emit_content_around_existing(&mut s.map, line, 0, line_len, style);
        }
    }
}
