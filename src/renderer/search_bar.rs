//! Renders the search / search-and-replace bar at the top of the editor column.

use ratatui::{Frame, buffer::Buffer, layout::Rect, style::Color};

use crate::app::App;

/// Number of terminal rows occupied by the search bar.
///
/// Returns 0 when no search is active, 1 for search-only, 2 for search+replace.
pub fn search_bar_height(app: &App) -> u16 {
    match &app.search {
        None => 0,
        Some(s) => if s.show_replace { 2 } else { 1 },
    }
}

/// Render the search (and optionally replace) bar into `area`.
///
/// `area` should be exactly `search_bar_height` rows tall and span the full
/// terminal width (same as `info_line` / `status_bar`).
#[mutants::skip] // Writes into ratatui Frame — void, not testable via return value.
pub fn render_search_bar(f: &mut Frame, area: Rect, app: &App) {
    let Some(search) = &app.search else { return };
    let theme = &app.theme;

    let bar_bg = theme.ui_bar;
    let label_fg = theme.muted;
    let text_fg = theme.text;
    let error_fg = theme.warning;
    let match_info_fg = theme.accent;

    // ── Search row ────────────────────────────────────────────────────────────
    let search_row = Rect { height: 1, y: area.y, ..area };
    let buf = f.buffer_mut();

    // Flood-fill bar background.
    for col in 0..area.width {
        buf[(area.x + col, search_row.y)].set_bg(bar_bg);
        if search.show_replace && area.height >= 2 {
            buf[(area.x + col, search_row.y + 1)].set_bg(bar_bg);
        }
    }

    // Build search row content.
    let label = if search.regex_mode { " Search [re]: " } else { " Search: " };
    let query_display = if search.focus_search {
        format!("{}█", search.query)
    } else {
        search.query.clone()
    };

    // Match counter / error badge (right side).
    let right_badge = if search.regex_error {
        " [bad regex] ".to_string()
    } else if search.query.is_empty() {
        String::new()
    } else if search.matches.is_empty() {
        " [no matches] ".to_string()
    } else {
        format!(" [{}/{}] ", search.current + 1, search.matches.len())
    };

    // Render label.
    let mut x = area.x;
    for ch in label.chars() {
        if x >= area.x + area.width { break; }
        buf[(x, search_row.y)].set_char(ch).set_fg(label_fg).set_bg(bar_bg);
        x += 1;
    }

    // Render query text (error color when regex is invalid).
    let qfg = if search.regex_error { error_fg } else { text_fg };
    for ch in query_display.chars() {
        if x >= area.x + area.width { break; }
        buf[(x, search_row.y)].set_char(ch).set_fg(qfg).set_bg(bar_bg);
        x += 1;
    }

    // Render right badge right-aligned.
    if !right_badge.is_empty() {
        let badge_chars: Vec<char> = right_badge.chars().collect();
        let badge_len = badge_chars.len() as u16;
        let badge_x = area.x + area.width.saturating_sub(badge_len);
        let rfg = if search.regex_error { error_fg } else { match_info_fg };
        for (i, &ch) in badge_chars.iter().enumerate() {
            let cx = badge_x + i as u16;
            if cx >= area.x + area.width { break; }
            buf[(cx, search_row.y)].set_char(ch).set_fg(rfg).set_bg(bar_bg);
        }
    }

    // ── Replace row ───────────────────────────────────────────────────────────
    if search.show_replace && area.height >= 2 {
        let replace_y = search_row.y + 1;
        let rlabel = " Replace: ";
        let replace_display = if !search.focus_search {
            format!("{}█", search.replace)
        } else {
            search.replace.clone()
        };

        // Hint right-aligned.
        let hint = " [Enter=replace] [Ctrl+A=all] ";
        let hint_chars: Vec<char> = hint.chars().collect();
        let hint_len = hint_chars.len() as u16;
        let hint_x = area.x + area.width.saturating_sub(hint_len);

        let mut rx = area.x;
        for ch in rlabel.chars() {
            if rx >= area.x + area.width { break; }
            buf[(rx, replace_y)].set_char(ch).set_fg(label_fg).set_bg(bar_bg);
            rx += 1;
        }
        for ch in replace_display.chars() {
            if rx >= area.x + area.width || rx >= hint_x { break; }
            buf[(rx, replace_y)].set_char(ch).set_fg(text_fg).set_bg(bar_bg);
            rx += 1;
        }
        for (i, &ch) in hint_chars.iter().enumerate() {
            let cx = hint_x + i as u16;
            if cx >= area.x + area.width { break; }
            buf[(cx, replace_y)].set_char(ch).set_fg(label_fg).set_bg(bar_bg);
        }
    }

}

// ── Search help modal ─────────────────────────────────────────────────────────

/// Column widths (terminal cells) for the two key–description pairs per row.
const C1: u16 = 13; // key col 1
const C2: u16 = 11; // desc col 1
const C3: u16 = 12; // key col 2
const C4: u16 = 11; // desc col 2

/// Inner width: 2-space left-pad + C1 + sep + C2 + sep + C3 + sep + C4 + 1-trailing.
const INNER_W: u16 = 2 + C1 + 1 + C2 + 1 + C3 + 1 + C4 + 1;
/// Total box width including left and right border glyphs.
const BOX_W: u16 = INNER_W + 2;

/// Shortcut table: (key1, desc1, key2, desc2).
const HELP: &[(&str, &str, &str, &str)] = &[
    ("Enter/Ctrl+F", "next match",  "Shift+Enter", "prev match"),
    ("Alt+R",        "regex mode",  "Ctrl+H",      "replace row"),
    ("Tab",          "swap field",  "Ctrl+A",       "replace all"),
    ("Esc",          "close",       "",             ""),
];

/// Height: top border + one row per entry + bottom border.
const BOX_H: u16 = HELP.len() as u16 + 2;

/// Render a floating keyboard-shortcut cheatsheet over `editor_area`.
///
/// The box is centered horizontally and anchored at the top of the editor
/// area.  It is only shown while `search.show_help` is `true` — the first
/// keypress inside search mode clears that flag, making the overlay ephemeral.
#[mutants::skip] // Writes into ratatui Frame — void, not testable via return value.
pub fn render_search_help_modal(f: &mut Frame, editor_area: Rect, app: &App) {
    let Some(search) = &app.search else { return };
    if !search.show_help {
        return;
    }
    if editor_area.width < BOX_W || editor_area.height < BOX_H {
        return;
    }

    let theme = &app.theme;
    let bg = theme.ui_bar;
    let border_fg = theme.muted;
    let key_fg = theme.accent;
    let desc_fg = theme.text;

    // Center horizontally; sit flush at the top of the editor area.
    let bx = editor_area.x + (editor_area.width.saturating_sub(BOX_W)) / 2;
    let by = editor_area.y;

    let buf = f.buffer_mut();

    // ── Top border with title ─────────────────────────────────────────────────
    const TITLE: &str = " Search shortcuts ";
    let title_chars: Vec<char> = TITLE.chars().collect();
    let title_len = title_chars.len() as u16;
    let dash_total = BOX_W - 2; // positions between corner glyphs

    buf[(bx, by)].set_char('╭').set_fg(border_fg).set_bg(bg);
    for i in 0..dash_total {
        let cx = bx + 1 + i;
        // Place title starting 1 dash in from the left corner.
        if i >= 1 && i < 1 + title_len {
            let ch = title_chars[(i - 1) as usize];
            let fg = if ch == ' ' { border_fg } else { theme.text };
            buf[(cx, by)].set_char(ch).set_fg(fg).set_bg(bg);
        } else {
            buf[(cx, by)].set_char('─').set_fg(border_fg).set_bg(bg);
        }
    }
    buf[(bx + BOX_W - 1, by)].set_char('╮').set_fg(border_fg).set_bg(bg);

    // ── Content rows ──────────────────────────────────────────────────────────
    for (row_i, &(k1, d1, k2, d2)) in HELP.iter().enumerate() {
        let ry = by + 1 + row_i as u16;
        let mut cx = bx;

        buf[(cx, ry)].set_char('│').set_fg(border_fg).set_bg(bg);
        cx += 1;

        // 2-space left padding.
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;

        cx = put_padded(buf, cx, ry, k1, C1, key_fg, bg);
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
        cx = put_padded(buf, cx, ry, d1, C2, desc_fg, bg);
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
        cx = put_padded(buf, cx, ry, k2, C3, key_fg, bg);
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
        cx = put_padded(buf, cx, ry, d2, C4, desc_fg, bg);

        // 1 trailing space then right border.
        buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
        buf[(cx, ry)].set_char('│').set_fg(border_fg).set_bg(bg);
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    let bottom_y = by + BOX_H - 1;
    buf[(bx, bottom_y)].set_char('╰').set_fg(border_fg).set_bg(bg);
    for i in 0..dash_total {
        buf[(bx + 1 + i, bottom_y)].set_char('─').set_fg(border_fg).set_bg(bg);
    }
    buf[(bx + BOX_W - 1, bottom_y)].set_char('╯').set_fg(border_fg).set_bg(bg);
}

/// Write `s` left-aligned into a `width`-cell field at `(x, y)`, padding
/// unused cells with spaces.  Returns `x + width`.
fn put_padded(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    s: &str,
    width: u16,
    fg: Color,
    bg: Color,
) -> u16 {
    let end = x + width;
    let mut cx = x;
    for ch in s.chars() {
        if cx >= end {
            break;
        }
        buf[(cx, y)].set_char(ch).set_fg(fg).set_bg(bg);
        cx += 1;
    }
    while cx < end {
        buf[(cx, y)].set_char(' ').set_bg(bg);
        cx += 1;
    }
    end
}
