use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::decoration::{DecorationMap, StyledSpan};

mod status;
mod utils;

pub use status::{render_info_line, render_status_bar};
pub use utils::{format_thousands, shorten_path, split_into_spans};

// ---------------------------------------------------------------------------
// Soft-wrap helpers
// ---------------------------------------------------------------------------

/// Soft-wrap a string into visual rows of at most `width` **terminal columns**.
///
/// Wide characters (CJK, emoji, …) are counted by their display width (1 or 2
/// columns) rather than by Unicode scalar-value count.  Breaks at the last
/// space before `width`; falls back to a hard break when no space exists.
/// Returns byte-slices of `s` so no allocation is needed.
pub fn wrap_line(s: &str, width: usize) -> Vec<&str> {
    use unicode_width::UnicodeWidthChar;

    if s.is_empty() || width == 0 {
        return vec![s];
    }

    let char_indices: Vec<(usize, char)> = s.char_indices().collect();

    // Fast path: total display width fits on one row.
    let total_display: usize = char_indices
        .iter()
        .map(|&(_, c)| UnicodeWidthChar::width(c).unwrap_or(1))
        .sum();
    if total_display <= width {
        return vec![s];
    }

    let mut result: Vec<&str> = Vec::new();
    let mut char_start = 0usize; // index into char_indices

    loop {
        if char_start >= char_indices.len() {
            break;
        }

        // Check whether the rest fits without another break.
        let remaining_display: usize = char_indices[char_start..]
            .iter()
            .map(|&(_, c)| UnicodeWidthChar::width(c).unwrap_or(1))
            .sum();
        if remaining_display <= width {
            let byte_start = char_indices[char_start].0;
            result.push(&s[byte_start..]);
            break;
        }

        // Walk chars until adding the next one would exceed `width` display cols.
        let mut display_col = 0usize;
        let mut last_space: Option<usize> = None; // char_indices index of last fitting space
        let mut chunk_end = char_start; // exclusive end: first char that doesn't fit

        for (ci, &(_, c)) in char_indices[char_start..].iter().enumerate() {
            let cw = UnicodeWidthChar::width(c).unwrap_or(1);
            if display_col + cw > width {
                chunk_end = char_start + ci;
                break;
            }
            display_col += cw;
            chunk_end = char_start + ci + 1;
            if c == ' ' {
                last_space = Some(char_start + ci);
            }
        }

        if chunk_end == char_start {
            // Pathological: first char alone is wider than `width` (e.g. width=1
            // with a CJK char).  Hard-break after exactly one char to avoid an
            // infinite loop.
            let byte_start = char_indices[char_start].0;
            let byte_end = char_indices
                .get(char_start + 1)
                .map_or(s.len(), |&(b, _)| b);
            result.push(&s[byte_start..byte_end]);
            char_start += 1;
            continue;
        }

        // Prefer breaking at the last in-range space; fall back to hard break.
        let (break_at, next_start) = match last_space {
            Some(sp) => (sp, sp + 1),
            None => (chunk_end, chunk_end),
        };

        let byte_start = char_indices[char_start].0;
        let byte_end = char_indices.get(break_at).map_or(s.len(), |&(b, _)| b);

        if byte_start < byte_end {
            result.push(&s[byte_start..byte_end]);
        }
        char_start = next_start;
    }

    if result.is_empty() {
        result.push(s);
    }

    result
}

/// Soft-wrap with separate widths for the first row and continuation rows.
///
/// The first visual row is wrapped at `first_width` terminal columns. All
/// subsequent rows (which are indented by `first_width − cont_width` columns
/// during rendering) are wrapped at `cont_width` so they don't overflow the
/// right edge.
///
/// When `cont_width >= first_width` this degenerates to `wrap_line(s,
/// first_width)`.  All returned `&str` slices point into the original `s`
/// so [`wrap_char_ranges`] works unchanged.
pub fn wrap_line_indented(s: &str, first_width: usize, cont_width: usize) -> Vec<&str> {
    // Fast path: no effective indent, delegate entirely.
    if cont_width >= first_width {
        return wrap_line(s, first_width);
    }

    // Wrap the full string at first_width to peel off exactly the first row.
    let first_pass = wrap_line(s, first_width);
    if first_pass.len() <= 1 {
        // Everything fit on one row — no continuation rows needed.
        return first_pass;
    }

    let first_row = first_pass[0];
    // Byte position just after first_row in `s`.
    let first_end_byte = first_row.as_ptr() as usize - s.as_ptr() as usize + first_row.len();
    // `wrap_line` skips the break-space; skip it here too so we hand the
    // correct remainder to the next wrap pass.  Using `.get()` avoids an
    // explicit bounds check (which would be an equivalent mutant when the
    // condition is always true after the `first_pass.len() <= 1` guard).
    let rest_start = match s.as_bytes().get(first_end_byte) {
        Some(&b' ') => first_end_byte + 1,
        _ => first_end_byte,
    };
    let rest = &s[rest_start..];

    // Wrap the remainder at the narrower continuation width.
    let mut result = vec![first_row];
    result.extend(wrap_line(rest, cont_width.max(1)));
    result
}

/// Compute the `(char_start, char_len)` pair for each element of a
/// [`wrap_line`] (or [`wrap_line_indented`]) output slice, measured in char
/// units within `line`.
///
/// `wrap_line` breaks at spaces and **skips** the break-space. A naïve
/// approach of accumulating `char_len` produces incorrect char offsets for
/// every segment after the first soft-wrap break. This function corrects for
/// that by measuring the byte gap between segments (the skipped space).
pub fn wrap_char_ranges(line: &str, wrapped: &[&str]) -> Vec<(usize, usize)> {
    let base = line.as_ptr() as usize;
    let mut result = Vec::with_capacity(wrapped.len());
    let mut prev_byte_end = 0usize;
    let mut prev_char_end = 0usize;

    for row_str in wrapped {
        let seg_byte_start = row_str.as_ptr() as usize - base;
        let gap_chars = line[prev_byte_end..seg_byte_start].chars().count();
        let char_start = prev_char_end + gap_chars;
        let char_len = row_str.chars().count();
        result.push((char_start, char_len));
        prev_byte_end = seg_byte_start + row_str.len();
        prev_char_end = char_start + char_len;
    }

    result
}

/// Blank columns on each side of the text content within the editor column.
pub const GUTTER: u16 = 1;

// ---------------------------------------------------------------------------
// MarkdownView widget
// ---------------------------------------------------------------------------

/// The main editor rendering widget.
pub struct MarkdownView<'a> {
    pub lines: &'a [String],
    pub decoration_map: &'a DecorationMap,
    pub scroll_top: usize,
    pub cursor: (usize, usize),
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub theme: &'a crate::config::Theme,
    pub column_width: u16,
}

impl Widget for MarkdownView<'_> {
    #[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_width = (self.column_width as usize).saturating_sub(2 * GUTTER as usize);
        let visible = area.height as usize;
        let bg = self.theme.bg;
        let default_style = Style::default().fg(self.theme.text).bg(bg);

        // Flood-fill the entire area with the background colour.
        for row in 0..area.height {
            for col in 0..area.width {
                buf[(area.x + col, area.y + row)].set_bg(bg);
            }
        }

        let (cursor_log_row, cursor_log_col) = self.cursor;
        let mut cursor_buf_pos: Option<(u16, u16)> = None;

        let mut visual_row: usize = 0;
        let total = self.lines.len();
        let mut log_row = self.scroll_top;

        while visual_row < visible && log_row < total {
            let line = &self.lines[log_row];
            let line_decs = self.decoration_map.get(&log_row);
            // Compute the continuation indent before wrapping so continuation
            // rows are wrapped at the narrower effective width.
            let line_ci = line_decs
                .map(|decs| decs.iter().map(|s| s.continuation_indent).max().unwrap_or(0))
                .unwrap_or(0) as usize;
            let wrapped = wrap_line_indented(
                line,
                content_width.max(1),
                content_width.saturating_sub(line_ci).max(1),
            );

            let char_ranges = wrap_char_ranges(line, &wrapped);
            for (wrap_idx, (&row_str, &(char_start, char_len))) in
                wrapped.iter().zip(char_ranges.iter()).enumerate()
            {
                if visual_row >= visible {
                    break;
                }

                let char_end = char_start + char_len;

                // Adjust decoration spans to this visual row's char range.
                // Built first so continuation_indent is available for cursor tracking.
                let row_spans: Vec<StyledSpan> = line_decs
                    .map(|decs| {
                        decs.iter()
                            .filter(|s| s.char_end > char_start && s.char_start < char_end)
                            .map(|s| StyledSpan {
                                char_start: s.char_start.saturating_sub(char_start),
                                char_end: s.char_end.saturating_sub(char_start).min(char_len),
                                style: s.style,
                                is_blockquote: s.is_blockquote,
                                continuation_indent: s.continuation_indent,
                                full_line_bg: s.full_line_bg,
                                border_bottom: s.border_bottom,
                                is_rule: s.is_rule,
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Continuation rows (wrap_idx > 0) of blockquote/list lines are
                // indented to align with the text start after the `> ` / bullet prefix.
                //
                // IMPORTANT: read from `line_decs` (all logical-line spans), NOT from
                // `row_spans` (filtered to visual-row char range).  The bullet/indicator
                // span covers chars 0..1, which is outside the char range of any
                // continuation row — so it would be stripped from `row_spans` and the
                // indent would compute to zero.
                let continuation_indent: u16 = if wrap_idx > 0 {
                    line_decs
                        .map(|decs| {
                            decs.iter()
                                .map(|s| s.continuation_indent)
                                .max()
                                .unwrap_or(0)
                        })
                        .unwrap_or(0) as u16
                } else {
                    0
                };

                // Cursor tracking
                if log_row == cursor_log_row {
                    let is_last_wrap = wrap_idx + 1 == wrapped.len();
                    let in_range =
                        cursor_log_col >= char_start && (cursor_log_col < char_end || is_last_wrap);
                    if in_range {
                        let col_in_row = (cursor_log_col.saturating_sub(char_start))
                            .min(content_width.saturating_sub(1));
                        cursor_buf_pos = Some((
                            area.x + GUTTER + continuation_indent + col_in_row as u16,
                            area.y + visual_row as u16,
                        ));
                    }
                }

                let line_bg = row_spans.iter().find_map(|s| s.full_line_bg).unwrap_or(bg);
                let border_color = row_spans.iter().find_map(|s| s.border_bottom);
                let is_rule = row_spans.iter().any(|s| s.is_rule);
                let is_last_wrap = wrap_idx + 1 == wrapped.len();

                let y = area.y + visual_row as u16;

                for col in 0..self.column_width {
                    buf[(area.x + col, y)].set_bg(line_bg);
                }

                if is_rule {
                    let rule_style = row_spans
                        .iter()
                        .find(|s| s.is_rule)
                        .map(|s| s.style)
                        .unwrap_or(default_style);
                    for x in area.x + GUTTER..area.x + GUTTER + content_width as u16 {
                        buf[(x, y)].set_char('─').set_style(rule_style);
                    }
                } else {
                    use unicode_width::UnicodeWidthChar;
                    // Blockquote lines: use blockquote_color as the default fg for any
                    // undecorated text. The wide content span was removed from the
                    // decoration pipeline (fix for #120) so that inline markup (bold,
                    // italic, etc.) can emit their own correctly-colored spans. The
                    // renderer restores the blockquote color here for plain text gaps.
                    // Read from `line_decs` (all logical spans), not `row_spans` (filtered
                    // to the visual row's char range), so continuation rows also apply it.
                    let is_blockquote_line = line_decs
                        .map(|decs| decs.iter().any(|s| s.is_blockquote))
                        .unwrap_or(false);
                    let row_default = if is_blockquote_line {
                        Style::default()
                            .fg(self.theme.blockquote_color)
                            .bg(line_bg)
                    } else {
                        default_style.bg(line_bg)
                    };
                    let segments = split_into_spans(row_str, &row_spans, row_default);
                    let mut x = area.x + GUTTER + continuation_indent;
                    for span in &segments {
                        for ch in span.content.chars() {
                            let cw = UnicodeWidthChar::width(ch).unwrap_or(1);
                            // Stop before this char would overflow the content area.
                            if (x.saturating_sub(area.x + GUTTER)) as usize + cw > content_width {
                                break;
                            }
                            buf[(x, y)].set_char(ch).set_style(span.style);
                            if cw == 2 {
                                // Explicitly clear the second terminal column of the
                                // wide char.  Without this, ratatui's frame-diff never
                                // "owns" that cell, so when the line scrolls away the
                                // cell can retain stale content from the previous frame.
                                let x2 = x + 1;
                                if ((x2.saturating_sub(area.x + GUTTER)) as usize) < content_width {
                                    buf[(x2, y)].set_char(' ').set_style(span.style);
                                }
                            }
                            x += cw as u16;
                        }
                    }
                }

                // Bottom border (H1–H3 heading underline, last visual row only)
                if let Some(bc) = border_color
                    && is_last_wrap
                {
                    use ratatui::layout::Rect as R;
                    buf.set_style(
                        R {
                            x: area.x,
                            y,
                            width: self.column_width,
                            height: 1,
                        },
                        Style::default()
                            .underline_color(bc)
                            .add_modifier(Modifier::UNDERLINED),
                    );
                }

                visual_row += 1;
            }

            log_row += 1;
        }

        // Selection overlay — applied after content, before cursor.
        if let Some(selection) = self.selection {
            apply_selection_overlay(area, buf, &self, selection);
        }

        // Cursor cell — always on top.
        if let Some((cx, cy)) = cursor_buf_pos {
            buf[(cx, cy)]
                .set_fg(self.theme.bg)
                .set_bg(self.theme.accent);
        }
    }
}

fn apply_selection_overlay(
    area: Rect,
    buf: &mut Buffer,
    view: &MarkdownView<'_>,
    selection: ((usize, usize), (usize, usize)),
) {
    let ((sel_row_start, sel_col_start), (sel_row_end, sel_col_end)) = selection;
    // Written as GUTTER + GUTTER (not 2 * GUTTER) so that operator-replacement
    // mutants produce an observable difference: `+→-` gives 0, `+→*` gives 1.
    let content_width = (view.column_width as usize).saturating_sub(GUTTER as usize + GUTTER as usize);
    let visible = area.height as usize;
    let total = view.lines.len();
    let sel_fg = view.theme.selection_fg;
    let sel_bg = view.theme.selection_bg;

    let mut visual_row: usize = 0;
    let mut log_row = view.scroll_top;

    // Equivalent mutation note: `visual_row < visible` has an untestable `<→<=`
    // because the inner `if visual_row >= visible { break; }` fires immediately
    // on the extra iteration, making it a behavioural no-op.
    while visual_row < visible && log_row < total {
        if log_row > sel_row_end {
            break;
        }

        let line = &view.lines[log_row];
        let line_ci = view
            .decoration_map
            .get(&log_row)
            .map(|decs| decs.iter().map(|s| s.continuation_indent).max().unwrap_or(0))
            .unwrap_or(0) as usize;
        let wrapped = wrap_line_indented(
            line,
            content_width.max(1),
            content_width.saturating_sub(line_ci).max(1),
        );

        let char_ranges = wrap_char_ranges(line, &wrapped);
        for (wrap_idx, (_row_str, &(char_start, char_len))) in
            wrapped.iter().zip(char_ranges.iter()).enumerate()
        {
            if visual_row >= visible {
                break;
            }

            let char_end = char_start + char_len;

            // Mirror the continuation_indent logic from render() so selection
            // highlighting respects the same left-margin indent.
            let continuation_indent: u16 = if wrap_idx > 0 {
                view.decoration_map
                    .get(&log_row)
                    .map(|decs| {
                        decs.iter()
                            .map(|s| s.continuation_indent)
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(0) as u16
            } else {
                0
            };

            if log_row >= sel_row_start && log_row <= sel_row_end {
                let line_sel_start = if log_row == sel_row_start {
                    sel_col_start
                } else {
                    0
                };
                let line_sel_end = if log_row == sel_row_end {
                    sel_col_end
                } else {
                    usize::MAX
                };

                let row_sel_start = line_sel_start.max(char_start);
                let row_sel_end = line_sel_end.min(char_end);

                // Equivalent mutation note: `<→<=` is untestable because when
                // row_sel_start == row_sel_end the ci term makes x_start >= x_end,
                // so the for loop produces an empty range regardless.
                if row_sel_start < row_sel_end {
                    let y = area.y + visual_row as u16;
                    let x_start =
                        area.x + GUTTER + continuation_indent + (row_sel_start - char_start) as u16;
                    let x_end = (area.x + GUTTER + (row_sel_end - char_start) as u16)
                        .min(area.x + GUTTER + content_width as u16);
                    for x in x_start..x_end {
                        buf[(x, y)].set_fg(sel_fg).set_bg(sel_bg);
                    }
                }
            }

            visual_row += 1;
        }

        log_row += 1;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use tui_textarea::TextArea;

    use ratatui::style::{Modifier, Style};

    use crate::app::App;
    use crate::config::Theme;
    use crate::decoration::{DecorationMap, StyledSpan};
    use crate::status::StatusLine;

    use super::status::{build_normal_status_bar, build_timed_message_bar};
    use super::*;

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("notes/foo.md"),
            is_dirty: false,
            saved_content: None,
            theme: Theme::default_theme(),
            italic_support: false,
            powerline_glyphs: false,
            last_keystroke: None,
            force_redecorate: false,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
            free_scroll: false,
            clipboard: None,
            initial_file_empty: false,
        }
    }

    // --- shorten_path ---

    #[test]
    fn shorten_path_long() {
        let p = Path::new("/home/user/docs/notes/foo.md");
        assert_eq!(shorten_path(p, 2), "notes/foo.md");
    }

    #[test]
    fn shorten_path_short_stays_whole() {
        let p = Path::new("foo.md");
        assert_eq!(shorten_path(p, 3), "foo.md");
    }

    #[test]
    fn shorten_path_exact_components() {
        let p = Path::new("/a/b/c");
        assert_eq!(shorten_path(p, 3), "a/b/c");
    }

    #[test]
    fn shorten_path_more_components_than_max() {
        let p = Path::new("/home/user/projects/yame/src/main.rs");
        let result = shorten_path(p, 3);
        assert_eq!(result, "yame/src/main.rs");
    }

    // --- format_thousands ---

    #[test]
    fn format_thousands_small() {
        assert_eq!(format_thousands(0), "0");
        assert_eq!(format_thousands(999), "999");
    }

    #[test]
    fn format_thousands_1204() {
        assert_eq!(format_thousands(1204), "1,204");
    }

    #[test]
    fn format_thousands_million() {
        assert_eq!(format_thousands(1_000_000), "1,000,000");
    }

    #[test]
    fn format_thousands_exactly_1000() {
        assert_eq!(format_thousands(1000), "1,000");
    }

    // --- split_into_spans ---

    fn bold_style() -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    #[test]
    fn span_split_no_spans_returns_whole_line() {
        let result = split_into_spans("hello world", &[], Style::default());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "hello world");
    }

    #[test]
    fn span_split_mid_span_correct_boundaries() {
        let span = StyledSpan {
            char_start: 2,
            char_end: 5,
            style: bold_style(),
            ..Default::default()
        };
        let result = split_into_spans("0123456789", &[span], Style::default());
        assert_eq!(result[0].content, "01");
        assert_eq!(result[1].content, "234");
        assert_eq!(result[2].content, "56789");
    }

    #[test]
    fn span_split_multibyte_safe() {
        let span = StyledSpan {
            char_start: 1,
            char_end: 2,
            ..Default::default()
        };
        let result = split_into_spans("café", &[span], Style::default());
        assert_eq!(result[0].content, "c");
        assert_eq!(result[1].content, "a");
    }

    #[test]
    fn span_split_full_line_span() {
        let span = StyledSpan {
            char_start: 0,
            char_end: 5,
            style: bold_style(),
            ..Default::default()
        };
        let result = split_into_spans("hello", &[span], Style::default());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "hello");
    }

    #[test]
    fn span_split_empty_line_returns_one_span() {
        let result = split_into_spans("", &[], Style::default());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "");
    }

    #[test]
    fn span_split_overlapping_clips_later_span() {
        let s1 = StyledSpan {
            char_start: 0,
            char_end: 5,
            style: bold_style(),
            ..Default::default()
        };
        let s2 = StyledSpan {
            char_start: 3,
            char_end: 8,
            ..Default::default()
        };
        let result = split_into_spans("0123456789", &[s1, s2], Style::default());
        assert_eq!(result[0].content, "01234");
        assert_eq!(result[1].content, "567");
        assert_eq!(result[2].content, "89");
    }

    // --- wrap_line ---

    #[test]
    fn wrap_short_line_unchanged() {
        assert_eq!(wrap_line("hello", 40), vec!["hello"]);
    }

    #[test]
    fn wrap_breaks_at_word_boundary() {
        let result = wrap_line("one two three four five", 12);
        assert!(result[0].len() <= 12);
        assert!(result.iter().all(|s| s.len() <= 12));
    }

    #[test]
    fn wrap_hard_breaks_unbreakable_word() {
        let long = "abcdefghijklmnop";
        let result = wrap_line(long, 8);
        assert_eq!(result[0], "abcdefgh");
        assert_eq!(result[1], "ijklmnop");
    }

    #[test]
    fn wrap_wide_chars_count_display_columns() {
        // Each CJK char is 2 display columns wide.
        // "日本語テ" = 8 display cols; at width=4 it wraps into two rows of 2 chars each.
        let result = wrap_line("日本語テ", 4);
        assert_eq!(
            result.len(),
            2,
            "4 CJK chars (8 display cols) must wrap at width=4"
        );
        assert_eq!(result[0], "日本");
        assert_eq!(result[1], "語テ");
    }

    #[test]
    fn wrap_wide_chars_single_row_when_fits() {
        // "日本" = 4 display cols; at width=4 it fits without wrapping.
        let result = wrap_line("日本", 4);
        assert_eq!(
            result.len(),
            1,
            "2 CJK chars (4 display cols) must not wrap at width=4"
        );
    }

    #[test]
    fn wrap_empty_string() {
        let result = wrap_line("", 40);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn wrap_exact_width_no_break() {
        let result = wrap_line("abcde", 5);
        assert_eq!(result, vec!["abcde"]);
    }

    #[test]
    fn wrap_single_word_over_width_hard_breaks() {
        let result = wrap_line("abcdef", 4);
        assert_eq!(result[0], "abcd");
        assert_eq!(result[1], "ef");
    }

    #[test]
    fn wrap_zero_width_returns_whole_string() {
        // Early-exit uses `||`: both `s.is_empty()` and `width == 0` independently
        // trigger it.  With ||→&& mutation, width=0 no longer short-circuits and
        // the function hard-breaks each character individually instead.
        assert_eq!(wrap_line("hello", 0), vec!["hello"]);
    }

    #[test]
    fn wrap_soft_break_exact_content() {
        // Exercises the soft-break path (break at last space before width).
        // Kills line-96 `sp+1→sp-1` (next_start goes backward → includes "z abc")
        // and `sp+1→sp*1` (next_start stays at space → includes " abc").
        assert_eq!(wrap_line("xyz abc", 5), vec!["xyz", "abc"]);
    }

    #[test]
    fn wrap_cjk_pathological_single_col() {
        // CJK chars are 2 display columns; width=1 forces the pathological path where
        // the first char alone exceeds the limit (chunk_end == char_start).
        // Kills line-87 `+→*` (byte_end = byte_start → empty segment pushed) and
        // `+→-` (0usize - 1 underflows in debug → panic).
        // Also kills line-90 `-=` (char_start underflows after the first hard-break).
        assert_eq!(wrap_line("日", 1), vec!["日"]);
    }

    #[test]
    fn wrap_skips_empty_span_at_space_boundary() {
        // After wrapping "a" the next segment starts at the space; that space becomes
        // the break point so byte_start == byte_end.  The `if byte_start < byte_end`
        // guard must skip it.  With `<→<=` an empty string slice is pushed instead,
        // giving ["a", "", "b"].
        assert_eq!(wrap_line("a b", 1), vec!["a", "b"]);
    }

    // --- wrap_line_indented ---

    #[test]
    fn wrap_indented_no_indent_same_as_wrap_line() {
        // cont_width >= first_width → fast path, identical to wrap_line.
        let s = "one two three four five";
        assert_eq!(wrap_line_indented(s, 12, 12), wrap_line(s, 12));
    }

    #[test]
    fn wrap_indented_single_row_unchanged() {
        // Line fits on one row — no continuation rows, no clipping.
        assert_eq!(wrap_line_indented("hello", 40, 38), vec!["hello"]);
    }

    #[test]
    fn wrap_indented_continuation_row_narrower() {
        // "- word1 word2 word3" with first_width=20, cont_width=18 (indent=2).
        // The first row gets up to 20 cols; continuation row(s) get up to 18.
        // This test verifies continuation rows don't exceed cont_width characters.
        let s = "- word1 word2 word3 word4 word5 word6 word7";
        let rows = wrap_line_indented(s, 20, 18);
        assert!(rows.len() >= 2, "expected wrapping to produce multiple rows");
        // First row may use up to first_width.
        assert!(rows[0].len() <= 20, "first row too wide: {:?}", rows[0]);
        // All continuation rows must fit within cont_width.
        for row in &rows[1..] {
            assert!(
                row.len() <= 18,
                "continuation row exceeds cont_width: {:?}",
                row
            );
        }
    }

    #[test]
    fn wrap_indented_roundtrips_via_char_ranges() {
        // All &str slices returned by wrap_line_indented point into the original
        // string, so wrap_char_ranges must be able to locate them.
        let s = "- some indented list item that is long enough to wrap at a narrow width";
        let rows = wrap_line_indented(s, 30, 28);
        // Must not panic and char ranges must be non-overlapping ascending.
        let ranges = wrap_char_ranges(s, &rows);
        let mut prev_end = 0usize;
        for (start, len) in &ranges {
            assert!(*start >= prev_end, "ranges overlap or go backward");
            prev_end = start + len;
        }
    }

    #[test]
    fn wrap_indented_hard_break_continuation() {
        // An unbreakable word on a continuation row should still hard-break
        // within cont_width, not overflow to first_width.
        let s = "- abcdefghijklmnopqrstuvwx";
        // first_width=20, cont_width=18.  "- " is 2 chars on row 0; the 24-char
        // tail must hard-break at 18 on the continuation row.
        let rows = wrap_line_indented(s, 20, 18);
        assert!(rows.len() >= 2);
        for row in &rows[1..] {
            assert!(row.len() <= 18, "continuation hard-break too wide: {:?}", row);
        }
    }

    // Kills: renderer/mod.rs:144 (match arm Some(&b' ') mutations).
    // If the break-space is NOT skipped, rest starts with ' ' and wrap_line
    // produces a continuation row with a leading space.
    // Specifically kills:
    //   old `< → ==` / `< → >` (condition becomes false → else branch → space not skipped),
    //   old `+ → *` (first_end_byte * 1 == first_end_byte → same as else branch),
    //   new `Some(&b' ') → Some(&b'\0')` (space not matched → else branch),
    //   new `+ 1 → * 1` in match arm (rest_start = first_end_byte → space included).
    #[test]
    fn wrap_indented_continuation_rows_have_no_leading_space() {
        // "- abc def ghi" with first_width=8, cont_width=6.
        // first_pass wraps at a space; rest must start AFTER the space.
        // If the space is not skipped, continuation rows begin with ' '.
        let s = "- abc def ghi jkl";
        let rows = wrap_line_indented(s, 8, 6);
        assert!(rows.len() >= 2, "expected wrapping to produce multiple rows");
        for row in &rows[1..] {
            assert!(
                !row.starts_with(' '),
                "continuation row must not start with space (break-space must be skipped): {:?}",
                row
            );
        }
    }

    // --- wrap_char_ranges ---

    #[test]
    fn char_ranges_two_segments() {
        // "hello world" at width 5 → ["hello", "world"].
        // Kills all four wrap_char_ranges arithmetic mutants:
        //   line 132 +→- and +→* (wrong char_start for segment 2),
        //   line 135 +→*           (wrong prev_byte_end → inflated gap → wrong char_start),
        //   line 136 +→*           (wrong prev_char_end → wrong char_start for segment 2).
        let line = "hello world";
        let wrapped = wrap_line(line, 5);
        assert_eq!(wrapped, vec!["hello", "world"]);
        assert_eq!(wrap_char_ranges(line, &wrapped), vec![(0, 5), (6, 5)]);
    }

    #[test]
    fn char_ranges_three_segments() {
        // Three single-char segments with gap chars between each ensures
        // prev_byte_end and prev_char_end are threaded correctly across all iterations.
        let line = "a b c";
        let wrapped = wrap_line(line, 2);
        assert_eq!(wrapped, vec!["a", "b", "c"]);
        assert_eq!(wrap_char_ranges(line, &wrapped), vec![(0, 1), (2, 1), (4, 1)]);
    }

    // --- build_normal_status_bar ---

    #[test]
    fn status_bar_clean_has_no_dirty_flag() {
        let app = make_app();
        let line = build_normal_status_bar(&app);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!text.contains("[*]"), "clean file must not show [*]");
    }

    #[test]
    fn status_bar_dirty_shows_flag() {
        let mut app = make_app();
        app.is_dirty = true;
        let line = build_normal_status_bar(&app);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[*]"), "dirty file must show [*]");
    }

    #[test]
    fn status_bar_includes_path() {
        let app = make_app();
        let line = build_normal_status_bar(&app);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("foo.md"), "status bar must include filename");
    }

    #[test]
    fn status_bar_no_panic() {
        let app = make_app();
        let _ = build_normal_status_bar(&app);
    }

    // --- build_timed_message_bar ---

    #[test]
    fn timed_bar_includes_filename() {
        let app = make_app();
        let line = build_timed_message_bar(&app, "Saved.");
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("foo.md"), "timed message bar must include the filename");
    }

    #[test]
    fn timed_bar_includes_message_text() {
        let app = make_app();
        let line = build_timed_message_bar(&app, "Saved.");
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("Saved."), "timed message bar must include the message text");
    }

    #[test]
    fn timed_bar_message_span_is_bold_accent() {
        let app = make_app();
        let theme = Theme::default_theme();
        let line = build_timed_message_bar(&app, "Saved.");
        assert!(
            line.spans.iter().any(|s| s.content.contains("Saved.")
                && s.style.fg == Some(theme.accent)
                && s.style.add_modifier.contains(Modifier::BOLD)),
            "message span must be bold accent"
        );
    }

    #[test]
    fn timed_bar_has_no_hints_bg() {
        // The hints zone (ui_bg) must not appear — it should dissolve into canvas_bg.
        let app = make_app();
        let theme = Theme::default_theme();
        let line = build_timed_message_bar(&app, "Saved.");
        assert!(
            !line.spans.iter().any(|s| s.style.bg == Some(theme.ui_bg)),
            "timed message bar must not use hints_bg on any span"
        );
    }

    #[test]
    fn timed_bar_dirty_shows_dirty_marker() {
        let mut app = make_app();
        app.is_dirty = true;
        let line = build_timed_message_bar(&app, "Saved.");
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[*]"), "timed message bar must show [*] when dirty");
    }

    #[test]
    fn timed_bar_clean_has_no_dirty_marker() {
        let app = make_app();
        let line = build_timed_message_bar(&app, "Saved.");
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!text.contains("[*]"), "timed message bar must not show [*] when clean");
    }

    #[test]
    fn heading_delimiter_bold_survives_split_into_spans() {
        // Verify that Modifier::BOLD on a StyledSpan passes through split_into_spans
        // to the resulting ratatui Span — confirming the renderer pipeline carries BOLD.
        let delim_span = StyledSpan {
            char_start: 0,
            char_end: 2,
            style: Style::default()
                .fg(ratatui::style::Color::Rgb(100, 150, 200))
                .add_modifier(Modifier::BOLD),
            ..Default::default()
        };
        let segments = split_into_spans("# Hello", &[delim_span], Style::default());
        let hash_span = segments
            .iter()
            .find(|s| s.content.starts_with('#'))
            .expect("span starting with # must exist");
        assert!(
            hash_span.style.add_modifier.contains(Modifier::BOLD),
            "BOLD modifier must survive split_into_spans for heading delimiter"
        );
    }

    #[test]
    fn heading_delimiter_bold_reaches_buffer() {
        // End-to-end: render a decorated H1 line into a ratatui Buffer and confirm
        // that the `#` cell carries Modifier::BOLD.  This catches any renderer path
        // that might silently drop the modifier before it hits the terminal.
        use ratatui::buffer::Buffer;
        use ratatui::widgets::Widget;

        let area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 40,
            height: 3,
        };
        let mut buf = Buffer::empty(area);

        let theme = crate::config::Theme::default_theme();
        let mut deco = DecorationMap::default();
        deco.insert(
            0,
            vec![
                StyledSpan {
                    char_start: 0,
                    char_end: 2, // "# "
                    style: Style::default()
                        .fg(theme.headings.h1)
                        .add_modifier(Modifier::BOLD),
                    ..Default::default()
                },
                StyledSpan {
                    char_start: 2,
                    char_end: 7, // "Hello"
                    style: Style::default()
                        .fg(theme.headings.h1)
                        .add_modifier(Modifier::BOLD),
                    ..Default::default()
                },
            ],
        );

        let view = MarkdownView {
            lines: &["# Hello".to_string()],
            decoration_map: &deco,
            scroll_top: 0,
            cursor: (0, 0),
            selection: None,
            theme: &theme,
            column_width: 40,
        };
        view.render(area, &mut buf);

        // The `#` char is at x = GUTTER (= 1), y = 0.
        let cell = buf.cell((GUTTER, 0)).expect("cell must exist");
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "H1 # character must carry Modifier::BOLD in the rendered ratatui buffer"
        );
    }

    #[test]
    fn heading_delimiter_fg_survives_split_into_spans() {
        use ratatui::style::Color;
        let green = Color::Rgb(166, 227, 161);
        let heading = Color::Rgb(203, 166, 247);
        let delim_span = StyledSpan {
            char_start: 0,
            char_end: 2,
            style: Style::default().fg(green),
            ..Default::default()
        };
        let content_span = StyledSpan {
            char_start: 2,
            char_end: 7,
            style: Style::default().fg(heading),
            ..Default::default()
        };
        let segments = split_into_spans("# Hello", &[delim_span, content_span], Style::default());
        let hash_span = segments
            .iter()
            .find(|s| s.content.starts_with('#'))
            .expect("must have a span starting with #");
        assert_eq!(
            hash_span.style.fg,
            Some(green),
            "# character must carry green fg through split_into_spans"
        );
    }

    // ── apply_selection_overlay ──────────────────────────────────────────────
    //
    // Tests operate directly on a ratatui Buffer (no terminal required).
    // GUTTER = 1, so for area.x=0 the content column starts at x=1.
    // char N on a line maps to buffer x = area.x + GUTTER + N = 1 + N.

    fn sel_view<'a>(
        lines: &'a [String],
        deco: &'a DecorationMap,
        theme: &'a Theme,
    ) -> MarkdownView<'a> {
        MarkdownView {
            lines,
            decoration_map: deco,
            scroll_top: 0,
            cursor: (0, 0),
            selection: None,
            theme,
            column_width: 10,
        }
    }

    #[test]
    fn selection_overlay_single_row_highlights_correct_cells() {
        // Selection: row 0, chars 1..3 (exclusive).
        // content_width = 10 - 2*GUTTER = 8. Lines fit without wrapping.
        // Expected highlighted buffer cells: x=2, x=3  (= GUTTER + char_offset).
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 3 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["abcde".into(), "fghij".into(), "klmno".into()];
        let view = sel_view(&lines, &deco, &theme);

        apply_selection_overlay(area, &mut buf, &view, ((0, 1), (0, 3)));

        let sel_bg = theme.selection_bg;
        assert_ne!(buf.cell((1, 0)).unwrap().bg, sel_bg, "char 0 before selection");
        assert_eq!(buf.cell((2, 0)).unwrap().bg, sel_bg, "char 1 selected");
        assert_eq!(buf.cell((3, 0)).unwrap().bg, sel_bg, "char 2 selected");
        assert_ne!(buf.cell((4, 0)).unwrap().bg, sel_bg, "char 3 after selection");
        assert_ne!(buf.cell((1, 1)).unwrap().bg, sel_bg, "row 1 not selected");
    }

    #[test]
    fn selection_overlay_multi_row_highlights_correct_cells() {
        // Selection: row 0 col 2 to row 1 col 2.
        // Row 0: chars 2..4 → x=3, x=4
        // Row 1: chars 0..2 → x=1, x=2
        // Row 2: no highlight
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 4 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["aaaa".into(), "bbbb".into(), "cccc".into()];
        let view = sel_view(&lines, &deco, &theme);

        apply_selection_overlay(area, &mut buf, &view, ((0, 2), (1, 2)));

        let sel_bg = theme.selection_bg;
        // Row 0: only chars 2 and 3 highlighted
        assert_ne!(buf.cell((1, 0)).unwrap().bg, sel_bg, "row 0 char 0");
        assert_ne!(buf.cell((2, 0)).unwrap().bg, sel_bg, "row 0 char 1");
        assert_eq!(buf.cell((3, 0)).unwrap().bg, sel_bg, "row 0 char 2");
        assert_eq!(buf.cell((4, 0)).unwrap().bg, sel_bg, "row 0 char 3");
        assert_ne!(buf.cell((5, 0)).unwrap().bg, sel_bg, "row 0 beyond line");
        // Row 1: chars 0 and 1 highlighted
        assert_eq!(buf.cell((1, 1)).unwrap().bg, sel_bg, "row 1 char 0");
        assert_eq!(buf.cell((2, 1)).unwrap().bg, sel_bg, "row 1 char 1");
        assert_ne!(buf.cell((3, 1)).unwrap().bg, sel_bg, "row 1 char 2 not selected");
        // Row 2: no highlight
        assert_ne!(buf.cell((1, 2)).unwrap().bg, sel_bg, "row 2 not selected");
    }

    #[test]
    fn selection_overlay_middle_row_fully_highlighted() {
        // 3-row selection: the middle row has neither a start-col nor end-col
        // constraint — it should be highlighted from char 0 to line end.
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 4 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["aaaa".into(), "bbbb".into(), "cccc".into()];
        let view = sel_view(&lines, &deco, &theme);

        // Select row 0 col 1 → row 2 col 1.  Row 1 is the unconstrained middle.
        apply_selection_overlay(area, &mut buf, &view, ((0, 1), (2, 1)));

        let sel_bg = theme.selection_bg;
        // Row 1 (middle): chars 0..4 all highlighted
        assert_eq!(buf.cell((1, 1)).unwrap().bg, sel_bg, "middle row char 0");
        assert_eq!(buf.cell((2, 1)).unwrap().bg, sel_bg, "middle row char 1");
        assert_eq!(buf.cell((3, 1)).unwrap().bg, sel_bg, "middle row char 2");
        assert_eq!(buf.cell((4, 1)).unwrap().bg, sel_bg, "middle row char 3");
        // Row 2: only char 0 highlighted (col_end = 1)
        assert_eq!(buf.cell((1, 2)).unwrap().bg, sel_bg, "last row char 0");
        assert_ne!(buf.cell((2, 2)).unwrap().bg, sel_bg, "last row char 1 excluded");
    }

    #[test]
    fn selection_overlay_row_after_sel_end_not_highlighted() {
        // log_row > sel_row_end → the loop must break and leave that row clean.
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 3 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["aaaa".into(), "bbbb".into(), "cccc".into()];
        let view = sel_view(&lines, &deco, &theme);

        // Selection is only on row 0.
        apply_selection_overlay(area, &mut buf, &view, ((0, 0), (0, 4)));

        let sel_bg = theme.selection_bg;
        // Row 1 (past sel_row_end) must not be touched.
        assert_ne!(buf.cell((1, 1)).unwrap().bg, sel_bg, "row after sel_row_end");
        assert_ne!(buf.cell((1, 2)).unwrap().bg, sel_bg, "two rows past sel_row_end");
    }

    #[test]
    fn selection_overlay_row_before_sel_start_not_highlighted() {
        // scroll_top=0, sel_row_start=1: row 0 must not be highlighted.
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 3 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["aaaa".into(), "bbbb".into(), "cccc".into()];
        let view = sel_view(&lines, &deco, &theme);

        // Selection starts on row 1.
        apply_selection_overlay(area, &mut buf, &view, ((1, 0), (1, 3)));

        let sel_bg = theme.selection_bg;
        // Row 0 (before sel_row_start) must not be touched.
        assert_ne!(buf.cell((1, 0)).unwrap().bg, sel_bg, "row before sel_row_start");
        // Row 1 IS selected.
        assert_eq!(buf.cell((1, 1)).unwrap().bg, sel_bg, "selected row char 0");
    }

    #[test]
    fn selection_overlay_empty_selection_no_highlight() {
        // row_sel_start == row_sel_end → nothing should be highlighted.
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 2 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["aaaa".into(), "bbbb".into()];
        let view = sel_view(&lines, &deco, &theme);

        // Zero-width selection (col_start == col_end).
        apply_selection_overlay(area, &mut buf, &view, ((0, 2), (0, 2)));

        let sel_bg = theme.selection_bg;
        for x in 0..10u16 {
            assert_ne!(buf.cell((x, 0)).unwrap().bg, sel_bg, "zero-width selection must not highlight x={x}");
        }
    }

    // Kills: renderer/mod.rs:346:71 replace * with + in apply_selection_overlay.
    // "abcdefgh" exactly fills content_width (cw = column_width - 2*GUTTER = 10-2 = 8).
    // x_end = min(GUTTER + 8, GUTTER + cw) = min(9, 9) = 9 → cell x=8 IS highlighted.
    // With *→+ (cw=7): min(9, 8) = 8 → cell x=8 NOT highlighted.
    #[test]
    fn selection_overlay_content_width_last_char_included() {
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 10, height: 2 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["abcdefgh".into(), "x".into()]; // 8 chars = full cw
        let view = sel_view(&lines, &deco, &theme);
        apply_selection_overlay(area, &mut buf, &view, ((0, 0), (0, 8)));
        let sel_bg = theme.selection_bg;
        assert_eq!(
            buf.cell((8, 0)).unwrap().bg, sel_bg,
            "char 7 at x=8 must be highlighted (last char within content_width=8)"
        );
        assert_ne!(
            buf.cell((9, 0)).unwrap().bg, sel_bg,
            "x=9 is past content_width and must not be highlighted"
        );
    }

    // Combined test for wrap+continuation_indent correctness in apply_selection_overlay.
    //
    // Setup: column_width=8, GUTTER=1, cw=6.
    // "abcdefghij" (10 chars, no spaces) wraps to "abcdef" (chars 0-5) + "ghij" (chars 6-9).
    // Decoration: continuation_indent=2 on line 0.
    // Selection: full line (0..10).
    //
    // Sub-row 0 (wrap_idx=0): ci must be 0 (original: wrap_idx > 0 is false).
    //   x_start=1, x_end=min(1+6, 1+6)=7. Highlights x=1..6.
    // Sub-row 1 (wrap_idx=1): ci=2 (original: wrap_idx > 0 is true).
    //   x_start=1+2+0=3, x_end=min(1+(10-6), 1+6)=min(5,7)=5. Highlights x=3..4.
    //
    // Kills:
    //   346:71  *→+ or *→/: cw=5 → sub-row 0 x_end clips at 6, cell (6,0) excluded.
    //   375:56  >→== / >→>=: ci applied to sub-row 0 → x_start=3, cell (1,0) excluded.
    //   375:56  >→<: no ci ever → sub-row 1 x_start=1, cell (1,1) highlighted (wrong).
    //   407:80  -→+: x_start=1+2+(6+6)=15, out of area → cell (3,1) not highlighted.
    //   408:65  -→+: x_end=min(1+(10+6),7)=7 → cell (5,1) highlighted (wrong).
    //   409:37  +→*: min clips 1 cell early → cell (6,0) not highlighted.
    //   409:46  +→*: same.
    #[test]
    fn selection_overlay_continuation_indent_only_on_wrapped_subrows() {
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 8, height: 3 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let mut deco = DecorationMap::default();
        deco.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 1,
                continuation_indent: 2,
                ..Default::default()
            }],
        );
        let lines: Vec<String> = vec!["abcdefghij".into(), "xx".into()];
        let view = MarkdownView {
            lines: &lines,
            decoration_map: &deco,
            scroll_top: 0,
            cursor: (0, 0),
            selection: None,
            theme: &theme,
            column_width: 8, // cw = 8 - 2*1 = 6
        };
        apply_selection_overlay(area, &mut buf, &view, ((0, 0), (0, 10)));
        let sel_bg = theme.selection_bg;

        // Sub-row 0, first cell: ci must be 0 for wrap_idx=0.
        // With >→== or >→>=: ci=2 → x_start=3, cell (1,0) not highlighted.
        assert_eq!(
            buf.cell((1, 0)).unwrap().bg, sel_bg,
            "sub-row 0 char 0 (x=1) must be highlighted — no continuation indent on first sub-row"
        );
        // Sub-row 0, last cell (char 5, x=GUTTER+5=6): must reach the full cw.
        // With 409 *→* mutations or 346 *→+: x_end clips 1 cell early → x=6 excluded.
        assert_eq!(
            buf.cell((6, 0)).unwrap().bg, sel_bg,
            "sub-row 0 char 5 (x=6) must be highlighted — last char fills content_width"
        );
        // Sub-row 1, before continuation indent: must not be highlighted.
        // With >→<: ci never applied → x_start=1, cell (1,1) highlighted (wrong).
        assert_ne!(
            buf.cell((1, 1)).unwrap().bg, sel_bg,
            "sub-row 1 x=1 must not be highlighted (before continuation_indent=2)"
        );
        // Sub-row 1, first highlighted cell: x = GUTTER + ci + (char_start - char_start) = 1+2+0 = 3.
        // With >→<: ci=0 → x_start=1, cell (1,1) highlighted instead.
        // With 407:80 -→+: x_start=1+2+(6+6)=15, out of area → cell (3,1) not highlighted.
        assert_eq!(
            buf.cell((3, 1)).unwrap().bg, sel_bg,
            "sub-row 1 first char (x=3 with ci=2) must be highlighted"
        );
        // Sub-row 1, one past the last char (x=5): must not be highlighted (char_len=4, x_end=5).
        // With 408:65 -→+: x_end=min(1+(10+6),7)=7 → x=5 highlighted (wrong).
        assert_ne!(
            buf.cell((5, 1)).unwrap().bg, sel_bg,
            "sub-row 1 x=5 (past char_len=4) must not be highlighted"
        );
    }

    // Kills: renderer/mod.rs:407:41 replace + with - in apply_selection_overlay (x_start formula).
    // Selection starts partway into the second sub-row (char 7, within sub-row chars 6-9).
    // x_start = GUTTER + ci + (row_sel_start - char_start) = 1 + 2 + (7-6) = 4.
    // With +→-: x_start = 1 + 2 - 1 = 2 → cell (2,1) highlighted (wrong).
    //
    // Note: the existing selection_overlay_multi_row_highlights_correct_cells test already
    // catches this via u16 underflow when row_sel_start=2, char_start=0, ci=0 → 1-2 panics
    // in debug mode.  This test provides an explicit non-overflow scenario.
    #[test]
    fn selection_overlay_subrow_partial_start_x_is_exact() {
        use ratatui::buffer::Buffer;
        let area = Rect { x: 0, y: 0, width: 8, height: 3 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let mut deco = DecorationMap::default();
        deco.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 1,
                continuation_indent: 2,
                ..Default::default()
            }],
        );
        let lines: Vec<String> = vec!["abcdefghij".into(), "xx".into()];
        let view = MarkdownView {
            lines: &lines,
            decoration_map: &deco,
            scroll_top: 0,
            cursor: (0, 0),
            selection: None,
            theme: &theme,
            column_width: 8,
        };
        // Selection: char 7 to end of line (within sub-row 1, chars 6-9).
        // sub-row 1: row_sel_start=max(7,6)=7. x_start = 1+2+(7-6) = 4. Highlights x=4.
        apply_selection_overlay(area, &mut buf, &view, ((0, 7), (0, 10)));
        let sel_bg = theme.selection_bg;

        // Sub-row 0 must be completely clean (selection col_start=7 > sub-row 0 char_end=6).
        assert_ne!(
            buf.cell((1, 0)).unwrap().bg, sel_bg,
            "sub-row 0 must not be highlighted when selection starts at char 7"
        );
        // x=2 must NOT be highlighted (selection offset is 1, not -1).
        // With +→-: x_start = 1+2-(7-6) = 2 → cell (2,1) highlighted (wrong).
        assert_ne!(
            buf.cell((2, 1)).unwrap().bg, sel_bg,
            "x=2 must not be highlighted — start offset must add (not subtract) the intra-chunk offset"
        );
        // First highlighted cell is x=4 = GUTTER + ci + (row_sel_start - char_start) = 1+2+1.
        assert_eq!(
            buf.cell((4, 1)).unwrap().bg, sel_bg,
            "x=4 must be highlighted — correct partial-start offset in wrapped sub-row"
        );
    }

    // Kills: renderer/mod.rs:413 GUTTER+GUTTER → GUTTER-GUTTER (content_width=10 instead of 8).
    // Line has 9 chars; selection covers all of them.  With correct cw=8 the x_end clamp
    // stops highlighting at x=9 (GUTTER + 8 = 9 is exclusive), so x=9 is NOT coloured.
    // With `+→-` (cw=10): x_end = min(GUTTER+9, GUTTER+10) = 10 → x=9 IS coloured. ✗
    // With `+→*` (cw=GUTTER*GUTTER=1): x_end = min(10, 2) = 2 → x=8 NOT coloured. ✗
    #[test]
    fn selection_overlay_content_width_clamps_selection_at_gutter_boundary() {
        use ratatui::buffer::Buffer;
        // column_width=10, cw=8.  Line has 9 chars (one past cw).
        let area = Rect { x: 0, y: 0, width: 11, height: 2 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["abcdefghi".into(), "x".into()]; // 9 chars
        let view = MarkdownView {
            lines: &lines,
            decoration_map: &deco,
            scroll_top: 0,
            cursor: (0, 0),
            selection: None,
            theme: &theme,
            column_width: 10, // cw = 10 - (GUTTER+GUTTER) = 8
        };
        apply_selection_overlay(area, &mut buf, &view, ((0, 0), (0, 9)));
        let sel_bg = theme.selection_bg;
        // char 7 (x=8) is within content_width=8, must be highlighted.
        assert_eq!(
            buf.cell((8, 0)).unwrap().bg, sel_bg,
            "char 7 (x=8) is within content_width=8 and must be highlighted"
        );
        // x=9 = GUTTER + content_width is the first column PAST content; must not be highlighted.
        // `+→-` widens cw to 10: x_end becomes 10, x=9 gets highlighted → test fails. ✓
        assert_ne!(
            buf.cell((9, 0)).unwrap().bg, sel_bg,
            "x=9 is past content_width=8 and must not be highlighted"
        );
    }

    // Kills: renderer/mod.rs:425 `&&→||` and `log_row < total → <=`.
    //
    // The `log_row > sel_row_end { break; }` guard fires *before* the
    // out-of-bounds access — so to reach it we must set sel_row_end beyond
    // the last valid line index (= lines.len()).  That way the break never
    // fires and the mutated loop tries view.lines[total] → index panic.
    //
    // With correct `&&`: `while 2 < 5 && 2 < 2` = false → clean exit. ✓
    // With `&&→||`:      `2 < 5 || 2 < 2` = true → view.lines[2] → panic. ✓
    // With `<→<=` (log): `2 < 5 && 2 <= 2` = true → view.lines[2] → panic. ✓
    #[test]
    fn selection_overlay_loop_stops_at_document_end() {
        use ratatui::buffer::Buffer;
        // 2-line document in a 5-row viewport — total(2) < visible(5).
        let area = Rect { x: 0, y: 0, width: 10, height: 5 };
        let mut buf = Buffer::empty(area);
        let theme = Theme::default_theme();
        let deco = DecorationMap::default();
        let lines: Vec<String> = vec!["abcd".into(), "efgh".into()];
        let view = sel_view(&lines, &deco, &theme);
        // sel_row_end = lines.len() (= 2, one past the last valid index).
        // This prevents the `log_row > sel_row_end` early-break from saving
        // the mutated code when log_row reaches total=2 and tries to access
        // view.lines[2] out of bounds.
        apply_selection_overlay(area, &mut buf, &view, ((0, 0), (lines.len(), 0)));
        let sel_bg = theme.selection_bg;
        // Both lines selected — spot-check a cell on each.
        assert_eq!(buf.cell((1, 0)).unwrap().bg, sel_bg, "row 0 selected");
        assert_eq!(buf.cell((1, 1)).unwrap().bg, sel_bg, "row 1 selected");
        // Rows 2-4 are past the document and must not be touched.
        for y in 2..5u16 {
            assert_ne!(
                buf.cell((1, y)).unwrap().bg, sel_bg,
                "row {y} is past document end and must not be highlighted"
            );
        }
    }
}
