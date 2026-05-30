use std::collections::HashMap;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};

use crate::config::{Theme, blend_colors};
use crate::highlighting::HighlightCache;

mod spans;
mod words;

pub use spans::{byte_to_line_char, line_start_bytes};
pub use words::count_words;

use self::spans::{SpanParams, add_byte_range_span, line_char_len, make_span, push_span};
use self::words::{count_chars_in, link_split_char_idx};

// ---------------------------------------------------------------------------
// Span-emission helpers
// ---------------------------------------------------------------------------

/// Layer an additional `modifier` on top of all spans in `map[line]` that
/// overlap `[range_start, range_end)`.
///
/// Used when an *outer* tag's End fires after the inner tag's spans are already
/// committed.  For example, in `**bold and *italic* bold**` the Strong End fires
/// after the Emphasis End has already placed italic spans; calling this with
/// `Modifier::BOLD` ensures the overlap region ends up with both BOLD and ITALIC.
fn add_modifier_to_existing(
    map: &mut DecorationMap,
    line: usize,
    range_start: usize,
    range_end: usize,
    modifier: Modifier,
) {
    if let Some(spans) = map.get_mut(&line) {
        for span in spans.iter_mut() {
            if span.char_end > range_start && span.char_start < range_end {
                span.style = span.style.add_modifier(modifier);
            }
        }
    }
}

/// Emit a styled span in segments over `[range_start, range_end)`, skipping
/// any char-ranges that are already decorated in `map` for the given line.
///
/// This lets an *outer* inline tag (e.g. Emphasis) coexist with *inner* tags
/// (e.g. Strong, Code) that were processed first.  Without this, a single
/// large outer content span would swallow every inner span in `split_into_spans`.
fn emit_content_around_existing(
    map: &mut DecorationMap,
    line: usize,
    range_start: usize,
    range_end: usize,
    style: Style,
) {
    if range_start >= range_end {
        return;
    }

    // Collect existing blocked char-ranges inside [range_start, range_end).
    // We clone the ranges out so the immutable borrow on `map` is released
    // before we call push_span (which needs a mutable borrow).
    let mut blocked: Vec<(usize, usize)> = map
        .get(&line)
        .map(|spans| {
            spans
                .iter()
                .filter(|s| s.char_end > range_start && s.char_start < range_end)
                .map(|s| (s.char_start.max(range_start), s.char_end.min(range_end)))
                .collect()
        })
        .unwrap_or_default();
    blocked.sort_by_key(|&(start, _)| start);

    // Emit content in the gaps between blocked regions.
    let mut pos = range_start;
    for (block_start, block_end) in blocked {
        if pos < block_start {
            push_span(map, line, make_span(pos, block_start, style));
        }
        if block_end > pos {
            pos = block_end;
        }
    }
    if pos < range_end {
        push_span(map, line, make_span(pos, range_end, style));
    }
}

// ---------------------------------------------------------------------------
// Bold+italic combined helper
// ---------------------------------------------------------------------------

/// Emit spans for a bold+italic region where `outer_range` is the enclosing
/// tag's byte range and `inner_range` is the nested tag's byte range.
/// `inner_is_strong` is true when the *inner* tag uses `**` (2-char delimiter);
/// false when it uses `*`/`_` (1-char delimiter).
///
/// pulldown-cmark nests `***text***` as `Emphasis { Strong { text } }`, so the
/// outer delimiter is 1 char (`*`) and the inner is 2 chars (`**`).
/// `**_text_**` nests as `Strong { Emphasis { text } }`, with outer = 2 and inner = 1.
#[allow(clippy::too_many_arguments)]
fn emit_bold_italic_spans(
    map: &mut DecorationMap,
    line_starts: &[usize],
    text: &str,
    outer_range: std::ops::Range<usize>,
    inner_range: std::ops::Range<usize>,
    inner_is_strong: bool,
    theme: &Theme,
    italic_support: bool,
) {
    let inner_delim = if inner_is_strong { 2usize } else { 1 };

    let (start_line, start_char) = byte_to_line_char(line_starts, text, outer_range.start);
    let (end_line, end_char_excl) = byte_to_line_char(line_starts, text, outer_range.end);
    if start_line != end_line {
        return; // multi-line bold+italic not handled in v1
    }

    let (_, inner_start_char) = byte_to_line_char(line_starts, text, inner_range.start);
    let (_, inner_end_char) = byte_to_line_char(line_starts, text, inner_range.end);

    let content_start = inner_start_char + inner_delim;
    let content_end = inner_end_char.saturating_sub(inner_delim);
    if content_start > content_end || content_start >= end_char_excl {
        return;
    }

    // Blend bold and italic colors at 50 % for the combined content colour.
    let combined_color = blend_colors(theme.bold_color, theme.italic_color, 0.5);
    let delim_color = blend_colors(combined_color, theme.muted, theme.delimiter_blend);

    let delim_style = Style::default()
        .fg(delim_color)
        .add_modifier(Modifier::BOLD);

    let mut content_style = Style::default()
        .fg(combined_color)
        .add_modifier(Modifier::BOLD);
    if italic_support {
        content_style = content_style.add_modifier(Modifier::ITALIC);
    }

    // Opening delimiter (`***` / `**_` / `_**` / `___`)
    if content_start > start_char {
        push_span(
            map,
            start_line,
            make_span(start_char, content_start, delim_style),
        );
    }
    // Content
    if content_end > content_start {
        push_span(
            map,
            start_line,
            make_span(content_start, content_end, content_style),
        );
    }
    // Closing delimiter
    if end_char_excl > content_end {
        push_span(
            map,
            end_line,
            make_span(content_end, end_char_excl, delim_style),
        );
    }
}

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
    use ratatui::style::Style;
    let mut map: DecorationMap = HashMap::new();
    for (line_idx, line_spans) in hl.iter().enumerate() {
        let log_line = line_offset + line_idx;
        for hs in line_spans {
            let span = StyledSpan {
                char_start: hs.char_start,
                char_end: hs.char_end,
                style: Style::default().fg(hs.fg),
                is_blockquote: false,
                continuation_indent: 0,
                full_line_bg: None,
                border_bottom: None,
                is_rule: false,
            };
            map.entry(log_line).or_default().push(span);
        }
    }
    map
}

// ---------------------------------------------------------------------------
// build_decoration_map
// ---------------------------------------------------------------------------

/// Build the full decoration map from `text` and simultaneously count words.
///
/// Returns `(DecorationMap, word_count)` so callers avoid a second parser pass.
/// Pure function — no terminal or UI side effects. This is the v1.5 migration seam:
/// when moving to a background thread, only the call site changes.
///
/// `highlight_cache` is optional: pass `Some(&cache)` to enable syntect syntax
/// highlighting for fenced code blocks, or `None` to disable it (fenced_bg-only).
#[mutants::skip]
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
    highlight_cache: Option<&HighlightCache>,
) -> (DecorationMap, usize) {
    let line_starts = line_start_bytes(text);
    let mut map: DecorationMap = HashMap::new();
    let mut word_count = 0usize;

    let options =
        Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;

    let parser = Parser::new_ext(text, options).into_offset_iter();

    // State tracking
    let mut in_ordered_list = false;
    // Bold+italic nesting detection: set on Start, cleared on End.
    let mut in_strong: Option<std::ops::Range<usize>> = None;
    let mut in_emphasis: Option<std::ops::Range<usize>> = None;
    // TableHead: capture the byte range on Start so End can emit gaps-only spans.
    let mut in_table_head: Option<std::ops::Range<usize>> = None;

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

                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);

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

            // ---- b. Bold — record range; emit on End(Strong) ----
            Event::Start(Tag::Strong) => {
                in_strong = Some(range.clone());
            }

            // ---- c. Italic — record range; emit on End(Emphasis) ----
            Event::Start(Tag::Emphasis) => {
                in_emphasis = Some(range.clone());
            }

            // ---- b/c end: emit bold, italic, or combined bold+italic ----
            //
            // Combined bold+italic is only triggered when the two tags are
            // *directly adjacent* (delimiters touch with no intervening text).
            // For `***text***`: Emphasis(0..N) wraps Strong(1..N-1) →
            //   emph.start + 1 == strong.start  AND  strong.end + 1 == emph.end.
            // For `**_text_**`: Strong(0..N) wraps Emphasis(2..N-2) →
            //   strong.start + 2 == emph.start  AND  emph.end + 2 == strong.end.
            //
            // Non-adjacent nesting (`*italic **bold** rest*`) is two independent
            // decorations; the outer tag's state is left in place for its own End.
            Event::End(TagEnd::Strong) => {
                if let Some(strong_range) = in_strong.take() {
                    // Peek at in_emphasis to check adjacency without consuming it.
                    let adjacent = in_emphasis.as_ref().is_some_and(|emph| {
                        emph.start + 1 == strong_range.start && strong_range.end + 1 == emph.end
                    });
                    if adjacent {
                        // Emphasis(outer) wraps Strong(inner) with touching delimiters.
                        let outer = in_emphasis.take().unwrap();
                        emit_bold_italic_spans(
                            &mut map,
                            &line_starts,
                            text,
                            outer,
                            strong_range,
                            true, // inner_is_strong
                            theme,
                            italic_support,
                        );
                    } else {
                        // Plain bold — non-adjacent or no Emphasis context at all.
                        // Leave in_emphasis in place so its own End(Emphasis) fires later.
                        let (start_line, start_char) =
                            byte_to_line_char(&line_starts, text, strong_range.start);
                        let (end_line, end_char_excl) =
                            byte_to_line_char(&line_starts, text, strong_range.end);
                        if start_line == end_line {
                            let span_len = end_char_excl.saturating_sub(start_char);
                            if span_len >= 4 {
                                let delim_style = Style::default()
                                    .fg(blend_colors(
                                        theme.text,
                                        theme.muted,
                                        theme.delimiter_blend,
                                    ))
                                    .add_modifier(Modifier::BOLD);
                                let content_style = Style::default()
                                    .fg(theme.bold_color)
                                    .add_modifier(Modifier::BOLD);
                                push_span(
                                    &mut map,
                                    start_line,
                                    make_span(start_char, start_char + 2, delim_style),
                                );
                                emit_content_around_existing(
                                    &mut map,
                                    start_line,
                                    start_char + 2,
                                    end_char_excl.saturating_sub(2),
                                    content_style,
                                );
                                push_span(
                                    &mut map,
                                    end_line,
                                    make_span(end_char_excl - 2, end_char_excl, delim_style),
                                );
                                // Layer BOLD onto any inner spans (e.g. italic) in the
                                // bold content region so the overlap has both modifiers.
                                add_modifier_to_existing(
                                    &mut map,
                                    start_line,
                                    start_char + 2,
                                    end_char_excl.saturating_sub(2),
                                    Modifier::BOLD,
                                );
                            }
                        }
                    }
                }
            }

            Event::End(TagEnd::Emphasis) => {
                if let Some(emph_range) = in_emphasis.take() {
                    // Peek at in_strong to check adjacency without consuming it.
                    let adjacent = in_strong.as_ref().is_some_and(|strong| {
                        strong.start + 2 == emph_range.start && emph_range.end + 2 == strong.end
                    });
                    if adjacent {
                        // Strong(outer) wraps Emphasis(inner) with touching delimiters.
                        let outer = in_strong.take().unwrap();
                        emit_bold_italic_spans(
                            &mut map,
                            &line_starts,
                            text,
                            outer,
                            emph_range,
                            false, // inner_is_strong = false (inner is Emphasis, 1-char delim)
                            theme,
                            italic_support,
                        );
                    } else {
                        // Plain italic — non-adjacent or no Strong context at all.
                        let (start_line, start_char) =
                            byte_to_line_char(&line_starts, text, emph_range.start);
                        let (end_line, end_char_excl) =
                            byte_to_line_char(&line_starts, text, emph_range.end);
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
                                push_span(
                                    &mut map,
                                    start_line,
                                    make_span(start_char, start_char + 1, delim_style),
                                );
                                emit_content_around_existing(
                                    &mut map,
                                    start_line,
                                    start_char + 1,
                                    end_char_excl.saturating_sub(1),
                                    content_style,
                                );
                                push_span(
                                    &mut map,
                                    end_line,
                                    make_span(end_char_excl - 1, end_char_excl, delim_style),
                                );
                                // Layer ITALIC onto any inner spans (e.g. bold) in the
                                // italic content region so the overlap has both modifiers.
                                if italic_support {
                                    add_modifier_to_existing(
                                        &mut map,
                                        start_line,
                                        start_char + 1,
                                        end_char_excl.saturating_sub(1),
                                        Modifier::ITALIC,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // ---- d. Inline code ----
            Event::Code(s) => {
                word_count += s.split_whitespace().count();
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
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let (start_line, _) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, _) = byte_to_line_char(
                    &line_starts,
                    text,
                    range.end.saturating_sub(1).max(range.start),
                );
                let fence_bg_style = Style::default().fg(theme.text).bg(theme.fenced_bg);
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

                // Content lines: try syntect highlighting, fall back to fenced_bg-only.
                //
                // Extract the raw content (with newlines) between the fence delimiters
                // so syntect sees exactly what the user typed.
                let lang_str = lang.as_ref();
                let block_content: String = ((start_line + 1)..end_line)
                    .map(|l| {
                        let ls = line_starts[l];
                        let le = if l + 1 < line_starts.len() {
                            line_starts[l + 1]
                        } else {
                            text.len()
                        };
                        &text[ls..le]
                    })
                    .collect();

                let hl_result: Option<crate::highlighting::BlockHighlights> = highlight_cache
                    .and_then(|cache| cache.highlight_block(lang_str, &block_content));

                for (block_row, line) in ((start_line + 1)..end_line).enumerate() {
                    let line_len = line_char_len(&line_starts, text, line).max(1);

                    match hl_result.as_ref().and_then(|hl| hl.get(block_row)) {
                        Some(hl_spans) if !hl_spans.is_empty() => {
                            // Emit syntect fg spans directly — NO separate full-line background
                            // span.  split_into_spans clips any span whose char_start falls
                            // before the current char_pos, so a wide 0..N background span
                            // would consume the entire line and cause all subsequent fg spans
                            // to be skipped.  Instead we put full_line_bg on the first syntect
                            // span; since syntect always produces contiguous spans starting at
                            // col 0, this is always the span covering char 0.
                            for (i, hl_span) in hl_spans.iter().enumerate() {
                                let cs = hl_span.char_start.min(line_len);
                                let ce = hl_span.char_end.min(line_len);
                                if cs < ce {
                                    push_span(
                                        &mut map,
                                        line,
                                        StyledSpan {
                                            char_start: cs,
                                            char_end: ce,
                                            style: Style::default()
                                                .fg(hl_span.fg)
                                                .bg(theme.fenced_bg),
                                            // Only the first span signals full_line_bg so the
                                            // background fills the column beyond the last char.
                                            full_line_bg: if i == 0 {
                                                Some(theme.fenced_bg)
                                            } else {
                                                None
                                            },
                                            ..Default::default()
                                        },
                                    );
                                }
                            }
                        }
                        _ => {
                            // No highlights (disabled / unknown lang / empty line) — fenced_bg.
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
                    }
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

                for line in start_line..=end_line {
                    let line_len = line_char_len(&line_starts, text, line);
                    if line_len == 0 {
                        continue;
                    }
                    // ▌ indicator at char 0 (covers the `>` char visually).
                    //
                    // The rest of the line is intentionally left unspanned so that inline
                    // decorations (bold, italic, code, links) emit their own spans without
                    // a wide content span blocking `emit_content_around_existing`.
                    //
                    // The renderer detects `is_blockquote` on this indicator span and
                    // applies `theme.blockquote_color` as the default fg for any
                    // undecorated text on the line, preserving the blockquote visual style
                    // while letting inline markup render at its own correct colors.
                    push_span(
                        &mut map,
                        line,
                        StyledSpan {
                            char_start: 0,
                            char_end: 1,
                            style: indicator_style,
                            is_blockquote: true,
                            // Continuation visual rows indent 2 cols to align
                            // with text start after `> `.
                            continuation_indent: 2,
                            ..Default::default()
                        },
                    );
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
                let bullet_end = if in_ordered_list {
                    let line_bytes_start = line_starts[item_line];
                    let scan_start = range.start.saturating_sub(line_bytes_start);
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
                // continuation_indent = bullet_end + 1 so that soft-wrapped
                // continuation rows align with the item text (past bullet + space).
                let ci = (bullet_end + 1).min(255) as u8;
                push_span(
                    &mut map,
                    item_line,
                    StyledSpan {
                        char_start: item_char,
                        char_end: bullet_end,
                        style: bullet_style,
                        continuation_indent: ci,
                        ..Default::default()
                    },
                );
            }

            // ---- i. Todo items ----
            Event::TaskListMarker(checked) => {
                let (marker_line, marker_char) = byte_to_line_char(&line_starts, text, range.start);

                // The full task-list glyph is `- [ ] ` / `- [x] ` (marker_char chars
                // before `[`, then `[`, one char, `]`, space = 4 more chars).
                // Upgrade the bullet span's continuation_indent so that soft-wrapped
                // continuation rows align with the item text, not just the `- ` prefix.
                let task_ci = (marker_char + 4).min(255) as u8;
                if let Some(spans) = map.get_mut(&marker_line) {
                    for span in spans.iter_mut() {
                        if span.continuation_indent > 0 {
                            span.continuation_indent = task_ci;
                        }
                    }
                }

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
                            make_span(marker_char + 1, (marker_char + 2).min(bracket_end), x_style),
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
                // Capture the byte range; defer span emission to End(TableHead) so
                // that inline formatting spans (bold, italic, code) are already in
                // the map and `emit_content_around_existing` can skip over them.
                in_table_head = Some(range.clone());
            }

            Event::End(TagEnd::TableHead) => {
                if let Some(head_range) = in_table_head.take() {
                    let style = Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD);

                    let (start_line, _) = byte_to_line_char(&line_starts, text, head_range.start);
                    let (end_line, _) = byte_to_line_char(
                        &line_starts,
                        text,
                        head_range.end.saturating_sub(1).max(head_range.start),
                    );

                    for line in start_line..=end_line {
                        let line_len = line_char_len(&line_starts, text, line);
                        emit_content_around_existing(&mut map, line, 0, line_len, style);
                    }
                }
            }

            // ---- k. Strikethrough ----
            Event::Start(Tag::Strikethrough) => {
                let (start_line, start_char) = byte_to_line_char(&line_starts, text, range.start);
                let (end_line, end_char_excl) = byte_to_line_char(&line_starts, text, range.end);

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

    use super::words::link_split_char_idx;
    use super::*;

    fn make_theme() -> Theme {
        Theme::default_theme()
    }

    /// Convenience wrapper: run build_decoration_map and discard the word count.
    fn build_map(text: &str, theme: &Theme, italic_support: bool) -> DecorationMap {
        build_decoration_map(text, theme, italic_support, None).0
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

    #[test]
    fn heading_h1_delimiter_span_is_bold() {
        // The `# ` prefix span (0..2) must carry BOLD to match the content style.
        let text = "# Heading";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let delim = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 2)
            .expect("H1 delimiter span must exist at 0..2");
        assert!(
            delim.style.add_modifier.contains(Modifier::BOLD),
            "H1 delimiter span must be BOLD"
        );
    }

    #[test]
    fn heading_h2_delimiter_span_is_bold() {
        // The `## ` prefix span (0..3) must carry BOLD.
        let text = "## Heading";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let delim = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 3)
            .expect("H2 delimiter span must exist at 0..3");
        assert!(
            delim.style.add_modifier.contains(Modifier::BOLD),
            "H2 delimiter span must be BOLD"
        );
    }

    #[test]
    fn heading_h3_delimiter_span_not_bold() {
        // H3 content is not bold, so its delimiter should also not be bold.
        let text = "### Heading Three";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let delim = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 4)
            .expect("H3 delimiter span must exist at 0..4");
        assert!(
            !delim.style.add_modifier.contains(Modifier::BOLD),
            "H3 delimiter span must NOT be bold (H3 content is not bold)"
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

    // ---- b+c. Bold+italic combined (***text***) ----

    #[test]
    fn bold_italic_has_both_modifiers_with_support() {
        // ***hi*** — should produce BOLD | ITALIC on the content span.
        let text = "***hi***";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans.iter().find(|s| s.char_start == 3 && s.char_end == 5);
        assert!(content.is_some(), "content span at chars 3..5 must exist");
        let content = content.unwrap();
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "bold+italic content must have BOLD modifier"
        );
        assert!(
            content.style.add_modifier.contains(Modifier::ITALIC),
            "bold+italic content must have ITALIC modifier when italic_support=true"
        );
    }

    #[test]
    fn bold_italic_without_support_has_bold_not_italic() {
        let text = "***hi***";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans.iter().find(|s| s.char_start == 3 && s.char_end == 5);
        assert!(
            content.is_some(),
            "content span at 3..5 must exist even without italic support"
        );
        let content = content.unwrap();
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "BOLD must be applied regardless of italic_support"
        );
        assert!(
            !content.style.add_modifier.contains(Modifier::ITALIC),
            "ITALIC must not be applied when italic_support=false"
        );
    }

    #[test]
    fn bold_italic_delimiter_boundaries() {
        // ***hi*** = chars 0..8; opening *** = 0..3, content = 3..5, closing *** = 5..8.
        let text = "***hi***";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "opening *** delimiter must be at chars 0..3"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 5),
            "content must be at chars 3..5"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 8),
            "closing *** delimiter must be at chars 5..8"
        );
    }

    #[test]
    fn bold_italic_alt_syntax_bold_then_italic() {
        // **_text_** — Strong wraps Emphasis; outer=0..10, inner=2..8.
        // Opening delim = 0..3 (**_), content = 3..7, closing = 7..10 (_**).
        let text = "**_hi_**";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Content "hi" is at chars 3..5 (inner_start=2 + 1 = 3, inner_end=6 - 1 = 5).
        let content = spans.iter().find(|s| s.char_start == 3 && s.char_end == 5);
        assert!(content.is_some(), "content span must exist for **_hi_**");
        let content = content.unwrap();
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "**_hi_** content must be bold"
        );
        assert!(
            content.style.add_modifier.contains(Modifier::ITALIC),
            "**_hi_** content must be italic"
        );
    }

    #[test]
    fn bold_italic_non_adjacent_italic_wrapping_bold_overlap_has_bold_italic() {
        // *italic and **nested bold*** — bold nested inside italic.
        // The overlap region ("nested bold") must have BOTH BOLD+ITALIC so it renders
        // as bold-italic in the terminal.  The prefix ("italic and ") must be
        // ITALIC-only — no BOLD should bleed into text that is only italic.
        let text = "*italic and **nested bold***";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");

        // Overlap must have both modifiers.
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)
                    && s.style.add_modifier.contains(Modifier::ITALIC)),
            "bold content nested inside italic must carry both BOLD+ITALIC modifiers"
        );

        // The pure-italic prefix ("italic and ") must be ITALIC-only.
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::ITALIC)
                    && !s.style.add_modifier.contains(Modifier::BOLD)),
            "italic prefix before the bold region must be ITALIC-only (no BOLD)"
        );
    }

    #[test]
    fn bold_italic_non_adjacent_bold_wrapping_italic_overlap_has_bold_italic() {
        // **bold and *nested italic* inside bold** — italic nested inside bold.
        // The overlap region ("nested italic") must have BOTH BOLD+ITALIC.
        // The pure-bold prefix ("bold and ") must be BOLD-only.
        let text = "**bold and *nested italic* inside bold**";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");

        // Overlap must have both modifiers.
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)
                    && s.style.add_modifier.contains(Modifier::ITALIC)),
            "italic content nested inside bold must carry both BOLD+ITALIC modifiers"
        );

        // The pure-bold prefix ("bold and ") must be BOLD-only.
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)
                    && !s.style.add_modifier.contains(Modifier::ITALIC)),
            "bold prefix before the italic region must be BOLD-only (no ITALIC)"
        );
    }

    #[test]
    fn nested_bold_inside_italic_uses_bold_color_not_italic_color() {
        // *italic and **nested bold*** — "nested bold" must have bold_color, not italic_color.
        // The bug (before emit_content_around_existing) was that the outer italic content
        // span swallowed the inner bold spans in split_into_spans, making "nested bold"
        // render with italic_color.
        let text = "*italic and **nested bold***";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");

        // "nested bold" content span must use bold_color and carry BOLD.
        // It will also carry ITALIC (layered by the outer italic span), which is correct.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.bold_color)
                && s.style.add_modifier.contains(Modifier::BOLD)),
            "bold content inside italic must use bold_color with BOLD modifier"
        );

        // The italic prefix ("italic and ") must also have italic_color.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.italic_color)),
            "italic prefix before bold must use italic_color"
        );
    }

    #[test]
    fn nested_italic_inside_bold_uses_italic_color_not_bold_color() {
        // **bold and *nested italic* inside bold** — "nested italic" must have italic_color.
        // Outer bold content must have bold_color in the regions outside the italic.
        let text = "**bold and *nested italic* inside bold**";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");

        // "nested italic" must use italic_color.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.italic_color)),
            "italic content inside bold must use italic_color"
        );

        // Outer bold regions ("bold and " and " inside bold") must use bold_color.
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.bold_color)
                && s.style.add_modifier.contains(Modifier::BOLD)),
            "outer bold content must use bold_color with BOLD modifier"
        );
    }

    #[test]
    fn plain_bold_still_works_after_refactor() {
        // Regression: **text** must still produce bold-only spans.
        let text = "**bold**";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // Content at 2..6, no ITALIC.
        let content = spans.iter().find(|s| s.char_start == 2 && s.char_end == 6);
        assert!(content.is_some(), "bold content span at 2..6 must exist");
        let content = content.unwrap();
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "**bold** must have BOLD"
        );
        assert!(
            !content.style.add_modifier.contains(Modifier::ITALIC),
            "**bold** must NOT have ITALIC"
        );
    }

    #[test]
    fn plain_italic_still_works_after_refactor() {
        // Regression: *text* must still produce italic-only spans.
        let text = "*italic*";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let content = spans.iter().find(|s| s.char_start == 1 && s.char_end == 7);
        assert!(content.is_some(), "italic content span at 1..7 must exist");
        let content = content.unwrap();
        assert!(
            content.style.add_modifier.contains(Modifier::ITALIC),
            "*italic* must have ITALIC"
        );
        assert!(
            !content.style.add_modifier.contains(Modifier::BOLD),
            "*italic* must NOT have BOLD"
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

    /// Regression test for #133: blank lines inside a fenced code block must
    /// have `full_line_bg = Some(fenced_bg)` in the decoration map.  Before
    /// the renderer fix they had the span but the renderer's row_spans filter
    /// dropped it (char_end == 0 fails `char_start < char_end`).
    #[test]
    fn fenced_code_blank_content_line_has_fenced_bg() {
        // Line 0: before, 1: ```, 2: code, 3: <blank>, 4: more code, 5: ```, 6: after
        let text = "before\n```\ncode\n\nmore code\n```\nafter";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let blank_line = map
            .get(&3)
            .expect("blank content line (line 3) must have spans");
        assert!(
            blank_line
                .iter()
                .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
            "blank line inside fenced block must carry full_line_bg = fenced_bg; \
             got: {blank_line:?}"
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

    #[test]
    fn blockquote_plain_text_has_only_indicator_span() {
        // With no inline markup, only the indicator span (0..1) should exist.
        // Undecorated text gets blockquote_color from the renderer's default style.
        let text = "> just text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert_eq!(
            spans.len(),
            1,
            "plain blockquote should have exactly one span (the indicator); got {spans:?}"
        );
        assert_eq!(spans[0].char_start, 0);
        assert_eq!(spans[0].char_end, 1);
    }

    /// Regression test for #120: bold inside a blockquote must emit a span with
    /// `bold_color + BOLD`.  Before the fix, the wide content span (char 1..N) that
    /// blockquotes used to emit blocked `emit_content_around_existing`, preventing
    /// the bold content span from being placed at all.
    #[test]
    fn blockquote_bold_emits_bold_color_span() {
        let text = "> **bold**";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.bold_color)
                && s.style.add_modifier.contains(Modifier::BOLD)),
            "bold inside blockquote must produce a span with bold_color + BOLD"
        );
    }

    /// Regression test for #120: `add_modifier_to_existing` used to apply BOLD to
    /// the wide content span (char 1..N), making the **entire** blockquote line render
    /// bold — including text before and after the `**` delimiters.
    #[test]
    fn blockquote_bold_does_not_bleed_full_line() {
        let text = "> text **bold** text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        // No span rooted at char 1 (the old wide content span start) should carry BOLD.
        assert!(
            !spans
                .iter()
                .any(|s| s.char_start == 1 && s.style.add_modifier.contains(Modifier::BOLD)),
            "BOLD must not bleed onto a span starting at char 1 (regression #120)"
        );
    }

    /// Same regression guard as above, for italic.
    #[test]
    fn blockquote_italic_does_not_bleed_full_line() {
        let text = "> text *italic* text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans
                .iter()
                .any(|s| s.char_start == 1 && s.style.add_modifier.contains(Modifier::ITALIC)),
            "ITALIC must not bleed onto a span starting at char 1 (regression #120)"
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

    /// Regression test for FEEDBACK-2 §1.1:
    /// Inline bold inside a table header must not be swallowed by the wide
    /// header span.  Before the fix, `Start(TableHead)` emitted a single
    /// `add_byte_range_span` which — being sorted first by char_start=0 in
    /// `split_into_spans` — consumed the entire row and clipped all inner spans.
    ///
    /// After the fix, the wide span is emitted on `End(TableHead)` via
    /// `emit_content_around_existing`, so bold/italic spans placed by earlier
    /// inner events survive unchanged.
    #[test]
    fn table_header_inline_bold_not_swallowed() {
        let theme = make_theme();
        // Header has an explicit **Bold** cell — the bold content span must
        // appear in the decoration map with the bold_color (not just accent).
        let text = "| **Bold** | Plain |\n| --- | --- |\n| a | b |";
        let map = build_map(text, &theme, true);
        let line0 = map.get(&0).expect("line 0 should have spans");
        // The bold content span uses theme.bold_color with BOLD modifier.
        let has_bold_span = line0.iter().any(|s| {
            s.style.fg == Some(theme.bold_color) && s.style.add_modifier.contains(Modifier::BOLD)
        });
        assert!(
            has_bold_span,
            "bold inside table header must produce a bold_color span, not be swallowed"
        );
    }

    /// Italic inside a table header must also survive.
    #[test]
    fn table_header_inline_italic_not_swallowed() {
        let theme = make_theme();
        let text = "| *Italic* | Plain |\n| --- | --- |\n| a | b |";
        let map = build_map(text, &theme, true);
        let line0 = map.get(&0).expect("line 0 should have spans");
        let has_italic_span = line0.iter().any(|s| s.style.fg == Some(theme.italic_color));
        assert!(
            has_italic_span,
            "italic inside table header must produce an italic_color span, not be swallowed"
        );
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
        assert!(
            has_content,
            "H1 should have a content span starting at char 2"
        );
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

    // ---- Blockquote span structure (post-#120 fix) ----
    //
    // The wide content span (char 1..N) was removed so inline decorations (bold,
    // italic, etc.) can emit correctly inside blockquotes.  Only the indicator
    // span (0..1) is emitted; the renderer uses its `is_blockquote` flag to apply
    // `blockquote_color` as the default fg for undecorated text on the line.

    #[test]
    fn blockquote_only_indicator_carries_is_blockquote() {
        // After the #120 fix, the indicator (0..1) is the only blockquote span.
        // No wide content span at char_start >= 1 should exist.
        let text = "> quoted text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.is_blockquote && s.char_start >= 1),
            "no wide content span (char_start >= 1, is_blockquote) should exist after #120 fix"
        );
    }

    // ---- Continuation indent (#39 blockquote, #59 list) ----

    #[test]
    fn blockquote_indicator_span_has_continuation_indent_2() {
        let text = "> quoted text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.is_blockquote && s.char_start == 0 && s.continuation_indent == 2),
            "blockquote indicator span must have continuation_indent=2"
        );
    }

    #[test]
    fn unordered_list_bullet_has_continuation_indent_2() {
        // "- item": bullet at char 0, bullet_end = 1, ci = bullet_end + 1 = 2
        let text = "- item text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 0 && s.char_end == 1 && s.continuation_indent == 2),
            "unordered bullet span must have continuation_indent=2"
        );
    }

    #[test]
    fn ordered_list_bullet_has_continuation_indent_3() {
        // "1. item": bullet at char 0, finds '.', bullet_end = 2, ci = 3
        let text = "1. item text";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 0 && s.continuation_indent == 3),
            "ordered bullet span (1.) must have continuation_indent=3"
        );
    }

    #[test]
    fn todo_unchecked_continuation_indent_is_6() {
        // "- [ ] todo": marker `[` is at char 2, so task_ci = 2 + 4 = 6.
        // Continuation rows align with text start after the full `- [ ] ` glyph.
        let text = "- [ ] todo item";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let max_ci = spans
            .iter()
            .map(|s| s.continuation_indent)
            .max()
            .unwrap_or(0);
        assert_eq!(
            max_ci, 6,
            "unchecked todo item must have max continuation_indent=6 (past `- [ ] `)"
        );
    }

    #[test]
    fn todo_checked_continuation_indent_is_6() {
        // "- [x] done": same marker position as unchecked → task_ci = 6.
        let text = "- [x] done item";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let max_ci = spans
            .iter()
            .map(|s| s.continuation_indent)
            .max()
            .unwrap_or(0);
        assert_eq!(
            max_ci, 6,
            "checked todo item must have max continuation_indent=6 (past `- [x] `)"
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

    // ---- Range diagnostics ----

    #[test]
    fn debug_pulldown_bold_italic_byte_ranges() {
        use pulldown_cmark::{Event, Options, Parser, Tag};

        // (text, emph_start, strong_start, strong_end, emph_end)
        // For `***text***` Emphasis wraps Strong, adjacent:
        //   emph: 0..17, strong: 1..16 → emph.start+1==strong.start, strong.end+1==emph.end ✓
        // For `**_text_**` Strong wraps Emphasis:
        //   strong: 0..8, emph: 2..6 → strong.start+2==emph.start, emph.end+2==strong.end ✓
        // For `*x and **y***` non-adjacent (italic wrapping bold):
        //   emph: 0..N, strong: offset..N-1 → emph.start+1 ≠ strong.start ✓ (not adjacent)
        let cases: &[(&str, usize, usize, usize, usize)] = &[
            ("***bold-italic***", 0, 1, 16, 17),
            ("**_bold-italic_**", 0, 0, 15, 15), // placeholder, overwritten below
        ];
        let _ = cases; // will not use the table form; just print and assert non-adjacent

        let options =
            Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;

        // Verify the adjacent case gives correct ranges.
        {
            let text = "***bold-italic***";
            let mut emph = (0usize, 0usize);
            let mut strong = (0usize, 0usize);
            for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
                match event {
                    Event::Start(Tag::Emphasis) => emph = (range.start, range.end),
                    Event::Start(Tag::Strong) => strong = (range.start, range.end),
                    _ => {}
                }
            }
            assert_eq!(emph, (0, 17), "Emphasis range for ***bold-italic***");
            assert_eq!(strong, (1, 16), "Strong range for ***bold-italic***");
            // Adjacency: emph.start+1 == strong.start AND strong.end+1 == emph.end
            assert_eq!(emph.0 + 1, strong.0, "adjacent: emph.start+1==strong.start");
            assert_eq!(strong.1 + 1, emph.1, "adjacent: strong.end+1==emph.end");
        }

        // Verify non-adjacent case does NOT satisfy the adjacency check.
        {
            let text = "*italic and **nested bold***";
            let mut emph = (0usize, 0usize);
            let mut strong = (0usize, 0usize);
            for (event, range) in Parser::new_ext(text, options).into_offset_iter() {
                match event {
                    Event::Start(Tag::Emphasis) => emph = (range.start, range.end),
                    Event::Start(Tag::Strong) => strong = (range.start, range.end),
                    _ => {}
                }
            }
            let adjacent = emph.0 + 1 == strong.0 && strong.1 + 1 == emph.1;
            assert!(
                !adjacent,
                "non-adjacent nesting must NOT satisfy adjacency check; \
                 emph={:?}, strong={:?}",
                emph, strong
            );
        }
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

    // ---- Direct helper tests: add_modifier_to_existing ----
    //
    // These call the private helper directly to verify the overlap filter on line 37:
    //   `if span.char_end > range_start && span.char_start < range_end`

    // Kills: mod.rs:37:30 replace > with >=
    // With >=: char_end=10 >= range_start=10 → span incorrectly gets the modifier.
    #[test]
    fn add_modifier_not_applied_to_span_ending_at_range_start() {
        use super::spans::{make_span, push_span};
        use ratatui::style::Style;
        let mut map = DecorationMap::default();
        push_span(&mut map, 0, make_span(5, 10, Style::default()));
        add_modifier_to_existing(&mut map, 0, 10, 15, Modifier::BOLD);
        let span = &map[&0][0];
        assert!(
            !span.style.add_modifier.contains(Modifier::BOLD),
            "span [5,10) ending exactly at range_start=10 must not receive BOLD"
        );
    }

    // Kills: mod.rs:37:63 replace < with <=
    // With <=: char_start=15 <= range_end=15 → span incorrectly gets the modifier.
    #[test]
    fn add_modifier_not_applied_to_span_starting_at_range_end() {
        use super::spans::{make_span, push_span};
        use ratatui::style::Style;
        let mut map = DecorationMap::default();
        push_span(&mut map, 0, make_span(15, 20, Style::default()));
        add_modifier_to_existing(&mut map, 0, 10, 15, Modifier::BOLD);
        let span = &map[&0][0];
        assert!(
            !span.style.add_modifier.contains(Modifier::BOLD),
            "span [15,20) starting exactly at range_end=15 must not receive BOLD"
        );
    }

    // Kills: mod.rs:37:44 replace && with ||
    // With ||: char_start=0 < range_end=15 is TRUE → || short-circuits to true → span
    // incorrectly gets the modifier even though it ends before the range.
    #[test]
    fn add_modifier_not_applied_to_span_entirely_before_range() {
        use super::spans::{make_span, push_span};
        use ratatui::style::Style;
        let mut map = DecorationMap::default();
        push_span(&mut map, 0, make_span(0, 5, Style::default()));
        add_modifier_to_existing(&mut map, 0, 10, 15, Modifier::BOLD);
        let span = &map[&0][0];
        assert!(
            !span.style.add_modifier.contains(Modifier::BOLD),
            "span [0,5) entirely before range [10,15) must not receive BOLD"
        );
    }

    // ---- Direct helper tests: emit_content_around_existing ----
    //
    // Verifies the gap-filling logic in lines 69–89 of mod.rs directly.

    // Kills: mod.rs:69:40 (>→==, >→<, >→>=), 69:70 (<→==, <→>, <→<=),
    //        79:16 (<→==, meaning the gap-emit condition fails for real gaps),
    //        82:22 (>→== and >→< both fail to advance pos, causing overlap).
    #[test]
    fn emit_content_skips_blocked_interior() {
        use super::spans::{make_span, push_span};
        use ratatui::style::{Color, Style};
        let mut map = DecorationMap::default();
        let block_style = Style::default().fg(Color::Red);
        let fill_style = Style::default().fg(Color::Blue);
        // Existing span at [4,7) — must block the fill.
        push_span(&mut map, 0, make_span(4, 7, block_style));
        // Emit content around existing spans in range [2,10).
        emit_content_around_existing(&mut map, 0, 2, 10, fill_style);
        let spans = map.get(&0).expect("line 0 must have spans");
        // Leading gap [2,4) must be filled.
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 2 && s.char_end == 4 && s.style.fg == Some(Color::Blue)),
            "gap [2,4) before the block must be filled with fill_style; spans={:?}",
            spans
        );
        // Trailing gap [7,10) must be filled.
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 7 && s.char_end == 10 && s.style.fg == Some(Color::Blue)),
            "gap [7,10) after the block must be filled with fill_style; spans={:?}",
            spans
        );
        // No fill span must overlap the blocked region [4,7).
        assert!(
            !spans
                .iter()
                .any(|s| s.char_start < 7 && s.char_end > 4 && s.style.fg == Some(Color::Blue)),
            "no fill_style span must overlap the blocked region [4,7); spans={:?}",
            spans
        );
    }

    // Kills: mod.rs:79:16 replace < with <= (emits a zero-width [5,5) span when range
    //        starts at the block boundary).
    #[test]
    fn emit_content_no_leading_zero_width_span() {
        use super::spans::{make_span, push_span};
        use ratatui::style::{Color, Style};
        let mut map = DecorationMap::default();
        push_span(
            &mut map,
            0,
            make_span(5, 8, Style::default().fg(Color::Red)),
        );
        // Range starts exactly at the block start: no leading gap should exist.
        emit_content_around_existing(&mut map, 0, 5, 10, Style::default().fg(Color::Blue));
        let spans = map.get(&0).expect("line 0 must have spans");
        // Trailing gap [8,10) must be filled.
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 8 && s.char_end == 10 && s.style.fg == Some(Color::Blue)),
            "trailing gap [8,10) must be filled; spans={:?}",
            spans
        );
        // No zero-width span at the range/block start.
        assert!(
            !spans.iter().any(|s| s.char_start == 5 && s.char_end == 5),
            "no zero-width span [5,5) when range_start == block_start; spans={:?}",
            spans
        );
    }

    // Kills: mod.rs:86:12 replace < with <= (emits a zero-width [10,10) trailing span
    //        when pos == range_end after the last block).
    #[test]
    fn emit_content_no_trailing_zero_width_span() {
        use super::spans::{make_span, push_span};
        use ratatui::style::{Color, Style};
        let mut map = DecorationMap::default();
        // Block extends to range_end; after the loop pos == range_end → no tail.
        push_span(
            &mut map,
            0,
            make_span(5, 10, Style::default().fg(Color::Red)),
        );
        emit_content_around_existing(&mut map, 0, 2, 10, Style::default().fg(Color::Blue));
        let spans = map.get(&0).expect("line 0 must have spans");
        // Leading gap [2,5) must be filled.
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 2 && s.char_end == 5 && s.style.fg == Some(Color::Blue)),
            "leading gap [2,5) must be filled; spans={:?}",
            spans
        );
        // No zero-width span anywhere.
        assert!(
            !spans.iter().any(|s| s.char_start == s.char_end),
            "no zero-width span must be emitted; spans={:?}",
            spans
        );
    }

    // Kills: mod.rs:69:54 replace && with || in emit_content_around_existing.
    // With ||: a span at [12,15) satisfies char_end=15 > range_start=2 even though it
    // starts past range_end=10.  It gets included in 'blocked', clamped to [12,10), and
    // the gap loop emits [2,12) instead of [2,10).
    #[test]
    fn emit_content_does_not_block_external_span_past_range_end() {
        use super::spans::{make_span, push_span};
        use ratatui::style::{Color, Style};
        let mut map = DecorationMap::default();
        // Span entirely outside the emit range [2,10), past its end.
        push_span(
            &mut map,
            0,
            make_span(12, 15, Style::default().fg(Color::Red)),
        );
        emit_content_around_existing(&mut map, 0, 2, 10, Style::default().fg(Color::Blue));
        let spans = map.get(&0).expect("line 0 must have spans");
        // Entire range [2,10) must be filled as one unbroken span.
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 2 && s.char_end == 10 && s.style.fg == Some(Color::Blue)),
            "range [2,10) must be emitted exactly; spans={:?}",
            spans
        );
        // No fill span must bleed past range_end=10.
        assert!(
            !spans
                .iter()
                .any(|s| s.char_end > 10 && s.style.fg == Some(Color::Blue)),
            "no fill span must extend past range_end=10; spans={:?}",
            spans
        );
    }

    // ---- Direct helper tests: line_char_len ----

    // Kills: spans.rs:32:30 replace < with >
    // With >: `line_idx + 1 > line_starts.len()` is never true for valid line indices,
    // so the else branch always fires → le = text.len() → counts chars including \n.
    #[test]
    fn line_char_len_non_last_line_excludes_newline() {
        use super::spans::line_char_len;
        let text = "hello\nworld";
        let starts = line_start_bytes(text); // [0, 6]
        // Line 0 = "hello" (5 chars); mutation returns 11 (whole text).
        assert_eq!(
            line_char_len(&starts, text, 0),
            5,
            "line 0 must be 5 chars, not 11"
        );
        // Last line still correct (exercises the else branch in both variants).
        assert_eq!(line_char_len(&starts, text, 1), 5, "line 1 must be 5 chars");
    }

    // ---- Direct helper tests: add_byte_range_span ----

    // Kills: spans.rs:77:31 (==→!= swaps start_char assignment between lines),
    //        spans.rs:78:29 (==→!= swaps end_char assignment between lines),
    //        spans.rs:79:32 (+→- or +→* makes end exclusive off-by-1 or off-by-2),
    //        spans.rs:88 (delete char_start → defaults to 0 on first line),
    //        spans.rs:89 (delete char_end → defaults to 0 on first line).
    #[test]
    fn add_byte_range_span_multiline_correct_boundaries() {
        use super::spans::{SpanParams, add_byte_range_span, line_start_bytes as lsb};
        use ratatui::style::Style;
        // text: "abcde\nfghij" — line 0 = "abcde" (5 chars), line 1 = "fghij" (5 chars)
        let text = "abcde\nfghij";
        let starts = lsb(text); // [0, 6]
        let mut map = DecorationMap::default();
        // byte 2 = 'c' on line 0 (char 2); byte 8 = 'i' on line 1 (char 2, exclusive end = 9)
        add_byte_range_span(
            &mut map,
            &starts,
            text,
            2,
            9,
            SpanParams {
                style: Style::default(),
                full_line_bg: None,
                is_blockquote: false,
            },
        );
        // Line 0: c_start = start_char = 2, c_end = line_char_len(0) = 5.
        let l0 = map.get(&0).expect("line 0 must have a span");
        assert!(
            l0.iter().any(|s| s.char_start == 2 && s.char_end == 5),
            "line 0 span must be [2,5); got: {:?}",
            l0
        );
        // Line 1: c_start = 0, c_end = end_char_inclusive+1 = 3.
        let l1 = map.get(&1).expect("line 1 must have a span");
        assert!(
            l1.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "line 1 span must be [0,3); got: {:?}",
            l1
        );
    }

    // Kills: spans.rs:83:39 replace + with * (c_end.max(c_start+1) → c_end.max(c_start*1) = c_end.max(c_start)).
    // An empty intermediate line gets line_char_len=0; the max ensures at least 1-char width.
    // With *: max(0, 0) = 0 → zero-width span for the empty line.
    #[test]
    fn add_byte_range_span_empty_intermediate_line_gets_min_width() {
        use super::spans::{SpanParams, add_byte_range_span, line_start_bytes as lsb};
        use ratatui::style::Style;
        // text: "ab\n\ncd" — line 0="ab", line 1="" (empty), line 2="cd"
        let text = "ab\n\ncd";
        let starts = lsb(text); // [0, 3, 4]
        let mut map = DecorationMap::default();
        add_byte_range_span(
            &mut map,
            &starts,
            text,
            0,
            6,
            SpanParams {
                style: Style::default(),
                full_line_bg: None,
                is_blockquote: false,
            },
        );
        // Empty line 1 must get char_end = 1 (min-width clamp), not 0.
        let l1 = map.get(&1).expect("empty line 1 must have a span");
        assert!(
            l1.iter().any(|s| s.char_end >= 1),
            "empty intermediate line must get char_end >= 1 (min-width clamp); got: {:?}",
            l1
        );
        assert!(
            !l1.iter().any(|s| s.char_start == s.char_end),
            "empty intermediate line must not produce a zero-width span; got: {:?}",
            l1
        );
    }

    // Kills: spans.rs:91 `delete field is_blockquote` (uses Default=false instead of params)
    //        spans.rs:92 `delete field full_line_bg`  (uses Default=None instead of params)
    // These fields are propagated through SpanParams → StyledSpan.  Since all production
    // call sites currently pass false/None, we need an explicit test with non-default values
    // to distinguish propagation from defaulting.
    #[test]
    fn add_byte_range_span_propagates_span_params_fields() {
        use super::spans::{SpanParams, add_byte_range_span, line_start_bytes as lsb};
        use ratatui::style::{Color, Style};
        let text = "hello";
        let starts = lsb(text);
        let mut map = DecorationMap::default();
        add_byte_range_span(
            &mut map,
            &starts,
            text,
            0,
            5,
            SpanParams {
                style: Style::default(),
                full_line_bg: Some(Color::Red),
                is_blockquote: true,
            },
        );
        let spans = map.get(&0).expect("line 0 must have a span");
        assert!(
            spans.iter().any(|s| s.is_blockquote),
            "is_blockquote must propagate from SpanParams to StyledSpan; got: {:?}",
            spans
        );
        assert!(
            spans.iter().any(|s| s.full_line_bg == Some(Color::Red)),
            "full_line_bg must propagate from SpanParams to StyledSpan; got: {:?}",
            spans
        );
    }

    // ── block_highlights_to_decoration_map ───────────────────────────────────

    use crate::highlighting::HlSpan;

    #[test]
    fn bh_to_deco_empty_highlights_gives_empty_map() {
        let hl: crate::highlighting::BlockHighlights = vec![];
        let map = block_highlights_to_decoration_map(&hl, 0);
        assert!(
            map.is_empty(),
            "empty BlockHighlights must produce empty map"
        );
    }

    #[test]
    fn bh_to_deco_single_line_single_span() {
        let hl = vec![vec![HlSpan {
            char_start: 0,
            char_end: 3,
            fg: Color::Rgb(255, 0, 0),
        }]];
        let map = block_highlights_to_decoration_map(&hl, 0);
        let spans = map.get(&0).expect("line 0 must be present");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].char_start, 0);
        assert_eq!(spans[0].char_end, 3);
    }

    #[test]
    fn bh_to_deco_fg_colour_preserved() {
        let fg = Color::Rgb(100, 200, 50);
        let hl = vec![vec![HlSpan {
            char_start: 0,
            char_end: 5,
            fg,
        }]];
        let map = block_highlights_to_decoration_map(&hl, 0);
        let spans = map.get(&0).unwrap();
        assert_eq!(
            spans[0].style.fg,
            Some(fg),
            "fg colour must be preserved in StyledSpan"
        );
    }

    #[test]
    fn bh_to_deco_multiline_maps_correct_line_indices() {
        let hl = vec![
            vec![HlSpan {
                char_start: 0,
                char_end: 2,
                fg: Color::Rgb(1, 2, 3),
            }],
            vec![HlSpan {
                char_start: 0,
                char_end: 4,
                fg: Color::Rgb(4, 5, 6),
            }],
        ];
        let map = block_highlights_to_decoration_map(&hl, 0);
        assert!(map.contains_key(&0), "line 0 must be present");
        assert!(map.contains_key(&1), "line 1 must be present");
        assert_eq!(map.get(&0).unwrap()[0].char_end, 2);
        assert_eq!(map.get(&1).unwrap()[0].char_end, 4);
    }

    #[test]
    fn bh_to_deco_line_offset_applied() {
        // With line_offset=5, the first hl line maps to DecorationMap key 5.
        let hl = vec![vec![HlSpan {
            char_start: 0,
            char_end: 1,
            fg: Color::Rgb(0, 0, 0),
        }]];
        let map = block_highlights_to_decoration_map(&hl, 5);
        assert!(
            map.contains_key(&5),
            "line_offset must shift line index to 5"
        );
        assert!(
            !map.contains_key(&0),
            "line 0 must not be present when offset=5"
        );
    }

    #[test]
    fn bh_to_deco_markdown_fields_are_zero() {
        // Markdown-specific fields must all be at their zero/false defaults.
        let hl = vec![vec![HlSpan {
            char_start: 0,
            char_end: 3,
            fg: Color::Rgb(1, 1, 1),
        }]];
        let map = block_highlights_to_decoration_map(&hl, 0);
        let span = &map.get(&0).unwrap()[0];
        assert!(!span.is_blockquote, "is_blockquote must be false");
        assert_eq!(span.continuation_indent, 0, "continuation_indent must be 0");
        assert!(span.full_line_bg.is_none(), "full_line_bg must be None");
        assert!(span.border_bottom.is_none(), "border_bottom must be None");
        assert!(!span.is_rule, "is_rule must be false");
    }

    #[test]
    fn bh_to_deco_empty_inner_lines_not_inserted() {
        // Lines with no spans should not create entries in the map.
        let hl = vec![
            vec![],
            vec![HlSpan {
                char_start: 0,
                char_end: 2,
                fg: Color::Rgb(0, 0, 0),
            }],
        ];
        let map = block_highlights_to_decoration_map(&hl, 0);
        assert!(
            !map.contains_key(&0),
            "empty span list must not create a map entry"
        );
        assert!(
            map.contains_key(&1),
            "non-empty span list must create a map entry"
        );
    }
}
