use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::status::StatusMode;

use super::format_thousands;

/// Universal box-drawing separator (no special font required).
const SEP_UNIVERSAL: char = '│';
/// Powerline/Nerd Font filled right-arrow. Requires a patched font.
const SEP_POWERLINE: char = '\u{e0b0}';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the pill1 (filename + dirty marker) span and return its background
/// colour.  Shared between the normal bar and the timed-message bar so both
/// always show the same pill while only the hints zone changes.
fn pill1_parts(app: &App) -> (Span<'static>, Color) {
    let theme = &app.theme;
    let (pill_bg, pill_fg): (Color, Color) = if app.is_dirty {
        (theme.accent, theme.bg)
    } else {
        (theme.text, theme.bg)
    };
    let dirty_marker = if app.is_dirty { " [*]" } else { "" };
    let path_str = &app.shortened_path;
    let text = format!(" {path_str}{dirty_marker} ");
    (
        Span::styled(text, Style::default().fg(pill_fg).bg(pill_bg)),
        pill_bg,
    )
}

// ---------------------------------------------------------------------------
// Public render entry points
// ---------------------------------------------------------------------------

/// Render the bottom status bar into `area`.
#[cfg_attr(feature = "mutants-skip", mutants::skip)] // Writes into ratatui Buffer — void, not testable via return value.
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

        StatusMode::TimedMessage { text, .. } => build_timed_message_bar(app, text),

        StatusMode::DismissibleMessage(text) => {
            let msg = format!(" {text} ");
            let msg_display_width = msg.chars().count() as u16;
            let pad = area
                .width
                .saturating_sub(msg_display_width)
                .saturating_div(2);
            let padded = format!("{:pad$}{msg}{:pad$}", "", "", pad = pad as usize);
            Line::from(vec![Span::styled(
                padded,
                Style::default().fg(warning_fg).bg(hints_bg),
            )])
        }

        StatusMode::Normal => build_normal_status_bar(app),

        StatusMode::GoToLine { input } => build_goto_line_bar(app, input),
    };

    let para = Paragraph::new(content).style(Style::default().bg(canvas_bg));
    f.render_widget(para, area);
}

// ---------------------------------------------------------------------------
// Bar builders (pub(super) for unit-testing)
// ---------------------------------------------------------------------------

/// Build the status bar for a timed flash message (e.g. "Saved.").
///
/// Pill1 (filename / dirty state) stays visible on the left.  The hints zone
/// is replaced by the message text drawn on `canvas_bg` so it visually
/// dissolves into the background — only the accent-coloured text remains.
pub(super) fn build_timed_message_bar(app: &App, text: &str) -> Line<'static> {
    let theme = &app.theme;
    let canvas_bg = theme.bg;
    let sep = if app.powerline_glyphs {
        SEP_POWERLINE
    } else {
        SEP_UNIVERSAL
    }
    .to_string();

    let (pill1, pill_bg) = pill1_parts(app);
    // Separator transitions pill_bg → canvas_bg so the hints zone disappears.
    let cap1 = Span::styled(sep, Style::default().fg(pill_bg).bg(canvas_bg));
    let msg = Span::styled(
        format!(" {text} "),
        Style::default()
            .fg(theme.accent)
            .bg(canvas_bg)
            .add_modifier(Modifier::BOLD),
    );

    Line::from(vec![pill1, cap1, msg])
}

pub(super) fn build_normal_status_bar(app: &App) -> Line<'static> {
    let theme = &app.theme;
    let canvas_bg = theme.bg;
    let hints_bg = theme.ui_bg;
    let muted_fg = theme.muted;
    let sep = if app.powerline_glyphs {
        SEP_POWERLINE
    } else {
        SEP_UNIVERSAL
    }
    .to_string();

    let (pill1, pill_bg) = pill1_parts(app);
    let cap1 = Span::styled(sep.clone(), Style::default().fg(pill_bg).bg(hints_bg));

    let hints = Span::styled(
        " ^S Save  ^X Exit  ^Z Undo  ^Y Redo  ^F Search  ⌥T FmtTbl ",
        Style::default().fg(muted_fg).bg(hints_bg),
    );
    let cap2 = Span::styled(sep, Style::default().fg(hints_bg).bg(canvas_bg));

    Line::from(vec![pill1, cap1, hints, cap2])
}

/// Build the status bar for the go-to-line prompt.
///
/// Shows `" Go to line: {input}_ "` centred on `hints_bg` so the prompt is
/// visually distinct from the normal hints bar but not as alarming as the exit
/// prompt.  The trailing `_` acts as a simple text cursor indicator.
pub(super) fn build_goto_line_bar(app: &App, input: &str) -> Line<'static> {
    let theme = &app.theme;
    let hints_bg = theme.ui_bg;
    let accent_fg = theme.accent;

    let content = format!(" Go to line: {input}_ ");
    Line::from(vec![Span::styled(
        content,
        Style::default()
            .fg(accent_fg)
            .bg(hints_bg)
            .add_modifier(Modifier::BOLD),
    )])
}

/// Nerd Font typewriter icon (nf-md-typewriter, U+F04C3).  Requires a patched font.
const TYPEWRITER_GLYPH: &str = "󰓃";
/// ASCII fallback for the typewriter mode indicator.
const TYPEWRITER_ASCII: &str = "[T]";
/// Nerd Font eye icon (nf-md-eye, U+F06D2) for focus mode.  Requires a patched font.
const FOCUS_GLYPH: &str = "󰛐";
/// ASCII fallback for the focus mode indicator.
const FOCUS_ASCII: &str = "[F]";

/// Render the second-to-last row: cursor position, word count, and mode indicators.
#[cfg_attr(feature = "mutants-skip", mutants::skip)] // Writes into ratatui Buffer — void, not testable via return value.
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

    // Mode indicators — right-aligned, accent coloured.
    // Each active mode contributes a token; they are concatenated and rendered
    // as a single right-aligned widget so they stack naturally.
    let mut indicators = String::new();
    if app.focus_mode {
        let icon = if app.powerline_glyphs {
            FOCUS_GLYPH
        } else {
            FOCUS_ASCII
        };
        indicators.push_str(&format!(" {icon} "));
    }
    if app.typewriter_mode {
        let icon = if app.powerline_glyphs {
            TYPEWRITER_GLYPH
        } else {
            TYPEWRITER_ASCII
        };
        indicators.push_str(&format!(" {icon} "));
    }
    if !indicators.is_empty() {
        let iw = indicators.chars().count() as u16;
        if iw <= area.width {
            let indicator_area = Rect {
                x: area.x + area.width - iw,
                width: iw,
                ..area
            };
            f.render_widget(
                Paragraph::new(indicators)
                    .style(Style::default().fg(theme.accent).bg(theme.bg)),
                indicator_area,
            );
        }
    }
}
