//! Renders the settings modal (F2) as a floating overlay.

use ratatui::{Frame, buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}};

use crate::app::App;
use crate::settings::{APPEARANCE_EXPAND_IDX, FieldKind, SettingsModal, TAB_NAMES};

use super::search_bar::flood_reset;

// ---------------------------------------------------------------------------
// Layout constants
// ---------------------------------------------------------------------------

/// Total modal width (terminal columns), including border glyphs.
pub const MODAL_W: u16 = 56;
/// Number of field rows visible at once (the scrollable area height).
pub const VISIBLE_FIELDS: usize = 12;
/// Total modal height: top border + tab row + separator + fields + separator + hint + bottom border.
pub const MODAL_H: u16 = VISIBLE_FIELDS as u16 + 6;

// Label column width inside the modal.
const LABEL_W: u16 = 22;
// Value column starts after label.
const VALUE_X_OFFSET: u16 = LABEL_W + 2; // "│ " prefix + label + " "

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render the settings modal centred over `editor_area`.
///
/// Only drawn while `app.settings` is `Some`.  Dismissed by F2 or Esc
/// (handled in `handle_key_event`).
#[mutants::skip] // Writes into ratatui Frame — void, not testable via return value.
pub fn render_settings_modal(f: &mut Frame, editor_area: Rect, app: &App) {
    let Some(modal) = &app.settings else { return };

    if editor_area.width < MODAL_W || editor_area.height < MODAL_H {
        return;
    }

    let bx = editor_area.x + (editor_area.width.saturating_sub(MODAL_W)) / 2;
    let by = editor_area.y + (editor_area.height.saturating_sub(MODAL_H)) / 2;

    let theme = &app.theme;
    let bg = theme.ui_bar;
    let border_fg = theme.muted;
    let title_fg = theme.text;
    let active_tab_fg = theme.accent;
    let inactive_tab_fg = theme.muted;
    let label_fg = theme.text;
    let value_fg = theme.accent;
    let selected_bg = theme.ui_bg;
    let hint_fg = theme.muted;

    let buf = f.buffer_mut();

    // Clear the area, stripping any editor decorations behind the modal.
    flood_reset(buf, Rect { x: bx, y: by, width: MODAL_W, height: MODAL_H }, title_fg, bg);

    // ── Top border with title ─────────────────────────────────────────────────
    let inner_w = MODAL_W - 2;
    draw_top_border(buf, bx, by, inner_w, " Settings ", border_fg, title_fg, bg);

    // ── Tab bar ───────────────────────────────────────────────────────────────
    let tab_y = by + 1;
    draw_tab_bar(buf, bx, tab_y, inner_w, modal, active_tab_fg, inactive_tab_fg, bg, border_fg);

    // ── Separator ─────────────────────────────────────────────────────────────
    let sep_y = by + 2;
    draw_hline(buf, bx, sep_y, MODAL_W, border_fg, bg);

    // ── Field rows ────────────────────────────────────────────────────────────
    let fields_start_y = by + 3;
    let field_count = modal.field_count();
    let end_idx = (modal.scroll + VISIBLE_FIELDS).min(field_count);

    for (row, field_idx) in (modal.scroll..end_idx).enumerate() {
        let ry = fields_start_y + row as u16;
        let selected = field_idx == modal.field;
        let row_bg = if selected { selected_bg } else { bg };

        // Clear row
        for col in 0..MODAL_W {
            buf[(bx + col, ry)].set_bg(row_bg);
        }
        buf[(bx, ry)].set_char('│').set_fg(border_fg).set_bg(bg);
        buf[(bx + MODAL_W - 1, ry)].set_char('│').set_fg(border_fg).set_bg(bg);

        // Expand toggle row
        if modal.tab == 0 && field_idx == APPEARANCE_EXPAND_IDX {
            let label = if modal.expanded { "▾ Show less" } else { "▸ Show more" };
            put_str(buf, bx + 2, ry, label, inactive_tab_fg, row_bg);
            continue;
        }

        // Normal field row
        let Some(def) = modal.field_def(field_idx) else { continue };

        // Label
        put_padded(buf, bx + 2, ry, def.label, LABEL_W, label_fg, row_bg);

        // Value / edit buffer
        let inner_x = bx + 2 + VALUE_X_OFFSET;
        let value_w = (MODAL_W as usize).saturating_sub(2 + 2 + VALUE_X_OFFSET as usize + 3);

        if selected && modal.editing {
            // Text / number / color edit: show input buffer with cursor block.
            let display = format!("{}█", modal.input_buf);
            let max_chars = value_w.saturating_sub(3);
            put_str_clipped(buf, inner_x, ry, &display, max_chars as u16, value_fg, row_bg);
            // Live swatch for hex color fields.
            if def.kind == FieldKind::HexColor
                && let Some((r, g, b)) = modal.input_rgb()
            {
                let swatch_x = inner_x
                    + (max_chars as u16).min(display.chars().count() as u16 + 1);
                put_swatch(buf, swatch_x, ry, Color::Rgb(r, g, b), row_bg);
            }
        } else {
            let value = modal.field_value(field_idx);
            match def.kind {
                FieldKind::Toggle => {
                    let (indicator, fg) = if value == "on" {
                        ("● on ", value_fg)
                    } else {
                        ("○ off", hint_fg)
                    };
                    put_str(buf, inner_x, ry, indicator, fg, row_bg);
                }
                FieldKind::HexColor => {
                    let max_chars = value_w.saturating_sub(3) as u16;
                    put_str_clipped(buf, inner_x, ry, &value, max_chars, value_fg, row_bg);
                    if let Some((r, g, b)) = modal.field_rgb(field_idx) {
                        let swatch_x = inner_x + max_chars.min(value.chars().count() as u16) + 1;
                        put_swatch(buf, swatch_x, ry, Color::Rgb(r, g, b), row_bg);
                    }
                }
                FieldKind::Cycler { .. } => {
                    let display = if value.is_empty() {
                        "‹ (default) ›".to_string()
                    } else {
                        format!("‹ {} ›", value)
                    };
                    put_str_clipped(buf, inner_x, ry, &display, value_w as u16, value_fg, row_bg);
                }
                _ => {
                    put_str_clipped(buf, inner_x, ry, &value, value_w as u16, value_fg, row_bg);
                }
            }
        }
    }

    // ── Separator ─────────────────────────────────────────────────────────────
    let sep2_y = fields_start_y + VISIBLE_FIELDS as u16;
    draw_hline(buf, bx, sep2_y, MODAL_W, border_fg, bg);

    // ── Scroll indicator ──────────────────────────────────────────────────────
    let hint_y = sep2_y + 1;
    let scroll_hint = if field_count > VISIBLE_FIELDS {
        format!(" {}/{} ", modal.field + 1, field_count)
    } else {
        String::new()
    };
    // Left: nav hint
    put_str(buf, bx + 2, hint_y, "↑↓ nav  Tab/←→ switch  Enter  Esc close", hint_fg, bg);
    // Right: scroll position
    if !scroll_hint.is_empty() {
        let sc: Vec<char> = scroll_hint.chars().collect();
        let sc_len = sc.len() as u16;
        let sx = bx + MODAL_W - 1 - sc_len;
        for (i, ch) in sc.iter().enumerate() {
            buf[(sx + i as u16, hint_y)]
                .set_char(*ch)
                .set_fg(hint_fg)
                .set_bg(bg);
        }
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    let bottom_y = by + MODAL_H - 1;
    draw_bottom_border(buf, bx, bottom_y, inner_w, border_fg, bg);
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn draw_top_border(
    buf: &mut Buffer,
    bx: u16, by: u16,
    inner_w: u16,
    title: &str,
    border_fg: Color,
    title_fg: Color,
    bg: Color,
) {
    buf[(bx, by)].set_char('╭').set_fg(border_fg).set_bg(bg);
    let title_chars: Vec<char> = title.chars().collect();
    let title_len = title_chars.len() as u16;
    for i in 0..inner_w {
        let cx = bx + 1 + i;
        if i >= 1 && i < 1 + title_len {
            let ch = title_chars[(i - 1) as usize];
            let fg = if ch == ' ' { border_fg } else { title_fg };
            buf[(cx, by)].set_char(ch).set_fg(fg).set_bg(bg);
        } else {
            buf[(cx, by)].set_char('─').set_fg(border_fg).set_bg(bg);
        }
    }
    buf[(bx + inner_w + 1, by)].set_char('╮').set_fg(border_fg).set_bg(bg);
}

fn draw_bottom_border(buf: &mut Buffer, bx: u16, by: u16, inner_w: u16, border_fg: Color, bg: Color) {
    buf[(bx, by)].set_char('╰').set_fg(border_fg).set_bg(bg);
    for i in 0..inner_w {
        buf[(bx + 1 + i, by)].set_char('─').set_fg(border_fg).set_bg(bg);
    }
    buf[(bx + inner_w + 1, by)].set_char('╯').set_fg(border_fg).set_bg(bg);
}

fn draw_hline(buf: &mut Buffer, bx: u16, by: u16, total_w: u16, border_fg: Color, bg: Color) {
    buf[(bx, by)].set_char('├').set_fg(border_fg).set_bg(bg);
    for i in 1..total_w - 1 {
        buf[(bx + i, by)].set_char('─').set_fg(border_fg).set_bg(bg);
    }
    buf[(bx + total_w - 1, by)].set_char('┤').set_fg(border_fg).set_bg(bg);
}

#[allow(clippy::too_many_arguments)]
fn draw_tab_bar(
    buf: &mut Buffer,
    bx: u16, by: u16,
    inner_w: u16,
    modal: &SettingsModal,
    active_fg: Color,
    inactive_fg: Color,
    bg: Color,
    border_fg: Color,
) {
    buf[(bx, by)].set_char('│').set_fg(border_fg).set_bg(bg);
    let mut x = bx + 2;
    for (i, name) in TAB_NAMES.iter().enumerate() {
        let fg = if i == modal.tab { active_fg } else { inactive_fg };
        let bold = i == modal.tab;
        for ch in name.chars() {
            if x >= bx + inner_w {
                break;
            }
            let cell = buf[(x, by)].set_char(ch).set_fg(fg).set_bg(bg);
            if bold {
                cell.set_style(Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD));
            }
            x += 1;
        }
        if i + 1 < TAB_NAMES.len() {
            if x < bx + inner_w {
                buf[(x, by)].set_char(' ').set_bg(bg);
                x += 1;
            }
            if x < bx + inner_w {
                buf[(x, by)].set_char('·').set_fg(inactive_fg).set_bg(bg);
                x += 1;
            }
            if x < bx + inner_w {
                buf[(x, by)].set_char(' ').set_bg(bg);
                x += 1;
            }
        }
    }
    buf[(bx + inner_w + 1, by)].set_char('│').set_fg(border_fg).set_bg(bg);
}

/// Write `s` left-aligned, padding to `width` with spaces.
fn put_padded(buf: &mut Buffer, x: u16, y: u16, s: &str, width: u16, fg: Color, bg: Color) {
    let w = width as usize;
    let mut written = 0usize;
    for (i, ch) in s.chars().take(w).enumerate() {
        buf[(x + i as u16, y)].set_char(ch).set_fg(fg).set_bg(bg);
        written = i + 1;
    }
    for i in written..w {
        buf[(x + i as u16, y)].set_char(' ').set_bg(bg);
    }
}

/// Write `s` clipping at `max_chars` terminal columns.
fn put_str_clipped(buf: &mut Buffer, x: u16, y: u16, s: &str, max_chars: u16, fg: Color, bg: Color) {
    for (i, ch) in s.chars().take(max_chars as usize).enumerate() {
        buf[(x + i as u16, y)].set_char(ch).set_fg(fg).set_bg(bg);
    }
}

fn put_str(buf: &mut Buffer, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
    for (i, ch) in s.chars().enumerate() {
        buf[(x + i as u16, y)].set_char(ch).set_fg(fg).set_bg(bg);
    }
}

/// Draw a single colored swatch cell (█).
fn put_swatch(buf: &mut Buffer, x: u16, y: u16, color: Color, bg: Color) {
    buf[(x, y)].set_char('█').set_fg(color).set_bg(bg);
}
