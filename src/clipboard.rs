use crate::app::{App, ClipboardState, get_selection_text};

/// Copy selection or current line to the system clipboard.
/// On error, posts a dismissible status message.
#[mutants::skip] // Clipboard I/O — arboard calls not testable in a headless CI environment.
pub fn handle_copy(app: &mut App) {
    let text = get_copy_text(app);
    ensure_clipboard(app);
    let err = match &mut app.clipboard {
        ClipboardState::Ready(cb) => cb.set_text(text).err().map(|e| e.to_string()),
        _ => Some("clipboard unavailable".to_string()),
    };
    if let Some(e) = err {
        app.status
            .set_dismissible(format!("⚠ Clipboard unavailable: {e}"));
    }
}

/// Paste from the system clipboard into the buffer.
#[mutants::skip] // Clipboard I/O.
pub fn handle_paste(app: &mut App) {
    ensure_clipboard(app);
    let result = match &mut app.clipboard {
        ClipboardState::Ready(cb) => cb.get_text().map_err(|e| e.to_string()),
        _ => Err("clipboard unavailable".to_string()),
    };
    match result {
        Ok(text) => {
            app.textarea.insert_str(&text);
            app.mark_keystroke();
        }
        Err(e) => {
            app.status
                .set_dismissible(format!("⚠ Clipboard unavailable: {e}"));
        }
    }
}

/// Lazily initialise `app.clipboard` on first use.
///
/// On success, transitions `Uninitialized` → `Ready`.
/// On failure, transitions `Uninitialized` → `Unavailable` so subsequent
/// copy/paste operations fail immediately without blocking the event loop.
/// An already-`Ready` or `Unavailable` clipboard is left unchanged.
#[mutants::skip] // arboard::Clipboard I/O — not available in CI.
fn ensure_clipboard(app: &mut App) {
    if matches!(app.clipboard, ClipboardState::Uninitialized) {
        app.clipboard = match arboard::Clipboard::new() {
            Ok(cb) => ClipboardState::Ready(cb),
            Err(_) => ClipboardState::Unavailable,
        };
    }
}

/// Return the selected text if a selection is active, or the current line if not.
///
/// This is the text that `handle_copy` will send to the system clipboard.
pub fn get_copy_text(app: &App) -> String {
    get_selection_text(app).unwrap_or_else(|| {
        let (row, _) = app.textarea.cursor();
        app.textarea.lines().get(row).cloned().unwrap_or_default()
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ClipboardState;
    use crate::config::Theme;
    use crate::decoration::DecorationMap;
    use crate::status::StatusLine;
    use std::path::PathBuf;
    use tui_textarea::{CursorMove, TextArea};

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
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
            content_width: 0,
            clipboard: ClipboardState::Uninitialized,
            tab_width: 4,
            highlight_cache: None,
            file_mode: crate::app::FileMode::Markdown,
        }
    }

    #[test]
    fn get_copy_text_no_selection_returns_current_line() {
        let mut app = make_app();
        app.textarea = TextArea::new(vec!["hello world".to_string(), "second line".to_string()]);
        // Cursor at (0, 0), no selection — should return the first line.
        assert_eq!(get_copy_text(&app), "hello world");
    }

    #[test]
    fn get_copy_text_no_selection_second_line() {
        let mut app = make_app();
        app.textarea = TextArea::new(vec!["first".to_string(), "second".to_string()]);
        // Move cursor to the second line.
        app.textarea.move_cursor(CursorMove::Down);
        assert_eq!(get_copy_text(&app), "second");
    }

    #[test]
    fn get_copy_text_with_selection_returns_selection() {
        let mut app = make_app();
        app.textarea = TextArea::new(vec!["hello world".to_string()]);
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(CursorMove::Forward);
        }
        // Selection covers "hello".
        assert_eq!(get_copy_text(&app), "hello");
    }

    #[test]
    fn get_copy_text_empty_buffer_returns_empty_string() {
        let app = make_app();
        // Default TextArea has one empty line; no selection → "".
        assert_eq!(get_copy_text(&app), "");
    }
}
