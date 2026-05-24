use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::app::App;
use crate::decoration::{DecorationMap, StyledSpan};
use crate::status::StatusMode;

// ---------------------------------------------------------------------------
// Pure helpers (tested below)
// ---------------------------------------------------------------------------

/// Shorten a file path to at most `max_components` trailing components.
/// e.g. "/home/user/docs/notes/foo.md" → "notes/foo.md" with max_components=2
pub fn shorten_path(path: &std::path::Path, max_components: usize) -> String {
    let components: Vec<_> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let start = components.len().saturating_sub(max_components);
    components[start..].join("/")
}

/// Format a usize with thousands separators.
/// e.g. 1204 → "1,204", 999 → "999"
pub fn format_thousands(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + len / 3);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

// ---------------------------------------------------------------------------
// Step 7.3 — Span boundary splitting
// ---------------------------------------------------------------------------

/// Split a raw line string into ratatui `Span`s at decoration span boundaries.
///
/// `spans` need not be sorted or non-overlapping; this function sorts by
/// `char_start` and clips any later span that overlaps a prior one.
/// All indices are char indices (not byte indices).
pub fn split_into_spans(
    line: &str,
    spans: &[StyledSpan],
    default_style: Style,
) -> Vec<Span<'static>> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let char_count = chars.len();

    // Fast path — no decoration spans
    if spans.is_empty() {
        return vec![Span::styled(line.to_owned(), default_style)];
    }

    // Sort by char_start (stable — preserves paint order within same position)
    let mut sorted = spans.to_vec();
    sorted.sort_by_key(|s| s.char_start);

    let mut result: Vec<Span<'static>> = Vec::with_capacity(sorted.len() * 2 + 1);
    let mut char_pos = 0usize; // current position in char terms

    for span in &sorted {
        // Clamp and clip for safety
        let s_start = span.char_start.min(char_count);
        let s_end = span.char_end.min(char_count);
        // Clip start forward to handle overlapping spans
        let s_start = s_start.max(char_pos);

        if s_start >= s_end {
            continue; // zero-width or fully overlapped
        }

        // Unstyled gap before this span
        if char_pos < s_start {
            let byte_start = chars[char_pos].0;
            let byte_end = chars[s_start].0;
            result.push(Span::styled(
                line[byte_start..byte_end].to_owned(),
                default_style,
            ));
        }

        // Styled span content
        let byte_start = chars[s_start].0;
        let byte_end = if s_end < char_count {
            chars[s_end].0
        } else {
            line.len()
        };
        result.push(Span::styled(
            line[byte_start..byte_end].to_owned(),
            span.style,
        ));

        char_pos = s_end;
    }

    // Trailing unstyled tail
    if char_pos < char_count {
        let byte_start = chars[char_pos].0;
        result.push(Span::styled(line[byte_start..].to_owned(), default_style));
    }

    // Always return at least one span (handles empty lines)
    if result.is_empty() {
        result.push(Span::styled(String::new(), default_style));
    }

    result
}

// ---------------------------------------------------------------------------
// Step 7.5 — Soft-wrap
// ---------------------------------------------------------------------------

/// Soft-wrap a string into visual rows of at most `width` chars each.
///
/// Breaks at the last space before `width`; falls back to a hard break at
/// exactly `width` chars when no space exists. Returns byte-slices of `s`
/// so no allocation is needed.
///
/// Width is measured in Unicode scalar values (chars), not display columns.
/// Wide characters (e.g. CJK) are a v1.5 concern; use `unicode-width` then.
pub fn wrap_line(s: &str, width: usize) -> Vec<&str> {
    if s.is_empty() || width == 0 {
        return vec![s];
    }

    let char_indices: Vec<(usize, char)> = s.char_indices().collect();
    let total_chars = char_indices.len();

    // Fits on a single row without wrapping
    if total_chars <= width {
        return vec![s];
    }

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

        let chunk_end = char_start + width; // exclusive upper bound

        // Find the last space in [char_start .. chunk_end)
        let last_space_rel = char_indices[char_start..chunk_end]
            .iter()
            .rposition(|&(_, c)| c == ' ');

        let (break_char, next_char) = match last_space_rel {
            Some(rel) => {
                let abs = char_start + rel;
                (abs, abs + 1) // break before space, skip the space itself
            }
            None => (chunk_end, chunk_end), // hard break — no space found
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

/// Blank columns on each side of the text content within the editor column.
/// Keeps text from running flush against the background boundary.
const GUTTER: u16 = 1;

// ---------------------------------------------------------------------------
// Step 7.1 — MarkdownView widget
// ---------------------------------------------------------------------------

/// The main editor rendering widget.
///
/// Renders visible logical lines from `scroll_top`, applying soft-wrap,
/// decoration spans, cursor highlighting, and selection overlay.
pub struct MarkdownView<'a> {
    pub lines: &'a [String],
    pub decoration_map: &'a DecorationMap,
    pub scroll_top: usize,
    /// Logical (row, col) cursor position from `textarea.cursor()`.
    pub cursor: (usize, usize),
    /// Normalised selection range from `textarea.selection_range()`.
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub theme: &'a crate::config::Theme,
    /// Whether the terminal supports italic — stored for future use (e.g. tooltips).
    pub italic_support: bool,
    pub column_width: u16,
}

impl Widget for MarkdownView<'_> {
    #[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Content area is inset by GUTTER on each side within the column.
        let content_width = (self.column_width as usize).saturating_sub(2 * GUTTER as usize);
        let visible = area.height as usize;
        let bg = self.theme.bg;
        let default_style = Style::default().fg(self.theme.text).bg(bg);

        // 1. Flood-fill the entire area with the background colour.
        for row in 0..area.height {
            for col in 0..area.width {
                buf[(area.x + col, area.y + row)].set_bg(bg);
            }
        }

        let (cursor_log_row, cursor_log_col) = self.cursor;
        let mut cursor_buf_pos: Option<(u16, u16)> = None;

        // 2. Render each visible logical line.
        let mut visual_row: usize = 0;
        let total = self.lines.len();
        let mut log_row = self.scroll_top;

        while visual_row < visible && log_row < total {
            let line = &self.lines[log_row];
            let wrapped = wrap_line(line, content_width.max(1));
            let line_decs = self.decoration_map.get(&log_row);

            for (wrap_idx, &row_str) in wrapped.iter().enumerate() {
                if visual_row >= visible {
                    break;
                }

                // --- Compute this visual row's char range within the logical line ---
                let byte_off = (row_str.as_ptr() as usize).wrapping_sub(line.as_ptr() as usize);
                let char_start = line[..byte_off].chars().count();
                let char_len = row_str.chars().count();
                let char_end = char_start + char_len;

                // --- Cursor tracking ---
                if log_row == cursor_log_row {
                    let is_last_wrap = wrap_idx + 1 == wrapped.len();
                    let in_range =
                        cursor_log_col >= char_start && (cursor_log_col < char_end || is_last_wrap);
                    if in_range {
                        let col_in_row = (cursor_log_col.saturating_sub(char_start))
                            .min(content_width.saturating_sub(1));
                        cursor_buf_pos = Some((
                            area.x + GUTTER + col_in_row as u16,
                            area.y + visual_row as u16,
                        ));
                    }
                }

                // --- Adjust decoration spans to this visual row's char range ---
                let row_spans: Vec<StyledSpan> = line_decs
                    .map(|decs| {
                        decs.iter()
                            .filter(|s| s.char_end > char_start && s.char_start < char_end)
                            .map(|s| StyledSpan {
                                char_start: s.char_start.saturating_sub(char_start),
                                char_end: s.char_end.saturating_sub(char_start).min(char_len),
                                style: s.style,
                                is_blockquote: s.is_blockquote,
                                full_line_bg: s.full_line_bg,
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // --- Full-line background (headings, fenced blocks) ---
                let line_bg = row_spans.iter().find_map(|s| s.full_line_bg).unwrap_or(bg);

                let y = area.y + visual_row as u16;

                // Flood-fill background across the full column width first.
                for col in 0..self.column_width {
                    buf[(area.x + col, y)].set_bg(line_bg);
                }

                // --- Write character cells ---
                let row_default = default_style.bg(line_bg);
                let segments = split_into_spans(row_str, &row_spans, row_default);
                let mut x = area.x + GUTTER; // start after left gutter
                for span in &segments {
                    for ch in span.content.chars() {
                        if (x.saturating_sub(area.x + GUTTER)) as usize >= content_width {
                            break;
                        }
                        buf[(x, y)].set_char(ch).set_style(span.style);
                        x += 1;
                    }
                }

                visual_row += 1;
            }

            log_row += 1;
        }

        // 3. Selection overlay — applied after content, before cursor.
        if let Some(selection) = self.selection {
            apply_selection_overlay(area, buf, &self, selection);
        }

        // 4. Cursor cell — always on top.
        if let Some((cx, cy)) = cursor_buf_pos {
            buf[(cx, cy)]
                .set_fg(self.theme.bg)
                .set_bg(self.theme.accent);
        }
    }
}

/// Apply the selection highlight over the already-rendered buffer cells.
///
/// Iterates logical lines in the selection range, maps each visual row's
/// char range to buffer coordinates, and overwrites fg+bg for covered cells.
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
        // Stop scanning once we're past the selection end.
        if log_row > sel_row_end {
            break;
        }

        let line = &view.lines[log_row];
        let wrapped = wrap_line(line, content_width.max(1));

        for row_str in &wrapped {
            if visual_row >= visible {
                break;
            }

            let byte_off = (row_str.as_ptr() as usize).wrapping_sub(line.as_ptr() as usize);
            let char_start = line[..byte_off].chars().count();
            let char_len = row_str.chars().count();
            let char_end = char_start + char_len;

            if log_row >= sel_row_start && log_row <= sel_row_end {
                // Selection range on this logical line (in logical-line char units)
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

                // Intersect with this visual row's char range
                let row_sel_start = line_sel_start.max(char_start);
                let row_sel_end = line_sel_end.min(char_end);

                if row_sel_start < row_sel_end {
                    let y = area.y + visual_row as u16;
                    let x_start = area.x + GUTTER + (row_sel_start - char_start) as u16;
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
// Step 5.2 — Status bar
// ---------------------------------------------------------------------------

const POWERLINE_RIGHT: char = '\u{e0b0}';

/// Render the bottom status bar into `area`.
#[mutants::skip] // Writes into ratatui Buffer — mutations have no observable return value to assert on.
pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let bar_bg = theme.ui_bar;
    let text_fg = theme.text;
    let warning_fg = theme.warning;

    let content: Line = match &app.status.mode {
        StatusMode::ExitPrompt => Line::from(vec![Span::styled(
            " Save modified buffer? [Y]es  [N]o  [C]ancel ",
            Style::default()
                .fg(warning_fg)
                .bg(bar_bg)
                .add_modifier(Modifier::BOLD),
        )]),

        StatusMode::TimedMessage { text, .. } | StatusMode::DismissibleMessage(text) => {
            // Center the message text in the bar
            let msg = format!(" {text} ");
            let pad = area
                .width
                .saturating_sub(msg.len() as u16)
                .saturating_div(2);
            let padded = format!("{:pad$}{msg}", "", pad = pad as usize);
            Line::from(vec![Span::styled(
                padded,
                Style::default().fg(text_fg).bg(bar_bg),
            )])
        }

        StatusMode::Normal => build_normal_status_bar(app, area.width),
    };

    let para = Paragraph::new(content).style(Style::default().bg(bar_bg));
    f.render_widget(para, area);
}

fn build_normal_status_bar(app: &App, width: u16) -> Line<'static> {
    let theme = &app.theme;
    let bar_bg = theme.ui_bar;
    let text_fg = theme.text;
    let accent_fg = theme.accent;
    let muted_fg = theme.muted;

    // Left: shortened path + dirty flag
    let path_str = shorten_path(&app.file_path, 3);
    let dirty = if app.is_dirty { " [*]" } else { "" };
    let left = format!(" {path_str}{dirty} ");

    // Center: keybinding hints
    let center = " ^S Save  ^X Exit  ^Z Undo  ^Y Redo ";

    // Powerline separator
    let sep = POWERLINE_RIGHT.to_string();

    // Build left segment spans
    let left_bg = theme.ui_bg;
    let mut spans: Vec<Span<'static>> = vec![
        Span::styled(left, Style::default().fg(text_fg).bg(left_bg)),
        Span::styled(sep.clone(), Style::default().fg(left_bg).bg(bar_bg)),
    ];

    // Pad center — compute available space
    let used = spans.iter().map(|s| s.content.len()).sum::<usize>() + center.len() + sep.len();
    let right_pad = (width as usize).saturating_sub(used);
    let padded_center = format!("{center}{:right_pad$}", "", right_pad = right_pad);

    spans.push(Span::styled(
        padded_center,
        Style::default().fg(muted_fg).bg(bar_bg),
    ));
    spans.push(Span::styled(sep, Style::default().fg(accent_fg).bg(bar_bg)));

    Line::from(spans)
}

// ---------------------------------------------------------------------------
// Step 5.3 — Info line
// ---------------------------------------------------------------------------

/// Render the second-to-last row: cursor position and word count.
/// Only the text itself gets a background rectangle; the rest of the row
/// shows the editor background so the bar doesn't span the full terminal.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_info_line(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    // Clear full row with editor background first.
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(theme.bg)),
        area,
    );

    let (row, col) = app.textarea.cursor();
    let text = format!(
        " Ln {}, Col {} · {} words ",
        format_thousands(row + 1),
        format_thousands(col + 1),
        format_thousands(app.word_count),
    );
    // Render only as wide as the text content (char count ≈ display width for ASCII + ·).
    let text_width = (text.chars().count() as u16).min(area.width);
    let text_area = Rect {
        width: text_width,
        ..area
    };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(theme.muted).bg(theme.ui_bg)),
        text_area,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use tui_textarea::TextArea;

    use crate::config::Theme;
    use crate::decoration::DecorationMap;
    use crate::status::StatusLine;

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("notes/foo.md"),
            is_dirty: false,
            saved_content: None,
            theme: Theme::default_theme(),
            italic_support: false,
            last_keystroke: None,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
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
        assert_eq!(result[0].content, "01"); // unstyled prefix
        assert_eq!(result[1].content, "234"); // styled span
        assert_eq!(result[2].content, "56789"); // unstyled suffix
    }

    #[test]
    fn span_split_multibyte_safe() {
        // "café" — chars: c(0) a(1) f(2) é(3); 'é' is 2 bytes
        // span covers char index 1 ("a")
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
        // Two overlapping spans: first covers 0..5, second covers 3..8
        // Second should be clipped to start at 5
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
        // s1: 0..5 = "01234"
        // s2 clipped: 5..8 = "567"
        // trailing: 8..10 = "89"
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
        // 4 kanji chars, width=4 — fits in one visual row (char-count width)
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
        let line = build_normal_status_bar(&app, 80);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(!text.contains("[*]"), "clean file must not show [*]");
    }

    #[test]
    fn status_bar_dirty_shows_flag() {
        let mut app = make_app();
        app.is_dirty = true;
        let line = build_normal_status_bar(&app, 80);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("[*]"), "dirty file must show [*]");
    }

    #[test]
    fn status_bar_includes_path() {
        let app = make_app();
        let line = build_normal_status_bar(&app, 80);
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.contains("foo.md"), "status bar must include filename");
    }

    #[test]
    fn status_bar_zero_width_no_panic() {
        let app = make_app();
        // Must not panic even at zero width (saturating arithmetic).
        let _ = build_normal_status_bar(&app, 0);
    }
}
