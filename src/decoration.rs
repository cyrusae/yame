use std::collections::HashMap;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};

use crate::config::{Theme, blend_colors};

// ---------------------------------------------------------------------------
// Types (Step 6.1)
// ---------------------------------------------------------------------------

/// A styled span within a single logical line, using char indices (not byte indices).
#[derive(Debug, Clone, Default)]
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
    /// When set, renderer draws a full-width underline in this color after the row.
    /// Used for H1–H3 bottom borders.
    pub border_bottom: Option<Color>,
    /// When true, renderer replaces line content with a `─` rule pattern.
    pub is_rule: bool,
}

/// Maps logical line index → list of styled spans on that line.
pub type DecorationMap = HashMap<usize, Vec<StyledSpan>>;

// ---------------------------------------------------------------------------
// Byte-to-line/char mapping (Step 6.2)
// ---------------------------------------------------------------------------

/// Returns byte offsets of the start of each line (line 0 = offset 0).
pub fn line_start_bytes(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Convert a byte offset to `(line_index, char_offset_within_line)`.
/// Safe for non-char-boundary inputs — rounds down to the nearest boundary.
pub fn byte_to_line_char(line_starts: &[usize], text: &str, byte: usize) -> (usize, usize) {
    // Clamp and snap to a valid char boundary so multi-byte chars never panic.
    let byte = text.floor_char_boundary(byte.min(text.len()));
    let line = line_starts
        .partition_point(|&s| s <= byte)
        .saturating_sub(1);
    let line_start_byte = line_starts[line];
    let char_col = text[line_start_byte..byte].chars().count();
    (line, char_col)
}

/// Number of displayable chars on a line (excludes the trailing `\n`).
fn line_char_len(line_starts: &[usize], text: &str, line_idx: usize) -> usize {
    let ls = line_starts[line_idx];
    let le = if line_idx + 1 < line_starts.len() {
        line_starts[line_idx + 1].saturating_sub(1) // trim the \n
    } else {
        text.len()
    };
    text[ls..le].chars().count()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn push_span(map: &mut DecorationMap, line: usize, span: StyledSpan) {
    map.entry(line).or_default().push(span);
}

fn make_span(char_start: usize, char_end: usize, style: Style) -> StyledSpan {
    StyledSpan {
        char_start,
        char_end,
        style,
        ..Default::default()
    }
}

struct SpanParams {
    style: Style,
    full_line_bg: Option<Color>,
    is_blockquote: bool,
}

/// Add a span that covers a byte range; handles multi-line ranges by splitting per line.
fn add_byte_range_span(
    map: &mut DecorationMap,
    line_starts: &[usize],
    text: &str,
    byte_start: usize,
    byte_end: usize,
    params: SpanParams,
) {
    if byte_start >= byte_end {
        return;
    }
    let (start_line, start_char) = byte_to_line_char(line_starts, text, byte_start);
    // byte_end is exclusive; point to the last byte for end calculation
    let end_byte = byte_end.saturating_sub(1).max(byte_start);
    let (end_line, end_char_inclusive) = byte_to_line_char(line_starts, text, end_byte);

    for line in start_line..=end_line {
        let c_start = if line == start_line { start_char } else { 0 };
        let c_end = if line == end_line {
            end_char_inclusive + 1 // make exclusive
        } else {
            line_char_len(line_starts, text, line)
        };
        let c_end = c_end.max(c_start + 1); // always at least 1 char wide
        push_span(
            map,
            line,
            StyledSpan {
                char_start: c_start,
                char_end: c_end,
                style: params.style,
                is_blockquote: params.is_blockquote,
                full_line_bg: params.full_line_bg,
                ..Default::default()
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Step 6.3 — build_decoration_map
// ---------------------------------------------------------------------------

/// Build the full decoration map from `text`.
///
/// Pure function — no terminal or UI side effects. This is the v1.5 migration seam:
/// when moving to a background thread, only the call site changes.
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
    cursor_line: usize,
) -> DecorationMap {
    let line_starts = line_start_bytes(text);
    let mut map: DecorationMap = HashMap::new();

    let options =
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;

    let parser = Parser::new_ext(text, options).into_offset_iter();

    // State tracking
    let mut in_ordered_list = false;

    for (event, range) in parser {
        match event {
            // ---- a. Headings ----
            Event::Start(Tag::Heading { level, .. }) => {
                let heading_color = match level {
                    HeadingLevel::H1 => theme.headings.h1,
                    HeadingLevel::H2 => theme.headings.h2,
                    HeadingLevel::H3 => theme.headings.h3,
                    HeadingLevel::H4 => theme.headings.h4,
                    HeadingLevel::H5 => theme.headings.h5,
                    HeadingLevel::H6 => theme.headings.h6,
                };
                let bold = matches!(level, HeadingLevel::H1 | HeadingLevel::H2);
                let mut content_style = Style::default().fg(heading_color);
                if bold {
                    content_style = content_style.add_modifier(Modifier::BOLD);
                }
                // `# ` / `## ` / `### ` etc. blend toward muted like other delimiters.
                let delim_style = Style::default().fg(blend_colors(
                    heading_color,
                    theme.muted,
                    theme.delimiter_blend,
                ));
                // Bottom border for H1–H3 (thin underline in heading color).
                let border_bottom = matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3
                )
                .then_some(heading_color);

                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

                // Number of `#` characters + the trailing space.
                let level_num = match level {
                    HeadingLevel::H1 => 1usize,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let delim_chars = level_num + 1; // e.g. "# " = 2, "## " = 3

                // Delimiter span (the `#`s + space).
                let delim_end = (start_char + delim_chars).min(end_char_excl);
                if delim_end > start_char {
                    push_span(
                        &mut map,
                        start_line,
                        StyledSpan {
                            char_start: start_char,
                            char_end: delim_end,
                            style: delim_style,
                            full_line_bg: Some(theme.heading_bg),
                            border_bottom,
                            ..Default::default()
                        },
                    );
                }

                // Content span (the heading text itself), possibly multi-line.
                if delim_end < end_char_excl || end_line > start_line {
                    for line in start_line..=end_line {
                        let c_start = if line == start_line { delim_end } else { 0 };
                        let c_end = if line == end_line {
                            end_char_excl
                        } else {
                            line_char_len(&line_starts, text, line)
                        };
                        if c_start < c_end {
                            push_span(
                                &mut map,
                                line,
                                StyledSpan {
                                    char_start: c_start,
                                    char_end: c_end,
                                    style: content_style,
                                    full_line_bg: Some(theme.heading_bg),
                                    border_bottom,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }

            // ---- b. Bold ----
            Event::Start(Tag::Strong) => {
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

                if start_line == end_line {
                    let span_len = end_char_excl.saturating_sub(start_char);
                    if span_len >= 4 {
                        let delim_style = Style::default()
                            .fg(blend_colors(theme.text, theme.muted, theme.delimiter_blend))
                            .add_modifier(Modifier::BOLD);
                        let content_style = Style::default()
                            .fg(theme.bold_color)
                            .add_modifier(Modifier::BOLD);

                        // opening **
                        push_span(
                            &mut map,
                            start_line,
                            make_span(start_char, start_char + 2, delim_style),
                        );
                        // content
                        if start_char + 2 < end_char_excl.saturating_sub(2) {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(start_char + 2, end_char_excl - 2, content_style),
                            );
                        }
                        // closing **
                        push_span(
                            &mut map,
                            end_line,
                            make_span(end_char_excl - 2, end_char_excl, delim_style),
                        );
                    }
                }
            }

            // ---- c. Italic ----
            Event::Start(Tag::Emphasis) => {
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

                if start_line == end_line {
                    let span_len = end_char_excl.saturating_sub(start_char);
                    if span_len >= 2 {
                        let delim_style = Style::default().fg(blend_colors(
                            theme.italic_color,
                            theme.muted,
                            theme.delimiter_blend,
                        ));
                        let mut content_style = Style::default().fg(theme.italic_color);
                        if italic_support {
                            content_style = content_style.add_modifier(Modifier::ITALIC);
                        }

                        // opening *
                        push_span(
                            &mut map,
                            start_line,
                            make_span(start_char, start_char + 1, delim_style),
                        );
                        // content
                        if start_char + 1 < end_char_excl.saturating_sub(1) {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(start_char + 1, end_char_excl - 1, content_style),
                            );
                        }
                        // closing *
                        push_span(
                            &mut map,
                            end_line,
                            make_span(end_char_excl - 1, end_char_excl, delim_style),
                        );
                    }
                }
            }

            // ---- d. Inline code ----
            Event::Code(_) => {
                let style = Style::default().fg(theme.code_color).bg(theme.code_bg);
                add_byte_range_span(
                    &mut map,
                    &line_starts,
                    text,
                    range.start,
                    range.end,
                    SpanParams {
                        style,
                        full_line_bg: None,
                        is_blockquote: false,
                    },
                );
            }

            // ---- e. Fenced code blocks ----
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                // TODO(v1.5): pass block content and language tag to syntect here
                let style = Style::default().bg(theme.fenced_bg);
                add_byte_range_span(
                    &mut map,
                    &line_starts,
                    text,
                    range.start,
                    range.end,
                    SpanParams {
                        style,
                        full_line_bg: Some(theme.fenced_bg),
                        is_blockquote: false,
                    },
                );
            }

            // ---- f. Blockquotes ----
            Event::Start(Tag::BlockQuote(_)) => {
                let (start_line, _) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, _) = byte_to_line_char(
                    &line_starts,
                    text,
                    range.end.saturating_sub(1).max(range.start),
                );

                let indicator_style = Style::default().fg(theme.muted);
                let content_style = Style::default().fg(theme.blockquote_color);

                for line in start_line..=end_line {
                    let line_len = line_char_len(&line_starts, text, line);
                    if line_len == 0 {
                        continue;
                    }
                    // ▌ indicator at char 0 (covers the `>` char visually)
                    push_span(
                        &mut map,
                        line,
                        StyledSpan {
                            char_start: 0,
                            char_end: 1,
                            style: indicator_style,
                            is_blockquote: true,
                            ..Default::default()
                        },
                    );
                    // rest of line content
                    if line_len > 1 {
                        push_span(
                            &mut map,
                            line,
                            StyledSpan {
                                char_start: 1,
                                char_end: line_len,
                                style: content_style,
                                is_blockquote: true,
                                ..Default::default()
                            },
                        );
                    }
                }
            }

            // ---- g. Links ----
            Event::Start(Tag::Link { .. }) => {
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

                // Only handle single-line links in v1
                if start_line == end_line {
                    let link_text_slice = &text[range.start..range.end];
                    let link_chars: Vec<char> = link_text_slice.chars().collect();

                    if let Some(split_idx) = link_split_char_idx(&link_chars) {
                        let delim_style = Style::default().fg(blend_colors(
                            theme.link_text,
                            theme.muted,
                            theme.delimiter_blend,
                        ));
                        let text_style = Style::default()
                            .fg(theme.link_text)
                            .add_modifier(Modifier::UNDERLINED);
                        let mut url_style = Style::default().fg(theme.link_url);
                        if italic_support {
                            url_style = url_style.add_modifier(Modifier::ITALIC);
                        }

                        // [ at start_char
                        push_span(
                            &mut map,
                            start_line,
                            make_span(start_char, start_char + 1, delim_style),
                        );
                        // text content
                        if split_idx > 1 {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(start_char + 1, start_char + split_idx, text_style),
                            );
                        }
                        // ] and ( around split
                        push_span(
                            &mut map,
                            start_line,
                            make_span(
                                start_char + split_idx,
                                start_char + split_idx + 2,
                                delim_style,
                            ),
                        );
                        // url content
                        let url_start = start_char + split_idx + 2;
                        let url_end = end_char_excl.saturating_sub(1);
                        if url_end > url_start {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(url_start, url_end, url_style),
                            );
                        }
                        // closing )
                        if end_char_excl > 0 {
                            push_span(
                                &mut map,
                                end_line,
                                make_span(end_char_excl - 1, end_char_excl, delim_style),
                            );
                        }
                    }
                }
            }

            // ---- h. List items ----
            Event::Start(Tag::List(kind)) => {
                in_ordered_list = kind.is_some();
            }
            Event::End(TagEnd::List(_)) => {
                in_ordered_list = false;
            }
            Event::Start(Tag::Item) => {
                let (item_line, item_char) = byte_to_line_char(&line_starts, text, range.start);

                let bullet_style = Style::default().fg(theme.accent);
                // Style the bullet/number: 1 char for unordered, 2 for ordered (e.g. `1.`)
                let bullet_end = if in_ordered_list {
                    // scan forward to find the `.` or `)` marker
                    let line_bytes_start = line_starts[item_line];
                    let scan_start = range.start.saturating_sub(line_bytes_start); // offset within line
                    let line_text = &text[line_starts[item_line]..];
                    line_text[scan_start..]
                        .find(['.', ')'])
                        .map(|i| {
                            item_char + count_chars_in(&line_text[scan_start..scan_start + i + 1])
                        })
                        .unwrap_or(item_char + 2)
                } else {
                    item_char + 1
                };
                push_span(
                    &mut map,
                    item_line,
                    make_span(item_char, bullet_end, bullet_style),
                );
            }

            // ---- i. Todo items ----
            Event::TaskListMarker(checked) => {
                let (marker_line, marker_char) = byte_to_line_char(&line_starts, text, range.start);

                if checked {
                    // Apply todo_done colour to the entire item line.
                    // Strikethrough is intentionally absent: real ~~strikethrough~~ syntax
                    // exists in Markdown and should remain visually distinct.
                    let line_len = line_char_len(&line_starts, text, marker_line);
                    let style = Style::default().fg(theme.todo_done);
                    push_span(&mut map, marker_line, make_span(0, line_len, style));
                } else {
                    // Style [ and ] in accent, leave space between as normal
                    let accent = Style::default().fg(theme.accent);
                    // `[ ]` is 3 chars: [, space, ]
                    push_span(
                        &mut map,
                        marker_line,
                        make_span(marker_char, marker_char + 1, accent),
                    );
                    push_span(
                        &mut map,
                        marker_line,
                        make_span(marker_char + 2, marker_char + 3, accent),
                    );
                }
            }

            // ---- j. Tables ----
            Event::Start(Tag::Table(_)) => {
                // Apply muted to all `|` characters in the table range
                let (start_line, _) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, _) = byte_to_line_char(
                    &line_starts,
                    text,
                    range.end.saturating_sub(1).max(range.start),
                );
                let pipe_style = Style::default().fg(theme.muted);

                for line in start_line..=end_line {
                    let ls = line_starts[line];
                    let le = if line + 1 < line_starts.len() {
                        line_starts[line + 1].saturating_sub(1)
                    } else {
                        text.len()
                    };
                    let line_text = &text[ls..le];
                    for (char_idx, c) in line_text.chars().enumerate() {
                        if c == '|' {
                            push_span(
                                &mut map,
                                line,
                                make_span(char_idx, char_idx + 1, pipe_style),
                            );
                        }
                    }
                }
            }
            Event::Start(Tag::TableHead) => {
                // Header cells: bold + accent_color
                let style = Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD);
                add_byte_range_span(
                    &mut map,
                    &line_starts,
                    text,
                    range.start,
                    range.end,
                    SpanParams {
                        style,
                        full_line_bg: None,
                        is_blockquote: false,
                    },
                );
            }

            // ---- k. Strikethrough ----
            Event::Start(Tag::Strikethrough) => {
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

                if start_line == end_line {
                    let span_len = end_char_excl.saturating_sub(start_char);
                    if span_len >= 4 {
                        let delim_style = Style::default().fg(blend_colors(
                            theme.strikethrough_color,
                            theme.muted,
                            theme.delimiter_blend,
                        ));
                        let content_style = Style::default()
                            .fg(theme.strikethrough_color)
                            .add_modifier(Modifier::CROSSED_OUT);

                        // opening ~~
                        push_span(
                            &mut map,
                            start_line,
                            make_span(start_char, start_char + 2, delim_style),
                        );
                        // content
                        if start_char + 2 < end_char_excl.saturating_sub(2) {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(start_char + 2, end_char_excl - 2, content_style),
                            );
                        }
                        // closing ~~
                        push_span(
                            &mut map,
                            end_line,
                            make_span(end_char_excl - 2, end_char_excl, delim_style),
                        );
                    }
                }
            }

            // ---- l. Horizontal rule ----
            Event::Rule => {
                let (rule_line, _) = byte_to_line_char(&line_starts, text, range.start);
                let line_len = line_char_len(&line_starts, text, rule_line).max(1);
                push_span(
                    &mut map,
                    rule_line,
                    StyledSpan {
                        char_start: 0,
                        char_end: line_len,
                        style: Style::default().fg(theme.rule_color),
                        is_rule: true,
                        ..Default::default()
                    },
                );
            }

            _ => {}
        }
    }

    // Step 6.5 — Remove cursor line entries so raw Markdown is visible while editing
    map.remove(&cursor_line);

    map
}

// ---------------------------------------------------------------------------
// Step 6.6 — Word count
// ---------------------------------------------------------------------------

/// Count words in Markdown text, excluding syntax characters.
pub fn count_words(text: &str) -> usize {
    Parser::new(text)
        .filter_map(|e| match e {
            Event::Text(s) | Event::Code(s) => Some(s.split_whitespace().count()),
            _ => None,
        })
        .sum()
}

// ---------------------------------------------------------------------------
// Private link helpers
// ---------------------------------------------------------------------------

/// Find the `](` split point in a `[text](url)` char slice.
/// Returns the char index of `]`.
fn link_split_char_idx(chars: &[char]) -> Option<usize> {
    let mut bracket_depth = 0usize;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => bracket_depth += 1,
            ']' if i + 1 < chars.len() && chars[i + 1] == '(' => {
                if bracket_depth <= 1 {
                    return Some(i);
                }
                bracket_depth = bracket_depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Count chars in a `&str` slice (for ordered list marker scanning).
fn count_chars_in(s: &str) -> usize {
    s.chars().count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_theme() -> Theme {
        Theme::default_theme()
    }

    // ---- Step 6.2 tests ----

    #[test]
    fn byte_mapping_single_line() {
        let text = "hello";
        let starts = line_start_bytes(text);
        assert_eq!(byte_to_line_char(&starts, text, 3), (0, 3));
    }

    #[test]
    fn byte_mapping_second_line() {
        let text = "hi\nworld";
        let starts = line_start_bytes(text);
        // byte 4 = 'o' in "world" (h=3, i=4, \n=5, w=3, o=4 → byte 4 is 'o'? let me recount)
        // "hi\nworld" — bytes: h=0, i=1, \n=2, w=3, o=4, r=5, l=6, d=7
        // line_starts = [0, 3]
        // byte 4 = 'o' → line 1, char 1
        assert_eq!(byte_to_line_char(&starts, text, 4), (1, 1));
    }

    #[test]
    fn byte_mapping_multibyte() {
        let text = "café\nok";
        let starts = line_start_bytes(text);
        // "café" = c(1) a(1) f(1) é(2) \n(1) = 6 bytes total for first line+newline
        // line_starts = [0, 6]
        // byte 6 = start of "ok" → line 1, char 0
        assert_eq!(byte_to_line_char(&starts, text, 6), (1, 0));
    }

    #[test]
    fn line_start_bytes_basic() {
        let starts = line_start_bytes("a\nb\nc");
        assert_eq!(starts, vec![0, 2, 4]);
    }

    #[test]
    fn line_start_bytes_trailing_newline() {
        let starts = line_start_bytes("a\n");
        assert_eq!(starts, vec![0, 2]);
    }

    // ---- Step 6.4 tests — one per element type ----

    // a. Headings
    #[test]
    fn heading_h1_has_full_line_bg() {
        let text = "# Hello World";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.full_line_bg.is_some()),
            "H1 must have full_line_bg"
        );
    }

    #[test]
    fn heading_h1_is_bold() {
        let text = "# Heading";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "H1 must be bold"
        );
    }

    #[test]
    fn heading_h3_not_bold() {
        let text = "### Heading Three";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // H3+ should not have bold modifier
        assert!(
            !spans
                .iter()
                .all(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "H3 should not be bold"
        );
    }

    // b. Bold
    #[test]
    fn bold_span_exists() {
        let text = "Text **bold content** here";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "bold span must exist"
        );
    }

    #[test]
    fn bold_delimiter_has_blended_color() {
        let text = "**hi**";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // There should be 3 spans: delim, content, delim
        assert!(spans.len() >= 2, "bold should produce multiple spans");
    }

    // c. Italic
    #[test]
    fn italic_span_with_support() {
        let text = "*italic text*";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC)),
            "italic span must exist when italic_support=true"
        );
    }

    #[test]
    fn italic_span_without_support_no_modifier() {
        let text = "*italic text*";
        let map = build_decoration_map(text, &make_theme(), false, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC)),
            "should not apply ITALIC modifier when italic_support=false"
        );
    }

    // d. Inline code
    #[test]
    fn inline_code_has_code_bg() {
        let text = "text `code` text";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.bg.is_some()),
            "inline code span must have a background color"
        );
    }

    // e. Fenced code blocks
    #[test]
    fn fenced_code_block_has_bg_on_all_lines() {
        let text = "before\n```\ncode line 1\ncode line 2\n```\nafter";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        // The fenced block spans lines 1-4 (the ``` delimiters + content)
        // At least lines 2 and 3 (code content) should have fenced_bg
        let has_fenced = map
            .iter()
            .any(|(_, spans)| spans.iter().any(|s| s.full_line_bg.is_some()));
        assert!(has_fenced, "fenced code block must have full_line_bg spans");
    }

    // f. Blockquotes
    #[test]
    fn blockquote_has_indicator_span() {
        let text = "> quoted text";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.is_blockquote && s.char_start == 0 && s.char_end == 1),
            "blockquote must have indicator span at char 0"
        );
    }

    #[test]
    fn blockquote_sets_is_blockquote_flag() {
        let text = "> A blockquote";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.is_blockquote),
            "blockquote spans must have is_blockquote=true"
        );
    }

    // g. Links
    #[test]
    fn link_text_has_underline() {
        let text = "[example](https://example.com)";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::UNDERLINED)),
            "link text must have underline"
        );
    }

    #[test]
    fn link_split_at_bracket_paren() {
        let chars: Vec<char> = "[text](url)".chars().collect();
        let idx = link_split_char_idx(&chars);
        assert_eq!(idx, Some(5)); // `]` is at index 5
    }

    // h. Lists
    #[test]
    fn list_bullet_has_accent_color() {
        let text = "- item one\n- item two";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let theme = make_theme();
        // At least one span should have accent color on line 0
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.accent)),
            "bullet must have accent color"
        );
    }

    // i. Todo items
    #[test]
    fn todo_unchecked_bracket_has_accent() {
        let text = "- [ ] todo item";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let theme = make_theme();
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.accent)),
            "unchecked todo brackets must have accent color"
        );
    }

    #[test]
    fn todo_checked_is_muted_no_strikethrough() {
        let text = "- [x] done item";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Must be todo_done colour (defaults to muted).
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.todo_done)),
            "checked todo must use todo_done colour"
        );
        // Must NOT have strikethrough — that is reserved for real ~~syntax~~.
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT)),
            "checked todo must not have CROSSED_OUT"
        );
    }

    // j. Tables
    #[test]
    fn table_pipes_have_muted_color() {
        let text = "| A | B |\n| - | - |\n| 1 | 2 |";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let theme = make_theme();
        let has_pipe_spans = map
            .values()
            .flatten()
            .any(|s| s.style.fg == Some(theme.muted) && s.char_end - s.char_start == 1);
        assert!(has_pipe_spans, "table pipes must have muted color");
    }

    #[test]
    fn table_header_is_bold() {
        let text = "| Head A | Head B |\n| --- | --- |\n| cell | cell |";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let has_bold = map
            .values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(has_bold, "table header must have bold");
    }

    // Step 6.5 — cursor line exclusion
    #[test]
    fn cursor_line_has_no_decoration() {
        let text = "# Heading\nnormal line";
        let map = build_decoration_map(text, &make_theme(), true, 0); // cursor on heading
        assert!(
            map.get(&0).map_or(true, |v| v.is_empty()),
            "cursor line (0) should have no decoration"
        );
        // Non-cursor line unaffected
        // line 1 has no decoration naturally (plain text), so we just check no crash
    }

    // Step 6.6 — word count
    #[test]
    fn word_count_excludes_markdown_syntax() {
        assert_eq!(count_words("**hello**"), 1);
        assert_eq!(count_words("# Title\n\nTwo words."), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn word_count_counts_code_content() {
        // `code` counts as a word
        assert_eq!(count_words("`word`"), 1);
    }

    // Multi-byte safety
    #[test]
    fn heading_with_multibyte_chars() {
        let text = "# Café résumé";
        // Should not panic
        let map = build_decoration_map(text, &make_theme(), true, 99);
        assert!(map.get(&0).is_some());
    }

    #[test]
    fn bold_with_multibyte_chars() {
        let text = "**café**";
        // Should not panic
        let _map = build_decoration_map(text, &make_theme(), true, 99);
    }

    // Full fixture smoke test (subset — full integration test in Phase 11)
    #[test]
    fn fixture_produces_nonempty_map() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true, 9999);
        assert!(!map.is_empty(), "fixture should produce decorations");
    }

    #[test]
    fn fixture_has_heading_bg() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true, 9999);
        // Line 0 is `# Heading One`
        let spans = map.get(&0).expect("line 0 should have heading spans");
        assert!(spans.iter().any(|s| s.full_line_bg.is_some()));
    }

    #[test]
    fn fixture_has_blockquote() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true, 9999);
        assert!(
            map.values().flatten().any(|s| s.is_blockquote),
            "fixture should have blockquote spans"
        );
    }

    #[test]
    fn fixture_word_count_nonzero() {
        let text = include_str!("../tests/fixtures/sample.md");
        assert!(count_words(text) > 100);
    }

    // k. Strikethrough
    #[test]
    fn strikethrough_has_crossed_out_modifier() {
        let text = "normal ~~struck~~ normal";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT)),
            "strikethrough content must have CROSSED_OUT modifier"
        );
    }

    #[test]
    fn strikethrough_delimiters_are_blended() {
        let text = "~~hi~~";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Should produce at least delimiter + content spans
        assert!(
            spans.len() >= 2,
            "strikethrough should produce multiple spans"
        );
    }

    // l. Horizontal rule
    #[test]
    fn horizontal_rule_sets_is_rule_flag() {
        // A `---` line surrounded by blank lines is a thematic break in pulldown-cmark
        let text = "above\n\n---\n\nbelow";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        assert!(
            map.values().flatten().any(|s| s.is_rule),
            "horizontal rule must set is_rule=true on its line"
        );
    }

    // a. Heading delimiter blending
    #[test]
    fn heading_h1_delimiter_is_blended() {
        let text = "# Hello";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Should have both delimiter span (char 0..2) and content span (char 2..)
        let has_delim = spans.iter().any(|s| s.char_start == 0 && s.char_end == 2);
        let has_content = spans.iter().any(|s| s.char_start == 2);
        assert!(has_delim, "H1 should have a delimiter span at 0..2");
        assert!(
            has_content,
            "H1 should have a content span starting at char 2"
        );
    }

    #[test]
    fn heading_h2_delimiter_is_three_chars() {
        let text = "## Title";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        // `## ` = 3 chars: delimiter at 0..3
        let has_delim = spans.iter().any(|s| s.char_start == 0 && s.char_end == 3);
        assert!(has_delim, "H2 should have delimiter span at 0..3");
    }

    #[test]
    fn heading_h1_has_border_bottom() {
        let text = "# Heading One";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.border_bottom.is_some()),
            "H1 must have border_bottom set"
        );
    }

    #[test]
    fn heading_h4_no_border_bottom() {
        let text = "#### Heading Four";
        let map = build_decoration_map(text, &make_theme(), true, 99);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.border_bottom.is_some()),
            "H4+ must not have border_bottom"
        );
    }
}
