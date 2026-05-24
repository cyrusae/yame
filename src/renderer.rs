use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget},
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
pub fn split_into_spans(line: &str, spans: &[StyledSpan], default_style: Style) -> Vec<Span<'static>> {
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
            result.push(Span::styled(line[byte_start..byte_end].to_owned(), default_style));
        }

        // Styled span content
        let byte_start = chars[s_start].0;
        let byte_end = if s_end < char_count { chars[s_end].0 } else { line.len() };
        result.push(Span::styled(line[byte_start..byte_end].to_owned(), span.style));

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
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_info_line(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let (row, col) = app.textarea.cursor();
    // Display 1-indexed
    let text = format!(
        " Ln {}, Col {} · {} words",
        format_thousands(row + 1),
        format_thousands(col + 1),
        format_thousands(app.word_count),
    );
    let para = Paragraph::new(text)
        .style(Style::default().fg(theme.muted).bg(theme.ui_bg))
        .block(Block::default());
    f.render_widget(para, area);
}

// ---------------------------------------------------------------------------
// Step 5.4 — Scrollbar
// ---------------------------------------------------------------------------

/// Render the vertical scrollbar in `area`.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_scrollbar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let total_lines = app.textarea.lines().len();

    let mut state = ScrollbarState::new(total_lines).position(app.scroll_top);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_style(Style::default().fg(theme.ui_bar))
        .thumb_style(Style::default().fg(theme.accent))
        .begin_symbol(None)
        .end_symbol(None);

    f.render_stateful_widget(scrollbar, area, &mut state);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        assert_eq!(result[0].content, "01");    // unstyled prefix
        assert_eq!(result[1].content, "234");   // styled span
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
        let s1 = StyledSpan { char_start: 0, char_end: 5, style: bold_style(), ..Default::default() };
        let s2 = StyledSpan { char_start: 3, char_end: 8, ..Default::default() };
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
}
