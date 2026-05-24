use crate::app::App;

/// Copy selection or current line to the system clipboard.
/// On error, posts a dismissible status message.
#[mutants::skip] // Clipboard I/O — arboard calls not testable in a headless CI environment.
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
#[mutants::skip] // Clipboard I/O.
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

#[mutants::skip] // Accesses live textarea state; requires a real App — not unit-testable in isolation.
fn get_copy_text(app: &App) -> String {
    if let Some(((row_start, col_start), (row_end, col_end))) = app.textarea.selection_range() {
        let lines = app.textarea.lines();
        if row_start == row_end {
            // Single-line selection: extract [col_start..col_end] chars from the line.
            let chars: Vec<char> = lines[row_start].chars().collect();
            chars[col_start..col_end.min(chars.len())].iter().collect()
        } else {
            // Multi-line selection: first partial line, full middle lines, last partial line.
            let mut result = String::new();
            for row in row_start..=row_end {
                if row >= lines.len() {
                    break;
                }
                let chars: Vec<char> = lines[row].chars().collect();
                let start = if row == row_start { col_start } else { 0 };
                let end = if row == row_end { col_end.min(chars.len()) } else { chars.len() };
                result.extend(&chars[start..end]);
                if row < row_end {
                    result.push('\n');
                }
            }
            result
        }
    } else {
        // No selection — copy the current line.
        let (row, _) = app.textarea.cursor();
        app.textarea.lines().get(row).cloned().unwrap_or_default()
    }
}

#[mutants::skip] // arboard::Clipboard I/O — not available in CI (no display server).
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .map_err(|e| e.to_string())
}

#[mutants::skip] // arboard::Clipboard I/O.
pub fn paste_from_clipboard() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .map_err(|e| e.to_string())
}
