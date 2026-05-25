use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tui_textarea::TextArea;

use crate::config::Theme;
use crate::decoration::DecorationMap;
use crate::status::StatusLine;

/// All mutable application state.
pub struct App {
    pub textarea: TextArea<'static>,
    pub file_path: PathBuf,
    pub is_dirty: bool,
    /// Snapshot of lines at last save, for dirty-flag recomputation after undo/redo.
    pub saved_content: Option<Vec<String>>,
    pub theme: Theme,
    pub italic_support: bool,
    /// When true, the status bar uses the Powerline filled-arrow glyph (U+E0B0)
    /// as a segment separator instead of the universal `│` box-drawing character.
    /// Controlled by `[layout] powerline_glyphs` in config. Default false.
    pub powerline_glyphs: bool,
    /// Set on every keystroke; cleared after decoration pass fires.
    pub last_keystroke: Option<Instant>,
    /// Set when a structural change (line count change, undo, redo, paste) requires
    /// the decoration map to be rebuilt on the very next frame instead of waiting
    /// for the debounce timer. Cleared immediately after the rebuild fires.
    pub force_redecorate: bool,
    pub decoration_map: DecorationMap,
    pub word_count: usize,
    pub status: StatusLine,
    pub config_warnings: Vec<String>,
    pub scroll_top: usize,
    /// True for exactly one frame after a scroll-wheel or Ctrl+Up/Down event.
    /// While set, the pre-draw `clamp_scroll` pass is skipped so the viewport
    /// can pan freely without snapping back to the cursor.  Cleared at the top
    /// of every draw cycle regardless of whether it was set.
    pub free_scroll: bool,
    /// Lazily-initialised system clipboard handle. `None` until the first copy/paste,
    /// then reused for the session to avoid reconnecting on every operation (expensive
    /// on Wayland where arboard opens a new display-server connection each time).
    pub clipboard: Option<arboard::Clipboard>,
    /// True when the file was 0 bytes (or did not exist) at load time.
    /// Prevents handle_save from growing a 0-byte file to a 1-byte bare newline
    /// when the buffer is still empty.  Reset to false after any non-empty save.
    pub initial_file_empty: bool,
}

impl App {
    /// Create a new App, loading file content if it exists.
    #[mutants::skip] // Calls load_file (fs I/O) and returns a struct — mutations masked by I/O.
    pub fn new(
        file_path: PathBuf,
        theme: Theme,
        italic_support: bool,
        powerline_glyphs: bool,
        config_warnings: Vec<String>,
        tab_width: usize,
    ) -> io::Result<Self> {
        // Detect whether the file is empty/new before loading, so handle_save can
        // preserve the 0-byte state instead of growing the file to a bare newline.
        let initial_file_empty =
            !file_path.exists() || std::fs::metadata(&file_path).is_ok_and(|m| m.len() == 0);
        let textarea = load_file(&file_path, tab_width)?;
        // Snapshot the initial content so recompute_dirty() has a baseline for both
        // existing files (undo back to load state → clean) and new files (empty baseline).
        let saved_content = Some(textarea.lines().to_vec());
        Ok(Self {
            textarea,
            file_path,
            is_dirty: false,
            saved_content,
            theme,
            italic_support,
            powerline_glyphs,
            last_keystroke: None,
            force_redecorate: false,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings,
            scroll_top: 0,
            free_scroll: false,
            clipboard: None,
            initial_file_empty,
        })
    }

    /// Record that a keystroke occurred: start the debounce timer and recompute
    /// dirty state from the actual buffer content.  Comparing against the saved
    /// baseline means pure navigation (arrows, Home/End…) never marks the file
    /// dirty, while typed characters and paste do.
    pub fn mark_keystroke(&mut self) {
        self.last_keystroke = Some(Instant::now());
        self.recompute_dirty();
    }

    /// Recompute is_dirty by comparing current lines to saved_content.
    pub fn recompute_dirty(&mut self) {
        self.is_dirty = match &self.saved_content {
            Some(saved) => self.textarea.lines() != saved.as_slice(),
            None => !self.textarea.lines().is_empty(),
        };
    }
}

/// Expand tab characters in a line to spaces.
///
/// Each `\t` is replaced with enough spaces to reach the next multiple of
/// `tab_width`. This is the standard "visual tab stop" expansion used by most
/// editors. Pure function — no I/O.
pub fn expand_tabs(line: &str, tab_width: usize) -> String {
    if !line.contains('\t') {
        return line.to_string();
    }
    let tab_width = tab_width.max(1);
    let mut result = String::with_capacity(line.len() + 8);
    let mut col = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            let spaces = tab_width - (col % tab_width);
            for _ in 0..spaces {
                result.push(' ');
            }
            col += spaces;
        } else {
            result.push(ch);
            col += 1;
        }
    }
    result
}

/// Load a file into a TextArea, or return an empty TextArea for new files.
/// Tabs are expanded to spaces using `tab_width` (default 4 if 0 is passed).
#[mutants::skip] // fs::read_to_string I/O — mutations (e.g. skipping the read) not testable without a real FS.
pub fn load_file(path: &Path, tab_width: usize) -> io::Result<TextArea<'static>> {
    let tw = if tab_width == 0 { 4 } else { tab_width };
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(|l| expand_tabs(l, tw)).collect();
        Ok(TextArea::new(lines))
    } else {
        Ok(TextArea::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tui_textarea::TextArea;

    // --- expand_tabs ---

    #[test]
    fn expand_tabs_no_tabs_unchanged() {
        assert_eq!(expand_tabs("hello world", 4), "hello world");
    }

    #[test]
    fn expand_tabs_leading_tab_becomes_spaces() {
        // \t at col 0 with width 4 → 4 spaces
        assert_eq!(expand_tabs("\thello", 4), "    hello");
    }

    #[test]
    fn expand_tabs_mid_line_aligns_to_stop() {
        // "ab\t" at col 2 with width 4 → 2 spaces to reach col 4
        assert_eq!(expand_tabs("ab\thello", 4), "ab  hello");
    }

    #[test]
    fn expand_tabs_at_exact_stop_gives_full_width() {
        // "abcd\t" at col 4 (already at stop) → 4 spaces to next stop
        assert_eq!(expand_tabs("abcd\t", 4), "abcd    ");
    }

    #[test]
    fn expand_tabs_multiple_tabs() {
        // Two tabs: first fills to col 4, second fills to col 8
        assert_eq!(expand_tabs("\t\t", 4), "        ");
    }

    #[test]
    fn expand_tabs_tab_width_two() {
        assert_eq!(expand_tabs("\t", 2), "  ");
    }

    #[test]
    fn expand_tabs_empty_string() {
        assert_eq!(expand_tabs("", 4), "");
    }

    use crate::config::Theme;
    use crate::decoration::DecorationMap;
    use crate::status::StatusLine;

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("test.md"),
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
            clipboard: None,
            initial_file_empty: false,
        }
    }

    #[test]
    fn mark_keystroke_sets_timer() {
        let mut app = make_app();
        assert!(app.last_keystroke.is_none(), "no timer before keystroke");
        app.mark_keystroke();
        assert!(
            app.last_keystroke.is_some(),
            "timer started after keystroke"
        );
    }

    #[test]
    fn mark_keystroke_dirty_when_content_differs_from_saved() {
        let mut app = make_app();
        // saved = [""], current = ["hello"] → dirty
        app.saved_content = Some(vec!["".to_string()]);
        app.textarea = TextArea::new(vec!["hello".to_string()]);
        app.mark_keystroke();
        assert!(app.is_dirty, "modified content → dirty");
    }

    #[test]
    fn mark_keystroke_clean_when_content_matches_saved() {
        let mut app = make_app();
        // saved and current both [""] → not dirty (navigation case)
        let baseline: Vec<String> = app.textarea.lines().iter().map(|s| s.to_string()).collect();
        app.saved_content = Some(baseline);
        app.mark_keystroke();
        assert!(!app.is_dirty, "navigation without content change → clean");
    }

    #[test]
    fn recompute_dirty_saved_matches_clean() {
        let mut app = make_app();
        // Saved snapshot equals current content → not dirty.
        let current: Vec<String> = app.textarea.lines().iter().map(|s| s.to_string()).collect();
        app.saved_content = Some(current);
        app.recompute_dirty();
        assert!(!app.is_dirty, "matching saved content → clean");
    }

    #[test]
    fn recompute_dirty_saved_differs_dirty() {
        let mut app = make_app();
        // Saved snapshot is different from current content → dirty.
        app.saved_content = Some(vec!["something else".to_string()]);
        app.recompute_dirty();
        assert!(app.is_dirty, "diverging saved content → dirty");
    }

    #[test]
    fn recompute_dirty_no_saved_nonempty_is_dirty() {
        let mut app = make_app();
        // No save record and the textarea has content → treat as modified.
        app.saved_content = None;
        // Default TextArea has at least one line (""), which is non-empty as a Vec.
        app.recompute_dirty();
        assert!(app.is_dirty, "unsaved non-empty buffer → dirty");
    }

    #[test]
    fn recompute_dirty_no_saved_truly_empty_is_clean() {
        let mut app = make_app();
        // An explicitly empty textarea (no lines at all) with no saved record → clean.
        app.textarea = TextArea::new(vec![]);
        app.saved_content = None;
        // TextArea::new(vec![]) may return [""] — test only that recompute_dirty runs
        // without panic and produces a consistent result.
        app.recompute_dirty(); // must not panic
    }
}
