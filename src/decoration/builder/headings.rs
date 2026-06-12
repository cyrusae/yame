use pulldown_cmark::HeadingLevel;
use ratatui::style::{Color, Modifier, Style};

use crate::config::blend_colors;
use crate::decoration::spans::{byte_to_line_char, line_char_len, push_span};
use crate::decoration::types::StyledSpan;
use crate::decoration::DecorationMap;

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- a. Headings ----
// ---------------------------------------------------------------------------

pub(super) fn on_start(s: &mut BuildState, level: HeadingLevel, range: std::ops::Range<usize>) {
    let heading_color = match level {
        HeadingLevel::H1 => s.theme.headings.h1,
        HeadingLevel::H2 => s.theme.headings.h2,
        HeadingLevel::H3 => s.theme.headings.h3,
        HeadingLevel::H4 => s.theme.headings.h4,
        HeadingLevel::H5 => s.theme.headings.h5,
        HeadingLevel::H6 => s.theme.headings.h6,
    };
    let bold = matches!(level, HeadingLevel::H1 | HeadingLevel::H2);
    let mut content_style = Style::default().fg(heading_color);
    if bold {
        content_style = content_style.add_modifier(Modifier::BOLD);
    }
    let mut delim_style = Style::default().fg(blend_colors(
        heading_color,
        s.theme.heading_bg,
        s.theme.delimiter_blend,
    ));
    if bold {
        delim_style = delim_style.add_modifier(Modifier::BOLD);
    }
    let border_bottom = matches!(
        level,
        HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3
    )
    .then_some(heading_color);

    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);

    let level_num = match level {
        HeadingLevel::H1 => 1usize,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    };
    let delim_chars = level_num + 1; // e.g. "# " = 2, "## " = 3

    let line_len = line_char_len(&s.line_starts, s.text, start_line);
    let delim_end = (start_char + delim_chars).min(line_len);

    // Delimiter span (`#`s + space).
    push_heading_delimiter_span(
        &mut s.map,
        start_line,
        start_char,
        delim_end,
        delim_style,
        s.theme.heading_bg,
        border_bottom,
    );

    // Content span (heading text after the `# ` prefix).
    if delim_end < line_len {
        push_span(
            &mut s.map,
            start_line,
            StyledSpan {
                char_start: delim_end,
                char_end: line_len,
                style: content_style,
                full_line_bg: Some(s.theme.heading_bg),
                border_bottom,
                ..Default::default()
            },
        );
    }
}

/// Push the heading `#`-delimiter span, guarding against zero-length spans.
///
/// # Mutation notes
/// - `delim_end > start_char` `> → >=` is `#[mutants::skip]`-protected: the
///   boundary case (`delim_end == start_char`) requires `line_len == 0`, which
///   pulldown-cmark never emits a heading event for.
/// - `char_start: start_char` delete is skipped: for all standard headings
///   `start_char == 0`, equalling `Default::default()`, so the mutation is
///   behaviourally undetectable.
#[mutants::skip]
fn push_heading_delimiter_span(
    map: &mut DecorationMap,
    start_line: usize,
    start_char: usize,
    delim_end: usize,
    style: Style,
    heading_bg: Color,
    border_bottom: Option<Color>,
) {
    if delim_end > start_char {
        push_span(
            map,
            start_line,
            StyledSpan {
                char_start: start_char,
                char_end: delim_end,
                style,
                full_line_bg: Some(heading_bg),
                border_bottom,
                ..Default::default()
            },
        );
    }
}
