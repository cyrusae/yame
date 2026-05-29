use std::io;
use std::time::Duration;

use ratatui::layout::Rect;

use yame::app::App;
use yame::renderer;
use yame::status::StatusMode;

#[mutants::skip] // Calls std::fs::write — I/O side effect.
pub(super) fn handle_save(app: &mut App) -> io::Result<()> {
    let lines = app.textarea.lines();
    // Always write 0 bytes for an empty buffer — consistent regardless of whether
    // the file was empty at open time.  A non-empty save always gets a POSIX newline.
    let content = if lines == [""] {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    // Create parent directories if they don't exist (e.g. `yame notes/new.md`).
    if let Some(parent) = app.file_path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        app.status.set_dismissible(format!("⚠ Save failed: {e}"));
        return Ok(());
    }
    match std::fs::write(&app.file_path, &content) {
        Ok(()) => {
            app.saved_content = Some(app.textarea.lines().to_vec());
            app.is_dirty = false;
            app.status.set_timed("Saved.", Duration::from_millis(1500));
        }
        Err(e) => {
            app.status.set_dismissible(format!("⚠ Save failed: {e}"));
        }
    }
    Ok(())
}

/// Returns true if the app should exit.
pub(super) fn handle_exit(app: &mut App) -> bool {
    if app.is_dirty {
        app.status.mode = StatusMode::ExitPrompt;
        false
    } else {
        true
    }
}

/// Clamp `app.scroll_top` so the cursor remains visible within `editor_area`.
///
/// Extracted from the draw closure so that `terminal.draw` can be a pure render
/// with no state mutations.
///
/// `col_width`      – full column width in terminal cells (includes gutters).
/// `bottom_padding` – virtual empty rows to keep below the cursor.
pub(super) fn clamp_scroll(
    app: &mut App,
    editor_area: Rect,
    col_width: u16,
    bottom_padding: usize,
) {
    let (cursor_row, cursor_col) = app.textarea.cursor();
    let visible_rows = editor_area.height as usize;
    let lines = app.textarea.lines();
    let cw = (col_width as usize)
        .saturating_sub(2 * renderer::GUTTER as usize)
        .max(1);

    // Scroll up: cursor above the viewport
    if cursor_row < app.scroll_top {
        app.scroll_top = cursor_row;
    }

    // Scroll down: check cursor's visual (post-wrap) position
    let above_visual: usize = lines
        .get(app.scroll_top..cursor_row.min(lines.len()))
        .unwrap_or(&[])
        .iter()
        .map(|l| renderer::wrap_line(l, cw).len())
        .sum();

    let cursor_line_str = lines.get(cursor_row).map_or("", |s| s.as_str());
    let cursor_wraps = renderer::wrap_line(cursor_line_str, cw);
    let cursor_subrow = {
        let char_ranges = renderer::wrap_char_ranges(cursor_line_str, &cursor_wraps);
        let mut subrow = 0usize;
        for (i, &(char_start, char_len)) in char_ranges.iter().enumerate() {
            let char_end = char_start + char_len;
            if cursor_col < char_end || i + 1 == char_ranges.len() {
                subrow = i;
                break;
            }
        }
        subrow
    };

    let cursor_visual = above_visual + cursor_subrow;

    if cursor_visual + bottom_padding >= visible_rows {
        let headroom = visible_rows.saturating_sub(1 + cursor_subrow + bottom_padding);
        let mut remaining = headroom;
        let mut new_top = cursor_row;
        while new_top > 0 {
            let prev_wraps =
                renderer::wrap_line(lines.get(new_top - 1).map_or("", |s| s.as_str()), cw).len();
            if prev_wraps > remaining {
                break;
            }
            remaining -= prev_wraps;
            new_top -= 1;
        }
        app.scroll_top = new_top;
    }
}
