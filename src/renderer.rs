use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::app::App;
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
// Step 5.2 — Status bar
// ---------------------------------------------------------------------------

const POWERLINE_RIGHT: char = '\u{e0b0}';

/// Render the bottom status bar into `area`.
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
}
