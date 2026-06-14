use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
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
    let (pill_bg, pill_fg): (Color, Color) = if app.read_only {
        (theme.muted, theme.bg)
    } else if app.is_dirty {
        (theme.accent, theme.bg)
    } else {
        (theme.text, theme.bg)
    };
    let marker = if app.read_only {
        " [RO]"
    } else if app.is_dirty {
        " [*]"
    } else {
        ""
    };
    let path_str = &app.shortened_path;
    let text = format!(" {path_str}{marker} ");
    (
        Span::styled(text, Style::default().fg(pill_fg).bg(pill_bg)),
        pill_bg,
    )
}

// ---------------------------------------------------------------------------
// Public render entry points
// ---------------------------------------------------------------------------

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

        StatusMode::SaveAs { input, .. } => build_save_as_bar(app, input),

        StatusMode::SwitchFilePrompt => Line::from(vec![Span::styled(
            " Unsaved changes will be lost — switch file? [Y]es  [N]o ",
            Style::default()
                .fg(warning_fg)
                .bg(hints_bg)
                .add_modifier(Modifier::BOLD),
        )]),
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

    let hints_text = if app.read_only {
        " ^E Unlock  ^Q Quit  F1 Keys "
    } else {
        " ^S Save  ^Q Quit  ^O Open  F1 Keys "
    };
    let hints = Span::styled(hints_text, Style::default().fg(muted_fg).bg(hints_bg));
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

/// Build the status bar for the save-as prompt (untitled buffer first save).
///
/// Shows `" Save as: {input}_ "` in accent colour on `hints_bg`, matching the
/// go-to-line prompt style so the two prompts feel visually consistent.
pub(super) fn build_save_as_bar(app: &App, input: &str) -> Line<'static> {
    let theme = &app.theme;
    let hints_bg = theme.ui_bg;
    let accent_fg = theme.accent;

    let content = format!(" Save as: {input}_ ");
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
///
/// Builds the entire row as a **single** `Line` covering the full `area` width so
/// that every cell receives an explicit (char, style) pair in one pass.  Using
/// multiple narrow `Paragraph` renders (clear + text + indicators) left gaps where
/// only the style was set but no character was written, which could expose stale
/// buffer content from the previous frame on certain terminals.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
pub fn render_info_line(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    // Hard-clear the row first so no stale cell content can survive from any
    // prior frame.  `Clear` writes Cell::default() (space + default style) to
    // every cell, giving us a clean slate before our spans land on top.
    f.render_widget(Clear, area);

    let (row, col) = app.textarea.cursor();
    let text = format!(
        " Ln {}, Col {} · {} words ",
        format_thousands(row + 1),
        format_thousands(col + 1),
        format_thousands(app.word_count),
    );

    // Mode indicators — right-aligned, accent coloured.
    // Each active mode contributes a token; they are concatenated into a single
    // right-aligned span so they stack naturally.
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

    let text_style = Style::default().fg(theme.muted).bg(theme.bg);
    let ind_style = Style::default().fg(theme.accent).bg(theme.bg);
    let bg_style = Style::default().bg(theme.bg);

    // How many display columns are available for each zone.
    let text_width = (text.chars().count() as u16).min(area.width);
    let remaining = area.width.saturating_sub(text_width);
    let ind_width = (indicators.chars().count() as u16).min(remaining);
    let gap_width = remaining.saturating_sub(ind_width);

    // Char-safe truncation (the widths above are in chars, not bytes).
    let text_str: String = text.chars().take(text_width as usize).collect();
    let ind_str: String = indicators.chars().take(ind_width as usize).collect();
    let gap_str = " ".repeat(gap_width as usize);

    // Render the entire row in one shot: [word-count][gap][indicators]
    // Every cell in `area` gets an explicit character + style so the diff
    // correctly replaces any previous content.
    let line = Line::from(vec![
        Span::styled(text_str, text_style),
        Span::styled(gap_str, bg_style),
        Span::styled(ind_str, ind_style),
    ]);
    f.render_widget(Paragraph::new(line).style(bg_style), area);
}
