use std::collections::HashSet;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

use crate::config::Theme;
use crate::decoration::frontmatter::{apply_frontmatter_spans, detect_frontmatter};
use crate::decoration::highlight::apply_highlight_spans;
use crate::decoration::spans::line_start_bytes;
use crate::decoration::types::DecorationMap;
use crate::highlighting::HighlightCache;

mod blocks;
mod fenced;
mod headings;
mod inline;
mod misc;
mod tables;

// ---------------------------------------------------------------------------
// BuildState
// ---------------------------------------------------------------------------

/// All mutable state that crosses event boundaries during a single
/// `build_decoration_map` pass.  Handler modules receive `&mut BuildState`
/// so they can read inputs and write outputs without any shared globals.
pub(crate) struct BuildState<'a> {
    // --- inputs (immutable) ---
    pub text: &'a str,
    pub theme: &'a Theme,
    pub italic_support: bool,
    pub highlight_cache: Option<&'a HighlightCache>,
    pub line_starts: Vec<usize>,

    // --- outputs (accumulate as events fire) ---
    pub map: DecorationMap,
    pub word_count: usize,

    // --- per-event-sequence state ---
    pub in_ordered_list: bool,
    pub in_strong: Option<std::ops::Range<usize>>,
    pub in_emphasis: Option<std::ops::Range<usize>>,
    pub in_table_head: Option<std::ops::Range<usize>>,
    pub code_block_lines: HashSet<usize>,
}

impl<'a> BuildState<'a> {
    fn new(
        text: &'a str,
        theme: &'a Theme,
        italic_support: bool,
        highlight_cache: Option<&'a HighlightCache>,
    ) -> Self {
        Self {
            text,
            theme,
            italic_support,
            highlight_cache,
            line_starts: line_start_bytes(text),
            map: DecorationMap::default(),
            word_count: 0,
            in_ordered_list: false,
            in_strong: None,
            in_emphasis: None,
            in_table_head: None,
            code_block_lines: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build the full decoration map from `text` and simultaneously count words.
///
/// Returns `(DecorationMap, word_count)` so callers avoid a second parser pass.
/// Pure function — no terminal or UI side effects. This is the v1.5 migration seam:
/// when moving to a background thread, only the call site changes.
///
/// `highlight_cache` is optional: pass `Some(&cache)` to enable syntect syntax
/// highlighting for fenced code blocks, or `None` to disable it (fenced_bg-only).
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
    highlight_cache: Option<&HighlightCache>,
) -> (DecorationMap, usize) {
    let mut s = BuildState::new(text, theme, italic_support, highlight_cache);

    let parser = Parser::new_ext(text, fenced::parser_options()).into_offset_iter();

    for (event, range) in parser {
        match event {
            // ---- a. Headings ----
            Event::Start(Tag::Heading { level, .. }) => {
                headings::on_start(&mut s, level, range);
            }

            // ---- b. Bold ----
            Event::Start(Tag::Strong) => inline::on_strong_start(&mut s, range),
            Event::End(TagEnd::Strong) => inline::on_strong_end(&mut s, range),

            // ---- c. Italic ----
            Event::Start(Tag::Emphasis) => inline::on_emphasis_start(&mut s, range),
            Event::End(TagEnd::Emphasis) => inline::on_emphasis_end(&mut s, range),

            // ---- d. Inline code ----
            Event::Code(ref val) => inline::on_code(&mut s, val, range),

            // ---- e. Fenced code blocks ----
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                fenced::on_start(&mut s, lang, range);
            }

            // ---- f. Blockquotes ----
            Event::Start(Tag::BlockQuote(_)) => blocks::on_blockquote_start(&mut s, range),

            // ---- g. Links ----
            Event::Start(Tag::Link { .. }) => misc::on_link(&mut s, range),

            // ---- h. List items ----
            Event::Start(Tag::List(kind)) => blocks::on_list_start(&mut s, kind),
            Event::End(TagEnd::List(_)) => blocks::on_list_end(&mut s),
            Event::Start(Tag::Item) => blocks::on_item_start(&mut s, range),

            // ---- i. Task-list markers ----
            Event::TaskListMarker(checked) => blocks::on_task_marker(&mut s, checked, range),

            // ---- j. Tables ----
            Event::Start(Tag::Table(_)) => tables::on_table_start(&mut s, range),
            Event::Start(Tag::TableHead) => tables::on_table_head_start(&mut s, range),
            Event::End(TagEnd::TableHead) => tables::on_table_head_end(&mut s, range),

            // ---- k. Strikethrough ----
            Event::Start(Tag::Strikethrough) => misc::on_strikethrough(&mut s, range),

            // ---- l. Horizontal rule ----
            Event::Rule => misc::on_rule(&mut s, range),

            // ---- m. Word count ----
            Event::Text(ref val) => misc::on_text(&mut s, val, range),

            _ => {}
        }
    }

    // Frontmatter post-pass: detect and restyle YAML/TOML frontmatter blocks.
    // Must run after the markdown parser so its rule-spans on the `---` delimiter
    // lines can be removed and replaced with frontmatter-specific styling.
    if let Some(end_line) = detect_frontmatter(text) {
        apply_frontmatter_spans(&mut s.map, &s.line_starts, text, end_line, s.theme);
    }

    // ==highlight== post-pass: scan non-code, non-frontmatter lines for ==...==.
    apply_highlight_spans(&mut s.map, text, &s.line_starts, &s.code_block_lines, s.theme);

    (s.map, s.word_count)
}
