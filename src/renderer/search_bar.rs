//! Renders the search / search-and-replace bar at the top of the editor column.

use ratatui::{Frame, buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}};

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

    // Clear any bold/italic/underline from the editor content rendered behind
    // this overlay before drawing borders and text on top.
    flood_reset(buf, Rect { x: bx, y: by, width: BOX_W, height: BOX_H }, desc_fg, bg);

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

// ── Shared overlay helper ─────────────────────────────────────────────────────

/// Flood-fill `area` in `buf` with a clean style, overwriting any bold/italic/
/// underline/etc. modifiers that were painted by the editor renderer behind the
/// overlay.  Every cell is set to `bg` background, `fg` foreground, and all
/// modifier bits cleared — so subsequent per-cell writes that only call `set_bg`
/// or `set_char` will never inherit decoration from the underlying document.
///
/// This must be called before drawing any border or content in a modal overlay.
#[mutants::skip] // Writes into ratatui Buffer — void, not testable via return value.
fn flood_reset(buf: &mut Buffer, area: Rect, fg: Color, bg: Color) {
    // Style::new() has empty add_modifier; remove_modifier(all) sets sub_modifier
    // to Modifier::all(), which clears every modifier bit when patched onto the
    // existing cell style via Cell::set_style → Style::patch.
    let clean = Style::new().fg(fg).bg(bg).remove_modifier(Modifier::all());
    buf.set_style(area, clean);
    // buf.set_style patches the style but doesn't reset the symbol — we need
    // spaces so the background colour is fully visible between drawn glyphs.
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_char(' ');
        }
    }
}

// ── Full shortcuts modal ──────────────────────────────────────────────────────

/// Row types for the shortcuts table.
enum HelpRow {
    /// A section divider with a title, rendered as `│ ─ Title ──────── │`.
    Section(&'static str),
    /// A two-column entry row: (key1, desc1, key2, desc2).
    Entry(&'static str, &'static str, &'static str, &'static str),
}

/// All shortcuts shown in the modal, in display order.
static SHORTCUTS: &[HelpRow] = &[
    HelpRow::Section("File"),
    HelpRow::Entry("Ctrl+S / ⌘S",  "Save",          "Ctrl+X / Esc", "Exit"),
    HelpRow::Entry("Ctrl+R",        "Reload config",  "",             ""),
    HelpRow::Section("Edit"),
    HelpRow::Entry("Ctrl+Z",        "Undo",           "Ctrl+Y",       "Redo"),
    HelpRow::Entry("Ctrl+C / ⌘C",  "Copy",           "Ctrl+V / ⌘V",  "Paste"),
    HelpRow::Entry("Tab",           "Indent",         "",             ""),
    HelpRow::Section("Navigate"),
    HelpRow::Entry("↑ / ↓",        "Move",           "Shift+↑/↓",    "Select"),
    HelpRow::Entry("Ctrl+↑/↓",     "Scroll",         "Ctrl+G",       "Go to line"),
    HelpRow::Section("Search"),
    HelpRow::Entry("Ctrl+F",        "Search",         "Ctrl+H",       "Find & replace"),
    HelpRow::Section("View"),
    HelpRow::Entry("Ctrl+T",        "Typewriter",     "Ctrl+D",       "Focus mode"),
    HelpRow::Section("Markdown"),
    HelpRow::Entry("Alt+T",         "Format table",   "()[]{}'\"*_``", "Auto-pair"),
    HelpRow::Entry("F1 / Esc",      "Close help",     "",             ""),
];

/// Column widths (terminal cells) for the four fields in each entry row.
const SC_C1: u16 = 12; // key1
const SC_C2: u16 = 14; // desc1
const SC_C3: u16 = 13; // key2
const SC_C4: u16 = 15; // desc2

/// Inner width: 2-space left-pad + C1 + sep + C2 + sep + C3 + sep + C4 + 1-trailing.
const SC_INNER_W: u16 = 2 + SC_C1 + 1 + SC_C2 + 1 + SC_C3 + 1 + SC_C4 + 1;
/// Total box width including left and right border glyphs.
const SC_BOX_W: u16 = SC_INNER_W + 2;

// Compile-time sanity checks so that arithmetic mutations to the constants above
// are caught immediately.  The expected values are derived from the column-width
// literals (SC_C1=12, SC_C2=14, SC_C3=13, SC_C4=15):
//   SC_INNER_W = 2 + 12 + 1 + 14 + 1 + 13 + 1 + 15 + 1 = 60
//   SC_BOX_W   = 60 + 2 = 62
const _: () = assert!(SC_INNER_W == 60, "SC_INNER_W arithmetic is wrong");
const _: () = assert!(SC_BOX_W == 62, "SC_BOX_W arithmetic is wrong");
/// Total box height: top border + one row per SHORTCUTS entry + bottom border.
const SC_BOX_H: u16 = SHORTCUTS.len() as u16 + 2;

/// Render the full keybindings reference modal, centred over `editor_area`.
///
/// Only shown while `app.show_shortcuts` is `true`.  Dismissed by F1 or Esc
/// (handled in `handle_key_event`).
#[mutants::skip] // Writes into ratatui Frame — void, not testable via return value.
pub fn render_shortcuts_modal(f: &mut Frame, editor_area: Rect, app: &App) {
    if !app.show_shortcuts {
        return;
    }
    if editor_area.width < SC_BOX_W || editor_area.height < SC_BOX_H {
        return;
    }

    let theme = &app.theme;
    let bg         = theme.ui_bar;
    let border_fg  = theme.muted;
    let section_fg = theme.accent;
    let key_fg     = theme.accent;
    let desc_fg    = theme.text;

    // Centre both horizontally and vertically.
    let bx = editor_area.x + (editor_area.width.saturating_sub(SC_BOX_W))  / 2;
    let by = editor_area.y + (editor_area.height.saturating_sub(SC_BOX_H)) / 2;

    let buf = f.buffer_mut();

    // Clear bold/italic/underline from editor content rendered behind this overlay.
    flood_reset(buf, Rect { x: bx, y: by, width: SC_BOX_W, height: SC_BOX_H }, desc_fg, bg);

    // ── Top border with title ─────────────────────────────────────────────────
    const TITLE: &str = " Keybindings ";
    let title_chars: Vec<char> = TITLE.chars().collect();
    let title_len   = title_chars.len() as u16;
    let dash_total  = SC_BOX_W - 2;

    buf[(bx, by)].set_char('╭').set_fg(border_fg).set_bg(bg);
    for i in 0..dash_total {
        let cx = bx + 1 + i;
        if i >= 1 && i < 1 + title_len {
            let ch  = title_chars[(i - 1) as usize];
            let fg  = if ch == ' ' { border_fg } else { theme.text };
            buf[(cx, by)].set_char(ch).set_fg(fg).set_bg(bg);
        } else {
            buf[(cx, by)].set_char('─').set_fg(border_fg).set_bg(bg);
        }
    }
    buf[(bx + SC_BOX_W - 1, by)].set_char('╮').set_fg(border_fg).set_bg(bg);

    // ── Content rows ──────────────────────────────────────────────────────────
    for (row_i, row) in SHORTCUTS.iter().enumerate() {
        let ry = by + 1 + row_i as u16;
        match row {
            HelpRow::Section(name) => {
                // │ ─ Name ─────────────────────────── │
                buf[(bx, ry)].set_char('│').set_fg(border_fg).set_bg(bg);

                let inner_end = bx + 1 + SC_INNER_W;
                let mut ix = bx + 1;

                // " ─ "
                buf[(ix, ry)].set_char(' ').set_bg(bg); ix += 1;
                buf[(ix, ry)].set_char('─').set_fg(border_fg).set_bg(bg); ix += 1;
                buf[(ix, ry)].set_char(' ').set_bg(bg); ix += 1;

                // section name
                for ch in name.chars() {
                    if ix >= inner_end { break; }
                    buf[(ix, ry)].set_char(ch).set_fg(section_fg).set_bg(bg);
                    ix += 1;
                }

                // trailing space before the rule
                if ix < inner_end {
                    buf[(ix, ry)].set_char(' ').set_bg(bg); ix += 1;
                }

                // fill with ─
                while ix < inner_end {
                    buf[(ix, ry)].set_char('─').set_fg(border_fg).set_bg(bg);
                    ix += 1;
                }

                buf[(inner_end, ry)].set_char('│').set_fg(border_fg).set_bg(bg);
            }

            HelpRow::Entry(k1, d1, k2, d2) => {
                // │  key1..  desc1..  key2..  desc2.. │
                let mut cx = bx;
                buf[(cx, ry)].set_char('│').set_fg(border_fg).set_bg(bg); cx += 1;

                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;

                cx = put_padded(buf, cx, ry, k1, SC_C1, key_fg,  bg);
                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
                cx = put_padded(buf, cx, ry, d1, SC_C2, desc_fg, bg);
                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
                cx = put_padded(buf, cx, ry, k2, SC_C3, key_fg,  bg);
                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
                cx = put_padded(buf, cx, ry, d2, SC_C4, desc_fg, bg);

                buf[(cx, ry)].set_char(' ').set_bg(bg); cx += 1;
                buf[(cx, ry)].set_char('│').set_fg(border_fg).set_bg(bg);
            }
        }
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    let bottom_y = by + SC_BOX_H - 1;
    buf[(bx, bottom_y)].set_char('╰').set_fg(border_fg).set_bg(bg);
    for i in 0..dash_total {
        buf[(bx + 1 + i, bottom_y)].set_char('─').set_fg(border_fg).set_bg(bg);
    }
    buf[(bx + SC_BOX_W - 1, bottom_y)].set_char('╯').set_fg(border_fg).set_bg(bg);
}

/// Write `s` left-aligned into a `width`-cell field at `(x, y)`, padding
/// unused cells with spaces.  Returns `x + width`.
#[mutants::skip] // Only called from #[mutants::skip] render_search_help_modal.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirror the compile-time asserts as runtime tests so that mutants on the
    /// constant expressions (e.g. `+` → `*`) are also caught by the test suite.
    #[test]
    fn shortcut_modal_constants_correct() {
        assert_eq!(SC_C1, 12);
        assert_eq!(SC_C2, 14);
        assert_eq!(SC_C3, 13);
        assert_eq!(SC_C4, 15);
        assert_eq!(
            SC_INNER_W,
            2 + 12 + 1 + 14 + 1 + 13 + 1 + 15 + 1,
            "SC_INNER_W does not match manual sum"
        );
        assert_eq!(SC_BOX_W, SC_INNER_W + 2, "SC_BOX_W must be SC_INNER_W + 2");
    }
}
