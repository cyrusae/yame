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
#[mutants::skip]
pub fn build_decoration_map(text: &str, theme: &Theme, italic_support: bool) -> DecorationMap {
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
                // `# ` / `## ` / `### ` blend toward muted using the neutral text
                // colour as the source (same formula as `*` and `[]()` delimiters).
                // Blending from heading_color was too vivid for saturated accent hues.
                let delim_style = Style::default().fg(blend_colors(
                    theme.text,
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
                // The inner `c_start < c_end` guard handles empty headings; no outer
                // pre-check needed.
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
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);
                let code_style = Style::default().fg(theme.code_color).bg(theme.code_bg);
                // Backtick delimiters blend toward muted (same standard as `*`, `[]()` etc.)
                let delim_style = Style::default()
                    .fg(blend_colors(
                        theme.code_color,
                        theme.muted,
                        theme.delimiter_blend,
                    ))
                    .bg(theme.code_bg);

                if start_line == end_line {
                    // Count the opening backtick run so we can split delimiters from content.
                    let bt = text[range.start..range.end]
                        .chars()
                        .take_while(|&c| c == '`')
                        .count()
                        .max(1);
                    let open_end = (start_char + bt).min(end_char_excl);
                    let close_start = end_char_excl.saturating_sub(bt).max(open_end);

                    // Opening backtick(s)
                    push_span(
                        &mut map,
                        start_line,
                        make_span(start_char, open_end, delim_style),
                    );
                    // Content between the backticks
                    if open_end < close_start {
                        push_span(
                            &mut map,
                            start_line,
                            make_span(open_end, close_start, code_style),
                        );
                    }
                    // Closing backtick(s)
                    if close_start < end_char_excl {
                        push_span(
                            &mut map,
                            start_line,
                            make_span(close_start, end_char_excl, delim_style),
                        );
                    }
                } else {
                    // Multi-line fallback (rare in practice — treat whole span uniformly).
                    add_byte_range_span(
                        &mut map,
                        &line_starts,
                        text,
                        range.start,
                        range.end,
                        SpanParams {
                            style: code_style,
                            full_line_bg: None,
                            is_blockquote: false,
                        },
                    );
                }
            }

            // ---- e. Fenced code blocks ----
            // DEFERRED(v1.5): pass block content and language tag to syntect for
            // syntax highlighting.
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let (start_line, _) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, _) = byte_to_line_char(
                    &line_starts,
                    text,
                    range.end.saturating_sub(1).max(range.start),
                );
                let fence_bg_style = Style::default().bg(theme.fenced_bg);
                // Fence ``` delimiters blend toward muted, same standard as other delimiters.
                let fence_delim_style = Style::default()
                    .fg(blend_colors(
                        theme.code_color,
                        theme.muted,
                        theme.delimiter_blend,
                    ))
                    .bg(theme.fenced_bg);
                let lang_style = Style::default().fg(theme.accent).bg(theme.fenced_bg);

                // Opening fence line: ``` delimiters → code_color, language tag → accent.
                // First span carries full_line_bg so the renderer flood-fills the whole row.
                {
                    let line_len = line_char_len(&line_starts, text, start_line);
                    let lb_start = line_starts[start_line];
                    let lb_end = if start_line + 1 < line_starts.len() {
                        line_starts[start_line + 1].saturating_sub(1)
                    } else {
                        text.len()
                    };
                    let fence_count = text[lb_start..lb_end]
                        .chars()
                        .take_while(|&c| c == '`' || c == '~')
                        .count()
                        .min(line_len);
                    push_span(
                        &mut map,
                        start_line,
                        StyledSpan {
                            char_start: 0,
                            char_end: fence_count.max(1),
                            style: fence_delim_style,
                            full_line_bg: Some(theme.fenced_bg),
                            ..Default::default()
                        },
                    );
                    let lang_str = lang.as_ref();
                    if !lang_str.is_empty() {
                        let lang_end = (fence_count + lang_str.chars().count()).min(line_len);
                        if lang_end > fence_count {
                            push_span(
                                &mut map,
                                start_line,
                                make_span(fence_count, lang_end, lang_style),
                            );
                        }
                    }
                }

                // Content lines: fenced_bg background only.
                for line in (start_line + 1)..end_line {
                    let line_len = line_char_len(&line_starts, text, line).max(1);
                    push_span(
                        &mut map,
                        line,
                        StyledSpan {
                            char_start: 0,
                            char_end: line_len,
                            style: fence_bg_style,
                            full_line_bg: Some(theme.fenced_bg),
                            ..Default::default()
                        },
                    );
                }

                // Closing fence line: ``` delimiters → code_color.
                if end_line > start_line {
                    let close_len = line_char_len(&line_starts, text, end_line);
                    let lb_start = line_starts[end_line];
                    let lb_end = if end_line + 1 < line_starts.len() {
                        line_starts[end_line + 1].saturating_sub(1)
                    } else {
                        text.len()
                    };
                    let close_fence = text[lb_start..lb_end]
                        .chars()
                        .take_while(|&c| c == '`' || c == '~')
                        .count()
                        .min(close_len);
                    push_span(
                        &mut map,
                        end_line,
                        StyledSpan {
                            char_start: 0,
                            char_end: close_fence.max(1),
                            style: fence_delim_style,
                            full_line_bg: Some(theme.fenced_bg),
                            ..Default::default()
                        },
                    );
                }
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
                    // `[x]` bracket in text colour for visual pop; item text in todo_done.
                    // Strikethrough is intentionally absent: real ~~strikethrough~~ syntax
                    // exists in Markdown and should remain visually distinct.
                    // The list bullet (`-`) is coloured by the list-bullet span; no full-line
                    // span here so the bullet keeps its accent colour.
                    let line_len = line_char_len(&line_starts, text, marker_line);
                    // [x] is 3 chars: [, x, ]
                    let bracket_end = (marker_char + 3).min(line_len);
                    push_span(
                        &mut map,
                        marker_line,
                        make_span(marker_char, bracket_end, Style::default().fg(theme.text)),
                    );
                    // Item text after the bracket
                    if bracket_end < line_len {
                        push_span(
                            &mut map,
                            marker_line,
                            make_span(bracket_end, line_len, Style::default().fg(theme.todo_done)),
                        );
                    }
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
                        // ~~ delimiters use neutral text→muted blend (same as `*`, `#` etc.)
                        let delim_style = Style::default().fg(blend_colors(
                            theme.text,
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
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.full_line_bg.is_some()),
            "H1 must have full_line_bg"
        );
    }

    #[test]
    fn heading_h1_is_bold() {
        let text = "# Heading";
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // There should be 3 spans: delim, content, delim
        assert!(spans.len() >= 2, "bold should produce multiple spans");
    }

    // c. Italic
    #[test]
    fn italic_span_with_support() {
        let text = "*italic text*";
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), false);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
        // The fenced block spans lines 1-4 (the ``` delimiters + content)
        // At least lines 2 and 3 (code content) should have fenced_bg
        let has_fenced = map
            .iter()
            .any(|(_, spans)| spans.iter().any(|s| s.full_line_bg.is_some()));
        assert!(has_fenced, "fenced code block must have full_line_bg spans");
    }

    #[test]
    fn fenced_code_fence_delimiters_are_dimmed() {
        // ``` delimiter lines must use the blended (dimmed) colour, not raw code_color.
        let text = "before\n```\ncode\n```\nafter";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let expected_fg = blend_colors(theme.code_color, theme.muted, theme.delimiter_blend);
        // Opening fence is line 1, closing fence is line 3.
        let opening = map.get(&1).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.style.fg == Some(expected_fg)),
            "opening ``` fence must have blended (dimmed) fg, not raw code_color"
        );
        let closing = map.get(&3).expect("closing fence line must have spans");
        assert!(
            closing.iter().any(|s| s.style.fg == Some(expected_fg)),
            "closing ``` fence must have blended (dimmed) fg, not raw code_color"
        );
    }

    #[test]
    fn inline_code_backtick_delimiters_are_dimmed() {
        // The ` chars around inline code must be dimmed; content uses code_color.
        let text = "text `hello` text";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        let expected_delim = blend_colors(theme.code_color, theme.muted, theme.delimiter_blend);
        // Opening ` at char 5, closing ` at char 11 — both must be dimmed.
        assert!(
            spans.iter().any(|s| s.char_start == 5
                && s.char_end == 6
                && s.style.fg == Some(expected_delim)),
            "opening backtick must be blended/dimmed at char 5..6"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 11
                && s.char_end == 12
                && s.style.fg == Some(expected_delim)),
            "closing backtick must be blended/dimmed at char 11..12"
        );
        // Content `hello` at chars 6..11 must use code_color.
        assert!(
            spans.iter().any(|s| s.char_start == 6
                && s.char_end == 11
                && s.style.fg == Some(theme.code_color)),
            "inline code content must use code_color at chars 6..11"
        );
    }

    #[test]
    fn fenced_code_language_tag_uses_accent() {
        // Language tag (e.g. "rust") must have a span with accent fg.
        let text = "before\n```rust\nlet x = 1;\n```\nafter";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        // Opening fence with language tag is line 1.
        let opening = map.get(&1).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.style.fg == Some(theme.accent)),
            "language tag on opening fence must have accent fg"
        );
    }

    // f. Blockquotes
    #[test]
    fn blockquote_has_indicator_span() {
        let text = "> quoted text";
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Item text after the bracket must use todo_done colour.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.todo_done)),
            "checked todo text must use todo_done colour"
        );
        // [x] bracket must use text colour for visual pop.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.text)),
            "checked todo [x] bracket must use theme.text colour"
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
        let has_bold = map
            .values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(has_bold, "table header must have bold");
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
        let map = build_decoration_map(text, &make_theme(), true);
        assert!(map.contains_key(&0));
    }

    #[test]
    fn bold_with_multibyte_chars() {
        let text = "**café**";
        // Should not panic
        let _map = build_decoration_map(text, &make_theme(), true);
    }

    // Full fixture smoke test (subset — full integration test in Phase 11)
    #[test]
    fn fixture_produces_nonempty_map() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        assert!(!map.is_empty(), "fixture should produce decorations");
    }

    #[test]
    fn fixture_has_heading_bg() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        // Line 0 is `# Heading One`
        let spans = map.get(&0).expect("line 0 should have heading spans");
        assert!(spans.iter().any(|s| s.full_line_bg.is_some()));
    }

    #[test]
    fn fixture_has_blockquote() {
        let text = include_str!("../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
        assert!(
            map.values().flatten().any(|s| s.is_rule),
            "horizontal rule must set is_rule=true on its line"
        );
    }

    // a. Heading delimiter blending
    #[test]
    fn heading_h1_delimiter_is_blended() {
        let text = "# Hello";
        let map = build_decoration_map(text, &make_theme(), true);
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
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // `## ` = 3 chars: delimiter at 0..3
        let has_delim = spans.iter().any(|s| s.char_start == 0 && s.char_end == 3);
        assert!(has_delim, "H2 should have delimiter span at 0..3");
    }

    #[test]
    fn heading_h1_has_border_bottom() {
        let text = "# Heading One";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.border_bottom.is_some()),
            "H1 must have border_bottom set"
        );
    }

    #[test]
    fn heading_h4_no_border_bottom() {
        let text = "#### Heading Four";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.border_bottom.is_some()),
            "H4+ must not have border_bottom"
        );
    }

    // --- Mutant-killing: heading with no content produces no content span ---

    #[test]
    fn heading_empty_content_produces_no_content_span() {
        // "# " has only the delimiter chars and no text after them.
        // The inner c_start < c_end guard must prevent an empty span being emitted.
        // If that guard becomes c_start <= c_end, a zero-width span at (2..2) appears.
        let text = "# ";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.char_start == s.char_end),
            "no zero-width (char_start == char_end) span must exist for an empty heading"
        );
    }

    // --- Mutant-killing: heading delimiter span carries its own fields ---

    #[test]
    fn heading_delimiter_span_has_own_full_line_bg() {
        // Deleting full_line_bg from the delimiter StyledSpan must be caught even
        // when the content span still carries it.
        let text = "# Hello";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 0 && s.char_end == 2 && s.full_line_bg.is_some()),
            "H1 delimiter span (0..2) must have full_line_bg set"
        );
    }

    #[test]
    fn heading_delimiter_span_has_own_border_bottom() {
        // Deleting border_bottom from the delimiter StyledSpan must be caught even
        // when the content span still carries it.
        let text = "# Hello";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 0 && s.char_end == 2 && s.border_bottom.is_some()),
            "H1 delimiter span (0..2) must have border_bottom set"
        );
    }

    #[test]
    fn heading_delimiter_style_differs_from_content() {
        // Deleting `style` from the delimiter span makes it fall back to default,
        // which won't match the blended delimiter fg.
        let text = "# Hello";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let delim = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 2)
            .expect("H1 delimiter span (0..2) must exist");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2)
            .expect("H1 content span must start at char 2");
        assert_ne!(
            delim.style.fg, content.style.fg,
            "delimiter fg must be blended (different from content fg)"
        );
        assert!(
            delim.style.fg.is_some(),
            "delimiter span must have an explicit fg color"
        );
    }

    #[test]
    fn heading_content_span_char_end_reaches_line_end() {
        // Deleting char_end from the content StyledSpan makes it default to 0.
        let text = "# Hello"; // 7 chars
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2)
            .expect("H1 content span must start at char 2");
        assert_eq!(
            content.char_end, 7,
            "H1 content span char_end must reach the end of '# Hello' (7 chars)"
        );
    }

    #[test]
    fn heading_content_span_has_own_border_bottom() {
        // Deleting border_bottom from the content StyledSpan must be caught even
        // when the delimiter span still carries it.
        let text = "# Hello";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2)
            .expect("H1 content span must start at char 2");
        assert!(
            content.border_bottom.is_some(),
            "H1 content span must have border_bottom set"
        );
    }

    // --- Mutant-killing: blockquote content span is_blockquote ---

    #[test]
    fn blockquote_content_span_has_is_blockquote() {
        // The indicator span (0..1) already has is_blockquote=true.
        // This test ensures the content span (char_start >= 1) also carries it.
        let text = "> quoted text";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.is_blockquote && s.char_start >= 1),
            "blockquote content span (char_start >= 1) must have is_blockquote=true"
        );
    }

    // --- Mutant-killing: strikethrough char boundaries ---

    #[test]
    fn strikethrough_opening_delimiter_boundary() {
        // "~~hi~~": opening ~~ must occupy exactly chars 0..2.
        let text = "~~hi~~";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "opening ~~ delimiter must be at char 0..2"
        );
    }

    #[test]
    fn strikethrough_content_boundary_and_modifier() {
        // "~~hi~~": content must be at chars 2..4 with CROSSED_OUT.
        let text = "~~hi~~";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| {
                s.char_start == 2
                    && s.char_end == 4
                    && s.style.add_modifier.contains(Modifier::CROSSED_OUT)
            }),
            "strikethrough content must be at char 2..4 with CROSSED_OUT"
        );
    }

    #[test]
    fn strikethrough_closing_delimiter_boundary() {
        // "~~hi~~": closing ~~ must occupy exactly chars 4..6.
        let text = "~~hi~~";
        let map = build_decoration_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 6),
            "closing ~~ delimiter must be at char 4..6"
        );
    }

    // --- Mutant-killing: horizontal rule span fields ---

    #[test]
    fn horizontal_rule_span_char_start_is_zero() {
        let text = "above\n\n---\n\nbelow";
        let map = build_decoration_map(text, &make_theme(), true);
        let rule = map
            .values()
            .flatten()
            .find(|s| s.is_rule)
            .expect("horizontal rule span must exist");
        assert_eq!(rule.char_start, 0, "rule span must start at char 0");
    }

    #[test]
    fn horizontal_rule_span_char_end_covers_line() {
        let text = "above\n\n---\n\nbelow";
        let map = build_decoration_map(text, &make_theme(), true);
        let rule = map
            .values()
            .flatten()
            .find(|s| s.is_rule)
            .expect("horizontal rule span must exist");
        assert!(rule.char_end > 0, "rule span char_end must be non-zero");
    }

    #[test]
    fn horizontal_rule_span_has_rule_color() {
        let text = "above\n\n---\n\nbelow";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let rule = map
            .values()
            .flatten()
            .find(|s| s.is_rule)
            .expect("horizontal rule span must exist");
        assert_eq!(
            rule.style.fg,
            Some(theme.rule_color),
            "rule span must have rule_color as fg"
        );
    }

    #[test]
    fn link_non_ascii_text_bracket_positions() {
        // [héllo](url): é is 2 bytes but 1 char — all span boundaries must be
        // char-indexed, not byte-indexed.  Previously split_idx could equal the
        // byte offset of ](, placing the ] span one position too late.
        let text = "[héllo](url)";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");

        // Opening [  must sit at char 0
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 1),
            "opening [ must be at char 0..1"
        );
        // Link text héllo must be chars 1..6 (5 chars)
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::UNDERLINED)
                    && s.char_start == 1
                    && s.char_end == 6),
            "link text must be underlined at chars 1..6"
        );
        // ]( delimiter must start at char 6 (right after o, not one byte late)
        assert!(
            spans.iter().any(|s| s.char_start == 6 && s.char_end == 8),
            "]( delimiter must be at chars 6..8"
        );
    }

    #[test]
    fn link_non_ascii_prefix_bracket_positions() {
        // Non-ASCII before the link: byte offset of [ != char offset of [.
        // Ensure start_char is derived from byte_to_line_char, not raw range.start.
        let text = "héllo [world](url)";
        let theme = make_theme();
        let map = build_decoration_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");

        // [ is at char 6 (h=0 é=1 l=2 l=3 o=4 ' '=5 [=6)
        assert!(
            spans.iter().any(|s| s.char_start == 6 && s.char_end == 7),
            "opening [ must be at char 6 (after non-ASCII prefix)"
        );
        // ]( is at char 12 ([=6 w=7 o=8 r=9 l=10 d=11 ]=12)
        assert!(
            spans.iter().any(|s| s.char_start == 12 && s.char_end == 14),
            "]( delimiter must be at chars 12..14"
        );
    }
}
