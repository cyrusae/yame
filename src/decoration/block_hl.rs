use ratatui::style::Style;

use super::types::{DecorationMap, StyledSpan};

// ---------------------------------------------------------------------------
// block_highlights_to_decoration_map
// ---------------------------------------------------------------------------

/// Convert `BlockHighlights` (syntect per-line spans) into a `DecorationMap`.
///
/// Used when the editor is in `FileMode::PlainHighlight` mode to apply
/// whole-file syntax colouring without running the markdown decoration pass.
/// Each `HlSpan` becomes a `StyledSpan` with a plain fg-colour style and no
/// markdown-specific fields (`is_blockquote`, `full_line_bg`, etc. are all
/// left at their zero/false defaults).
///
/// `line_offset` is added to every line index so the result aligns with
/// `DecorationMap` line numbers.  Pass `0` for a whole-file conversion.
pub fn block_highlights_to_decoration_map(
    hl: &crate::highlighting::BlockHighlights,
    line_offset: usize,
) -> DecorationMap {
    let mut map: DecorationMap = std::collections::HashMap::new();
    for (line_idx, line_spans) in hl.iter().enumerate() {
        let log_line = line_offset + line_idx;
        for hs in line_spans {
            let span = StyledSpan {
                char_start: hs.char_start,
                char_end: hs.char_end,
                style: Style::default().fg(hs.fg),
                is_blockquote: false,
                continuation_indent: 0,
                row_indent: 0,
                full_line_bg: None,
                border_bottom: None,
                is_rule: false,
                line_default_style: None,
            };
            map.entry(log_line).or_default().push(span);
        }
    }
    map
}
