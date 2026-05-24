use crate::app::App;

/// Copy selection or current line to the system clipboard.
/// On error, posts a dismissible status message.
pub fn handle_copy(app: &mut App) {
    let text = get_copy_text(app);
    match copy_to_clipboard(&text) {
        Ok(()) => {}
        Err(e) => {
            app.status
                .set_dismissible(format!("⚠ Clipboard unavailable: {e}"));
        }
    }
}

/// Paste from the system clipboard into the buffer.
pub fn handle_paste(app: &mut App) {
    match paste_from_clipboard() {
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

fn get_copy_text(app: &App) -> String {
    // TODO: respect selection range when tui-textarea exposes selection_range() in v0.7
    // For now, copy the current line.
    let (row, _) = app.textarea.cursor();
    app.textarea.lines().get(row).cloned().unwrap_or_default()
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .map_err(|e| e.to_string())
}

pub fn paste_from_clipboard() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .map_err(|e| e.to_string())
}
