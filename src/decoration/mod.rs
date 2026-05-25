use std::collections::HashMap;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};

use crate::config::{Theme, blend_colors};

mod spans;
mod words;

pub use spans::{byte_to_line_char, line_start_bytes};
pub use words::count_words;

use self::spans::{
    SpanParams, add_byte_range_span, line_char_len, make_span, push_span,
};
use self::words::{count_chars_in, link_split_char_idx};

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
// build_decoration_map
// ---------------------------------------------------------------------------

/// Build the full decoration map from `text` and simultaneously count words.
///
/// Returns `(DecorationMap, word_count)` so callers avoid a second parser pass.
/// Pure function — no terminal or UI side effects. This is the v1.5 migration seam:
/// when moving to a background thread, only the call site changes.
#[mutants::skip]
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
) -> (DecorationMap, usize) {
    let line_starts = line_start_bytes(text);
    let mut map: DecorationMap = HashMap::new();
    let mut word_count = 0usize;

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
                let mut delim_style = Style::default().fg(blend_colors(
                    heading_color,
                    theme.heading_bg,
                    theme.delimiter_blend,
                ));
                if bold {
                    delim_style = delim_style.add_modifier(Modifier::BOLD);
                }
                let border_bottom = matches!(
                    level,
                    HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3
                )
                .then_some(heading_color);

                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);

                let level_num = match level {
                    HeadingLevel::H1 => 1usize,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                let delim_chars = level_num + 1; // e.g. "# " = 2, "## " = 3

                let line_len = line_char_len(&line_starts, text, start_line);
                let delim_end = (start_char + delim_chars).min(line_len);

                // Delimiter span (`#`s + space).
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

                // Content span (heading text after the `# ` prefix).
                if delim_end < line_len {
                    push_span(
                        &mut map,
                        start_line,
                        StyledSpan {
                            char_start: delim_end,
                            char_end: line_len,
                            style: content_style,
                            full_line_bg: Some(theme.heading_bg),
                            border_bottom,
                            ..Default::default()
                        },
                    );
                }
            }

            // ---- b. Bold ----
            Event::Start(Tag::Strong) => {
                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) =
                    byte_to_line_char(&line_starts, text, range.end);

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
                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) =
                    byte_to_line_char(&line_starts, text, range.end);

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
            Event::Code(s) => {
                word_count += s.split_whitespace().count();
                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) =
                    byte_to_line_char(&line_starts, text, range.end);
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
                // Language tag blends accent toward muted so it pops without full brightness.
                let lang_style = Style::default()
                    .fg(blend_colors(
                        theme.accent,
                        theme.muted,
                        theme.delimiter_blend,
                    ))
                    .bg(theme.fenced_bg);

                // Opening fence line.
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

                // Closing fence line.
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
                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) =
                    byte_to_line_char(&line_starts, text, range.end);

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
                let (item_line, item_char) =
                    byte_to_line_char(&line_starts, text, range.start);

                let bullet_style = Style::default().fg(theme.accent);
                let bullet_end = if in_ordered_list {
                    let line_bytes_start = line_starts[item_line];
                    let scan_start = range.start.saturating_sub(line_bytes_start);
                    let line_text = &text[line_starts[item_line]..];
                    line_text[scan_start..]
                        .find(['.', ')'])
                        .map(|i| {
                            item_char
                                + count_chars_in(&line_text[scan_start..scan_start + i + 1])
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
                let (marker_line, marker_char) =
                    byte_to_line_char(&line_starts, text, range.start);

                if checked {
                    let line_len = line_char_len(&line_starts, text, marker_line);
                    // [x] is 3 chars at marker_char: [ x ]
                    let bracket_end = (marker_char + 3).min(line_len);
                    let muted = Style::default().fg(theme.muted);
                    let x_style = Style::default().fg(theme.text);
                    // `[`
                    push_span(
                        &mut map,
                        marker_line,
                        make_span(marker_char, (marker_char + 1).min(bracket_end), muted),
                    );
                    // `x`
                    if marker_char + 1 < bracket_end {
                        push_span(
                            &mut map,
                            marker_line,
                            make_span(
                                marker_char + 1,
                                (marker_char + 2).min(bracket_end),
                                x_style,
                            ),
                        );
                    }
                    // `]`
                    if marker_char + 2 < bracket_end {
                        push_span(
                            &mut map,
                            marker_line,
                            make_span(marker_char + 2, bracket_end, muted),
                        );
                    }
                    // Item text after the bracket
                    if bracket_end < line_len {
                        push_span(
                            &mut map,
                            marker_line,
                            make_span(
                                bracket_end,
                                line_len,
                                Style::default().fg(theme.todo_done),
                            ),
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
                let (start_line, start_char) =
                    byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) =
                    byte_to_line_char(&line_starts, text, range.end);

                if start_line == end_line {
                    let span_len = end_char_excl.saturating_sub(start_char);
                    if span_len >= 4 {
                        // ~~ delimiters use plain muted — blending toward text made them
                        // brighter than the struck-through content they surround.
                        let delim_style = Style::default().fg(theme.muted);
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

            // ---- m. Word count — accumulate plain text events ----
            // (Event::Code is handled above in its decoration arm)
            Event::Text(s) => {
                word_count += s.split_whitespace().count();
            }

            _ => {}
        }
    }

    (map, word_count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use pulldown_cmark::{Event, Options, Parser};
    use ratatui::style::Modifier;

    use crate::config::{Theme, blend_colors};

    use super::*;
    use super::words::link_split_char_idx;

    fn make_theme() -> Theme {
        Theme::default_theme()
    }

    /// Convenience wrapper: run build_decoration_map and discard the word count.
    fn build_map(text: &str, theme: &Theme, italic_support: bool) -> DecorationMap {
        build_decoration_map(text, theme, italic_support).0
    }

    // ---- Byte mapping tests ----

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
        assert_eq!(byte_to_line_char(&starts, text, 4), (1, 1));
    }

    #[test]
    fn byte_mapping_multibyte() {
        let text = "café\nok";
        let starts = line_start_bytes(text);
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

    // ---- a. Headings ----

    #[test]
    fn heading_h1_has_full_line_bg() {
        let text = "# Hello World";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.full_line_bg.is_some()),
            "H1 must have full_line_bg"
        );
    }

    #[test]
    fn heading_h1_is_bold() {
        let text = "# Heading";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans
                .iter()
                .all(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "H3 should not be bold"
        );
    }

    // ---- b. Bold ----

    #[test]
    fn bold_span_exists() {
        let text = "Text **bold content** here";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(spans.len() >= 2, "bold should produce multiple spans");
    }

    // ---- c. Italic ----

    #[test]
    fn italic_span_with_support() {
        let text = "*italic text*";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC)),
            "should not apply ITALIC modifier when italic_support=false"
        );
    }

    // ---- d. Inline code ----

    #[test]
    fn inline_code_has_code_bg() {
        let text = "text `code` text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.bg.is_some()),
            "inline code span must have a background color"
        );
    }

    // ---- e. Fenced code blocks ----

    #[test]
    fn fenced_code_block_has_bg_on_all_lines() {
        let text = "before\n```\ncode line 1\ncode line 2\n```\nafter";
        let map = build_map(text, &make_theme(), true);
        let has_fenced = map
            .iter()
            .any(|(_, spans)| spans.iter().any(|s| s.full_line_bg.is_some()));
        assert!(has_fenced, "fenced code block must have full_line_bg spans");
    }

    #[test]
    fn fenced_code_fence_delimiters_are_dimmed() {
        let text = "before\n```\ncode\n```\nafter";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let expected_fg = blend_colors(theme.code_color, theme.muted, theme.delimiter_blend);
        let opening = map.get(&1).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.style.fg == Some(expected_fg)),
            "opening ``` fence must have blended (dimmed) fg"
        );
        let closing = map.get(&3).expect("closing fence line must have spans");
        assert!(
            closing.iter().any(|s| s.style.fg == Some(expected_fg)),
            "closing ``` fence must have blended (dimmed) fg"
        );
    }

    #[test]
    fn inline_code_backtick_delimiters_are_dimmed() {
        let text = "text `hello` text";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        let expected_delim = blend_colors(theme.code_color, theme.muted, theme.delimiter_blend);
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
        assert!(
            spans.iter().any(|s| s.char_start == 6
                && s.char_end == 11
                && s.style.fg == Some(theme.code_color)),
            "inline code content must use code_color at chars 6..11"
        );
    }

    #[test]
    fn fenced_code_language_tag_is_dimmed_accent() {
        let text = "before\n```rust\nlet x = 1;\n```\nafter";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let expected = blend_colors(theme.accent, theme.muted, theme.delimiter_blend);
        let opening = map.get(&1).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.style.fg == Some(expected)),
            "language tag on opening fence must have dimmed accent fg"
        );
    }

    // ---- f. Blockquotes ----

    #[test]
    fn blockquote_has_indicator_span() {
        let text = "> quoted text";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.is_blockquote),
            "blockquote spans must have is_blockquote=true"
        );
    }

    // ---- g. Links ----

    #[test]
    fn link_text_has_underline() {
        let text = "[example](https://example.com)";
        let map = build_map(text, &make_theme(), true);
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
        assert_eq!(idx, Some(5));
    }

    // ---- h. Lists ----

    #[test]
    fn list_bullet_has_accent_color() {
        let text = "- item one\n- item two";
        let map = build_map(text, &make_theme(), true);
        let theme = make_theme();
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.accent)),
            "bullet must have accent color"
        );
    }

    // ---- i. Todo items ----

    #[test]
    fn todo_unchecked_bracket_has_accent() {
        let text = "- [ ] todo item";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.todo_done)),
            "checked todo text must use todo_done colour"
        );
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.text)),
            "checked todo [x] bracket must use theme.text colour"
        );
        assert!(
            !spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT)),
            "checked todo must not have CROSSED_OUT"
        );
    }

    // ---- j. Tables ----

    #[test]
    fn table_pipes_have_muted_color() {
        let text = "| A | B |\n| - | - |\n| 1 | 2 |";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
        let has_bold = map
            .values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(has_bold, "table header must have bold");
    }

    // ---- Word count ----

    #[test]
    fn word_count_excludes_markdown_syntax() {
        assert_eq!(count_words("**hello**"), 1);
        assert_eq!(count_words("# Title\n\nTwo words."), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn word_count_counts_code_content() {
        assert_eq!(count_words("`word`"), 1);
    }

    // ---- Multi-byte safety ----

    #[test]
    fn heading_with_multibyte_chars() {
        let text = "# Café résumé";
        let map = build_map(text, &make_theme(), true);
        assert!(map.contains_key(&0));
    }

    #[test]
    fn bold_with_multibyte_chars() {
        let text = "**café**";
        let _map = build_map(text, &make_theme(), true);
    }

    // ---- Fixture smoke tests ----

    #[test]
    fn fixture_produces_nonempty_map() {
        let text = include_str!("../../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        assert!(!map.is_empty(), "fixture should produce decorations");
    }

    #[test]
    fn fixture_has_heading_bg() {
        let text = include_str!("../../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have heading spans");
        assert!(spans.iter().any(|s| s.full_line_bg.is_some()));
    }

    #[test]
    fn fixture_has_blockquote() {
        let text = include_str!("../../tests/fixtures/sample.md");
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        assert!(
            map.values().flatten().any(|s| s.is_blockquote),
            "fixture should have blockquote spans"
        );
    }

    #[test]
    fn fixture_word_count_nonzero() {
        let text = include_str!("../../tests/fixtures/sample.md");
        assert!(count_words(text) > 100);
    }

    // ---- k. Strikethrough ----

    #[test]
    fn strikethrough_has_crossed_out_modifier() {
        let text = "normal ~~struck~~ normal";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.len() >= 2,
            "strikethrough should produce multiple spans"
        );
    }

    // ---- l. Horizontal rule ----

    #[test]
    fn horizontal_rule_sets_is_rule_flag() {
        let text = "above\n\n---\n\nbelow";
        let map = build_map(text, &make_theme(), true);
        assert!(
            map.values().flatten().any(|s| s.is_rule),
            "horizontal rule must set is_rule=true on its line"
        );
    }

    // ---- Heading delimiter blending ----

    #[test]
    fn heading_h1_delimiter_is_blended() {
        let text = "# Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let has_delim = spans.iter().any(|s| s.char_start == 0 && s.char_end == 2);
        let has_content = spans.iter().any(|s| s.char_start == 2);
        assert!(has_delim, "H1 should have a delimiter span at 0..2");
        assert!(has_content, "H1 should have a content span starting at char 2");
        let delim_span = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 2)
            .expect("delimiter span must exist");
        let expected_delim =
            blend_colors(theme.headings.h1, theme.heading_bg, theme.delimiter_blend);
        assert_eq!(
            delim_span.style.fg,
            Some(expected_delim),
            "H1 delimiter must be blended toward heading_bg"
        );
    }

    #[test]
    fn heading_h2_delimiter_is_three_chars() {
        let text = "## Title";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let has_delim = spans.iter().any(|s| s.char_start == 0 && s.char_end == 3);
        assert!(has_delim, "H2 should have delimiter span at 0..3");
    }

    #[test]
    fn heading_h1_has_border_bottom() {
        let text = "# Heading One";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.border_bottom.is_some()),
            "H1 must have border_bottom set"
        );
    }

    #[test]
    fn heading_h4_no_border_bottom() {
        let text = "#### Heading Four";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.border_bottom.is_some()),
            "H4+ must not have border_bottom"
        );
    }

    #[test]
    fn heading_empty_content_produces_no_content_span() {
        let text = "# ";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.char_start == s.char_end),
            "no zero-width span must exist for an empty heading"
        );
    }

    #[test]
    fn heading_delimiter_span_has_own_full_line_bg() {
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
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
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
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
        let text = "# Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
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
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2)
            .expect("H1 content span must start at char 2");
        assert_eq!(content.char_end, 7, "H1 content span char_end must reach 7");
    }

    #[test]
    fn heading_content_span_has_own_border_bottom() {
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
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

    // ---- Blockquote content span ----

    #[test]
    fn blockquote_content_span_has_is_blockquote() {
        let text = "> quoted text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.is_blockquote && s.char_start >= 1),
            "blockquote content span must have is_blockquote=true"
        );
    }

    // ---- Strikethrough char boundaries ----

    #[test]
    fn strikethrough_opening_delimiter_boundary() {
        let text = "~~hi~~";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "opening ~~ delimiter must be at char 0..2"
        );
    }

    #[test]
    fn strikethrough_content_boundary_and_modifier() {
        let text = "~~hi~~";
        let map = build_map(text, &make_theme(), true);
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
        let text = "~~hi~~";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 6),
            "closing ~~ delimiter must be at char 4..6"
        );
    }

    // ---- Horizontal rule span fields ----

    #[test]
    fn horizontal_rule_span_char_start_is_zero() {
        let text = "above\n\n---\n\nbelow";
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &make_theme(), true);
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
        let map = build_map(text, &theme, true);
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

    // ---- Link non-ASCII ----

    #[test]
    fn link_non_ascii_text_bracket_positions() {
        let text = "[héllo](url)";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 1),
            "opening [ must be at char 0..1"
        );
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::UNDERLINED)
                    && s.char_start == 1
                    && s.char_end == 6),
            "link text must be underlined at chars 1..6"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 6 && s.char_end == 8),
            "]( delimiter must be at chars 6..8"
        );
    }

    #[test]
    fn link_non_ascii_prefix_bracket_positions() {
        let text = "héllo [world](url)";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 6 && s.char_end == 7),
            "opening [ must be at char 6"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 12 && s.char_end == 14),
            "]( delimiter must be at chars 12..14"
        );
    }

    // ---- Inline code range diagnostic ----

    #[test]
    fn debug_inline_code_multiline_paragraph_ranges() {
        let text = "`inline code` at\n`too`.";
        let options =
            Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
        let mut code_ranges: Vec<(String, std::ops::Range<usize>)> = Vec::new();
        for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
            if let Event::Code(s) = event {
                code_ranges.push((s.to_string(), range));
            }
        }
        assert_eq!(code_ranges.len(), 2, "expected 2 Code events");
        let (c0, r0) = &code_ranges[0];
        assert_eq!(c0, "inline code");
        assert_eq!(r0.start, 0);
        assert_eq!(r0.end, 13);
        let (c1, r1) = &code_ranges[1];
        assert_eq!(c1, "too");
        assert_eq!(r1.start, 17);
        assert_eq!(r1.end, 22);
    }

    // ---- No span bleeds past closing backtick ----

    fn spans_covering(map: &DecorationMap, line: usize, char_pos: usize) -> Vec<(usize, usize)> {
        map.get(&line)
            .map(|spans| {
                spans
                    .iter()
                    .filter(|s| s.char_start <= char_pos && s.char_end > char_pos)
                    .map(|s| (s.char_start, s.char_end))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn no_span_past_closing_backtick_singleline() {
        let text = "`too`.";
        let map = build_map(text, &make_theme(), true);
        let covering = spans_covering(&map, 0, 5);
        assert!(
            covering.is_empty(),
            "period after closing backtick must not be in any span; got: {:?}",
            covering
        );
    }

    #[test]
    fn no_span_past_closing_backtick_multiline_paragraph() {
        let text = "`inline code` at\n`too`.";
        let map = build_map(text, &make_theme(), true);
        let covering = spans_covering(&map, 1, 5);
        assert!(
            covering.is_empty(),
            "period after `too` on line 1 must not be in any span; got: {:?}",
            covering
        );
    }

    #[test]
    fn no_span_past_closing_backtick_comma() {
        let text = "`foo`,";
        let map = build_map(text, &make_theme(), true);
        let covering = spans_covering(&map, 0, 5);
        assert!(
            covering.is_empty(),
            "comma after closing backtick must not be in any span; got: {:?}",
            covering
        );
    }

    #[test]
    fn no_span_past_closing_backtick_in_sentence() {
        let text = "see `foo`. More";
        let map = build_map(text, &make_theme(), true);
        let covering_period = spans_covering(&map, 0, 9);
        let covering_space = spans_covering(&map, 0, 10);
        assert!(
            covering_period.is_empty(),
            "period at char 9 must not be in any span; got: {:?}",
            covering_period
        );
        assert!(
            covering_space.is_empty(),
            "space at char 10 must not be in any span; got: {:?}",
            covering_space
        );
    }
}
