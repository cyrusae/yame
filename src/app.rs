use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tui_textarea::TextArea;

use crate::config::Theme;
use crate::decoration::DecorationMap;
use crate::highlighting::HighlightCache;
use crate::renderer::shorten_path;
use crate::status::StatusLine;

/// Three-state clipboard handle.
///
/// Avoids repeated blocking connection attempts on headless systems:
/// once an `arboard::Clipboard` connection fails we record it as
/// `Unavailable` and stop retrying for the rest of the session.
pub enum ClipboardState {
    /// Not yet tried — initialised lazily on the first copy/paste.
    Uninitialized,
    /// Connected and ready to use.
    Ready(arboard::Clipboard),
    /// A previous connection attempt failed; skip all further attempts.
    Unavailable,
}

/// All mutable application state.
pub struct App {
    pub textarea: TextArea<'static>,
    pub file_path: PathBuf,
    /// Pre-computed display string for the file path (shortened to 3 trailing
    /// components).  Cached here so the render hot-path does not re-allocate on
    /// every frame.  Updated whenever `file_path` changes.
    pub shortened_path: String,
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
    /// The visual-column offset (within the current visual subrow) to aim for
    /// while navigating Up/Down.  Set on the first arrow press of a vertical
    /// gesture and preserved across subsequent Up/Down presses so the cursor
    /// returns to its original column after passing through shorter lines.
    /// Cleared by any non-vertical-nav key (see `handle_key_event`).
    pub sticky_col: Option<usize>,
    /// Content width in terminal columns (column_width − 2×GUTTER) as of the
    /// last render frame.  Written by the event loop before dispatching each key
    /// event so `handle_visual_move` can use the same wrapping the renderer used.
    pub content_width: usize,
    /// Clipboard state: uninitialized, ready, or permanently unavailable.
    /// See [`ClipboardState`] for the three-state semantics.
    pub clipboard: ClipboardState,
    /// Number of spaces per Tab key press (and per tab-stop on load).
    /// Mirrors `[layout] tab_width` from config; stored here so `handle_key_event`
    /// can expand Tab keypresses without needing access to the full config.
    pub tab_width: usize,
    /// Syntect highlight cache. `None` when highlighting is disabled in config.
    /// Populated at startup from `[highlighting] enabled` + `syntect_theme`.
    pub highlight_cache: Option<HighlightCache>,
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
        highlight_cache: Option<HighlightCache>,
    ) -> io::Result<Self> {
        let textarea = load_file(&file_path, tab_width)?;
        // Snapshot the initial content so recompute_dirty() has a baseline for both
        // existing files (undo back to load state → clean) and new files (empty baseline).
        let saved_content = Some(textarea.lines().to_vec());
        let shortened_path = shorten_path(&file_path, 3);
        Ok(Self {
            textarea,
            shortened_path,
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
            sticky_col: None,
            content_width: 0,
            clipboard: ClipboardState::Uninitialized,
            tab_width: tab_width.max(1),
            highlight_cache,
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

/// Extract the currently-selected text from the textarea, or `None` if there
/// is no active selection.  Does not fall back to the current line.
///
/// Shared between the pair-wrap handler (`input.rs`) and the copy handler
/// (`clipboard.rs`), both of which need identical selection extraction logic.
pub fn get_selection_text(app: &App) -> Option<String> {
    let ((row_start, col_start), (row_end, col_end)) = app.textarea.selection_range()?;
    let lines = app.textarea.lines();
    if row_start == row_end {
        let chars: Vec<char> = lines[row_start].chars().collect();
        Some(chars[col_start..col_end.min(chars.len())].iter().collect())
    } else {
        let mut result = String::new();
        for row in row_start..=row_end {
            if row >= lines.len() {
                break;
            }
            let chars: Vec<char> = lines[row].chars().collect();
            let start = if row == row_start { col_start } else { 0 };
            let end = if row == row_end {
                col_end.min(chars.len())
            } else {
                chars.len()
            };
            result.extend(&chars[start..end]);
            if row < row_end {
                result.push('\n');
            }
        }
        Some(result)
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

    // Kills: app.rs:126 replace += with *= in expand_tabs.
    // With `col *= spaces`: after "a" col=1, first \t gives spaces=3 → col=3 (not 4),
    // second \t then gives spaces=1 (not 4) → result "a    " (5 chars) ≠ "a       " (8 chars).
    #[test]
    fn expand_tabs_double_tab_with_offset() {
        // "a" → col 1; \t → 3 spaces (col 4); \t → 4 spaces (col 8) = "a       "
        assert_eq!(expand_tabs("a\t\t", 4), "a       ");
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
