use std::collections::HashMap;

use ratatui::style::{Color, Style};

/// A styled span within a single logical line, using char indices (not byte indices).
#[derive(Debug, Clone)]
pub struct StyledSpan {
    /// Start char index within the line (inclusive).
    pub char_start: usize,
    /// End char index within the line (exclusive).
    pub char_end: usize,
    pub style: Style,
    /// True for blockquote lines — renderer indents continuation visual rows.
    pub is_blockquote: bool,
    /// When set, renderer expands this span's background to fill the full column width.
    pub full_line_bg: Option<Color>,
}

/// Maps logical line index → list of styled spans on that line.
pub type DecorationMap = HashMap<usize, Vec<StyledSpan>>;
