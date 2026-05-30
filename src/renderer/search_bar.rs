//! Renders the search / search-and-replace bar at the top of the editor column.

use ratatui::{Frame, layout::Rect};

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
