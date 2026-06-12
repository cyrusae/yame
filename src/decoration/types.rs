use std::collections::HashMap;

use ratatui::style::{Color, Style};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A styled span within a single logical line, using char indices (not byte indices).
#[derive(Debug, Clone, Default)]
pub struct StyledSpan {
    /// Start char index within the line (inclusive).
    pub char_start: usize,
    /// End char index within the line (exclusive).
    pub char_end: usize,
    pub style: Style,
    /// True for blockquote lines — kept for test compatibility; continuation
    /// indent uses `continuation_indent` instead.
    pub is_blockquote: bool,
    /// When non-zero, continuation visual rows (wrap_idx > 0) of the logical
    /// line are indented by this many terminal columns.  Used by blockquotes
    /// (indent 2, aligning with text after `> `) and list items (indent =
    /// bullet width + 1 space, aligning with item text).
    pub continuation_indent: u8,
    /// When non-zero, the **first** visual row of the logical line is also
    /// indented by this many terminal columns.  Distinct from
    /// `continuation_indent` (which only applies to wrap_idx > 0) so that list
    /// bullets — which occupy the first row — are not displaced.  Used by
    /// frontmatter content lines to visually separate them from prose.
    pub row_indent: u8,
    /// When set, renderer expands this span's background to fill the full column width.
    pub full_line_bg: Option<Color>,
    /// When set, renderer draws a full-width underline in this color after the row.
    /// Used for H1–H3 bottom borders.
    pub border_bottom: Option<Color>,
    /// When true, renderer replaces line content with a `─` rule pattern.
    pub is_rule: bool,
}

/// Maps logical line index → list of styled spans on that line.
pub type DecorationMap = HashMap<usize, Vec<StyledSpan>>;
