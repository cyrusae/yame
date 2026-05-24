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
    /// Set on every keystroke; cleared after decoration pass fires.
    pub last_keystroke: Option<Instant>,
    pub decoration_map: DecorationMap,
    pub word_count: usize,
    pub status: StatusLine,
    pub config_warnings: Vec<String>,
    pub scroll_top: usize,
}

impl App {
    /// Create a new App, loading file content if it exists.
    pub fn new(
        file_path: PathBuf,
        theme: Theme,
        italic_support: bool,
        config_warnings: Vec<String>,
    ) -> io::Result<Self> {
        let textarea = load_file(&file_path)?;
        Ok(Self {
            textarea,
            file_path,
            is_dirty: false,
            saved_content: None,
            theme,
            italic_support,
            last_keystroke: None,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings,
            scroll_top: 0,
        })
    }

    /// Mark that a keystroke occurred, triggering the debounce timer.
    pub fn mark_keystroke(&mut self) {
        self.last_keystroke = Some(Instant::now());
        self.is_dirty = true;
    }

    /// Recompute is_dirty by comparing current lines to saved_content.
    pub fn recompute_dirty(&mut self) {
        self.is_dirty = match &self.saved_content {
            Some(saved) => self.textarea.lines() != saved.as_slice(),
            None => !self.textarea.lines().is_empty(),
        };
    }
}

/// Load a file into a TextArea, or return an empty TextArea for new files.
pub fn load_file(path: &Path) -> io::Result<TextArea<'static>> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(TextArea::new(lines))
    } else {
        Ok(TextArea::default())
    }
}
