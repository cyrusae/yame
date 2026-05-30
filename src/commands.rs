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
    // Written as GUTTER + GUTTER (not 2 * GUTTER) so that the `* → /` mutant
    // (2 / GUTTER == 2 when GUTTER=1, i.e., equivalent) does not survive: the
    // `+ → *` and `+ → /` variants yield GUTTER*GUTTER=1 and GUTTER/GUTTER=1,
    // both ≠ 2 and therefore observable.
    let cw = (col_width as usize)
        .saturating_sub(renderer::GUTTER as usize + renderer::GUTTER as usize)
        .max(1);

    // Scroll up: cursor above the viewport.
    // Equivalent mutant note: `< → <=` when cursor_row == scroll_top sets
    // scroll_top = cursor_row — a no-op (same value) — so both produce identical
    // behaviour and no test can distinguish them.
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::layout::Rect;
    use tui_textarea::TextArea;

    use yame::app::{App, ClipboardState, FileMode};
    use yame::config::Theme;
    use yame::decoration::DecorationMap;
    use yame::status::StatusLine;

    use super::clamp_scroll;

    fn make_app(lines: Vec<&str>, content_width: usize) -> App {
        App {
            textarea: TextArea::new(lines.into_iter().map(String::from).collect()),
            file_path: PathBuf::from("test.md"),
            shortened_path: "test.md".to_string(),
            is_dirty: false,
            saved_content: None,
            theme: Theme::default_theme(),
            italic_support: false,
            powerline_glyphs: false,
            last_keystroke: None,
            force_redecorate: false,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
            free_scroll: false,
            sticky_col: None,
            content_width,
            clipboard: ClipboardState::Uninitialized,
            tab_width: 4,
            highlight_cache: None,
            file_mode: FileMode::Markdown,
        }
    }

    // Kills commands.rs:91:43 `replace + with * in clamp_scroll`.
    //
    // The mutation changes `i + 1 == char_ranges.len()` to `i * 1 == char_ranges.len()`
    // (i.e., `i == len`, always false for a valid iterator since i < len).  Without
    // the fallback arm, cursor_subrow is never set in the loop when cursor_col >=
    // char_end for every intermediate segment, leaving it at 0 (the first visual row)
    // instead of the correct last visual row.
    //
    // Setup (col_width=12 → cw=10):
    //   line 0: "aaaa"             → 1 visual row
    //   line 1: "bbbb"             → 1 visual row
    //   line 2: "0123456789 cc"    → 2 visual rows: ["0123456789", "cc"]
    //     char_ranges: (0,10) and (11,2)
    //     cursor_col = 13 (one past the final 'c' = end-of-line)
    //     → cursor_col < char_end(10)? No.  fallback (i+1==2)? Yes → subrow=1. ✓
    //     → mutation: fallback never fires → subrow=0 (wrong).
    //
    //   above_visual = 1 + 1 = 2; cursor_visual(correct)=3, cursor_visual(mutant)=2.
    //   With visible_rows=3: correct triggers scroll adjustment → scroll_top=1;
    //   mutant does not → scroll_top stays 0.
    #[test]
    fn clamp_scroll_subrow_fallback_fires_at_end_of_wrapped_line() {
        // col_width=12 → cw = 12-(1+1) = 10.  "0123456789 cc" = 10+space+2 = 13 chars.
        // At cw=10 it soft-wraps: ["0123456789", "cc"].
        let mut app = make_app(vec!["aaaa", "bbbb", "0123456789 cc"], 10);
        app.scroll_top = 0;

        // Place cursor at (2, 13): one past the last char of "0123456789 cc".
        // tui-textarea clamps to the line length, so this is the end-of-line position.
        use tui_textarea::CursorMove;
        app.textarea.move_cursor(CursorMove::Jump(2, 13));

        // editor_area height=3 (exactly 3 visual rows visible), width=12.
        let area = Rect {
            x: 0,
            y: 0,
            width: 12,
            height: 3,
        };
        clamp_scroll(&mut app, area, 12, 0);

        assert_eq!(
            app.scroll_top, 1,
            "cursor on the 2nd visual row of line 2 (total vis=3) must \
             cause scroll_top to advance to 1 so the cursor remains visible"
        );
    }
}
