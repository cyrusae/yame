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

/// Soft-wrap a string into visual rows of at most `width` chars each.
///
/// Breaks at the last space before `width`; falls back to a hard break at
/// exactly `width` chars when no space exists. Returns byte-slices of `s`
/// so no allocation is needed.
pub fn wrap_line(s: &str, width: usize) -> Vec<&str> {
    if s.is_empty() || width == 0 {
        return vec![s];
    }

    let total_chars = s.chars().count();
    if total_chars <= width {
        return vec![s];
    }

    let char_indices: Vec<(usize, char)> = s.char_indices().collect();

    let mut result: Vec<&str> = Vec::new();
    let mut char_start = 0usize;

    loop {
        let remaining = total_chars - char_start;
        if remaining == 0 {
            break;
        }

        if remaining <= width {
            let byte_start = char_indices[char_start].0;
            result.push(&s[byte_start..]);
            break;
        }

        let chunk_end = char_start + width;

        let last_space_rel = char_indices[char_start..chunk_end]
            .iter()
            .rposition(|&(_, c)| c == ' ');

        let (break_char, next_char) = match last_space_rel {
            Some(rel) => {
                let abs = char_start + rel;
                (abs, abs + 1)
            }
            None => (chunk_end, chunk_end),
        };

        let byte_start = char_indices[char_start].0;
        let byte_end = char_indices[break_char].0;
        result.push(&s[byte_start..byte_end]);

        char_start = next_char;
    }

    if result.is_empty() {
        result.push(s);
    }

    result
}

/// Compute the `(char_start, char_len)` pair for each element of a
/// [`wrap_line`] output slice, measured in char units within `line`.
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
            let wrapped = wrap_line(line, content_width.max(1));
            let line_decs = self.decoration_map.get(&log_row);

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
                    let row_default = default_style.bg(line_bg);
                    let segments = split_into_spans(row_str, &row_spans, row_default);
                    let mut x = area.x + GUTTER + continuation_indent;
                    for span in &segments {
                        for ch in span.content.chars() {
                            if (x.saturating_sub(area.x + GUTTER)) as usize >= content_width {
                                break;
                            }
                            buf[(x, y)].set_char(ch).set_style(span.style);
                            x += 1;
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
    let content_width = (view.column_width as usize).saturating_sub(2 * GUTTER as usize);
    let visible = area.height as usize;
    let total = view.lines.len();
    let sel_fg = view.theme.selection_fg;
    let sel_bg = view.theme.selection_bg;

    let mut visual_row: usize = 0;
    let mut log_row = view.scroll_top;

    while visual_row < visible && log_row < total {
        if log_row > sel_row_end {
            break;
        }

        let line = &view.lines[log_row];
        let wrapped = wrap_line(line, content_width.max(1));

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

    use super::status::build_normal_status_bar;
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
    fn wrap_multibyte_respects_char_count() {
        let result = wrap_line("日本語テ", 4);
        assert_eq!(result.len(), 1);
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
}
