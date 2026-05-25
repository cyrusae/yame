use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::status::StatusMode;

use super::{format_thousands, shorten_path};

const POWERLINE_RIGHT: char = '\u{e0b0}';

/// Render the bottom status bar into `area`.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let canvas_bg = theme.bg;
    let hints_bg = theme.ui_bg;
    let warning_fg = theme.warning;

    let content: Line = match &app.status.mode {
        StatusMode::ExitPrompt => Line::from(vec![Span::styled(
            " Save modified buffer? [Y]es  [N]o  [C]ancel ",
            Style::default()
                .fg(warning_fg)
                .bg(hints_bg)
                .add_modifier(Modifier::BOLD),
        )]),

        StatusMode::TimedMessage { text, .. } | StatusMode::DismissibleMessage(text) => {
            let msg = format!(" {text} ");
            let pad = area
                .width
                .saturating_sub(msg.len() as u16)
                .saturating_div(2);
            let padded = format!("{:pad$}{msg}", "", pad = pad as usize);
            Line::from(vec![Span::styled(
                padded,
                Style::default().fg(theme.text).bg(hints_bg),
            )])
        }

        StatusMode::Normal => build_normal_status_bar(app),
    };

    let para = Paragraph::new(content).style(Style::default().bg(canvas_bg));
    f.render_widget(para, area);
}

pub(super) fn build_normal_status_bar(app: &App) -> Line<'static> {
    let theme = &app.theme;
    let canvas_bg = theme.bg;
    let hints_bg = theme.ui_bg;
    let muted_fg = theme.muted;
    let sep = POWERLINE_RIGHT.to_string();

    let (pill_bg, pill_fg) = if app.is_dirty {
        (theme.accent, theme.bg)
    } else {
        (theme.text, theme.bg)
    };
    let dirty_marker = if app.is_dirty { " [*]" } else { "" };
    let path_str = shorten_path(&app.file_path, 3);
    let pill1_text = format!(" {path_str}{dirty_marker} ");

    let pill1 = Span::styled(pill1_text, Style::default().fg(pill_fg).bg(pill_bg));
    let cap1 = Span::styled(sep.clone(), Style::default().fg(pill_bg).bg(hints_bg));

    let hints = Span::styled(
        " ^S Save  ^X Exit  ^Z Undo  ^Y Redo  ^R Reload ",
        Style::default().fg(muted_fg).bg(hints_bg),
    );
    let cap2 = Span::styled(sep, Style::default().fg(hints_bg).bg(canvas_bg));

    Line::from(vec![pill1, cap1, hints, cap2])
}

/// Render the second-to-last row: cursor position and word count.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_info_line(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

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
    let text_width = (text.chars().count() as u16).min(area.width);
    let text_area = Rect {
        width: text_width,
        ..area
    };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(theme.muted).bg(theme.bg)),
        text_area,
    );
}
