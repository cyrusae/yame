use ratatui::style::{Color, Modifier};
use yame::config::Theme;
use yame::decoration::build_decoration_map;
use yame::highlighting::HighlightCache;

/// Full decoration pass against the fixture file.
/// Confirms headings, bold, blockquotes, and word count all survive the pipeline.
#[test]
fn fixture_decoration_roundtrip() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, word_count) = build_decoration_map(text, &theme, true, None);

    // Line 0 ("# Heading One") must have a full_line_bg highlight.
    assert!(
        map[&0].iter().any(|s| s.full_line_bg.is_some()),
        "H1 on line 0 must have full_line_bg"
    );

    // At least one span anywhere in the file must be BOLD.
    assert!(
        map.values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD)),
        "expected at least one BOLD span in the fixture"
    );

    // At least one span must be a blockquote.
    assert!(
        map.values().flatten().any(|s| s.is_blockquote),
        "expected at least one blockquote span in the fixture"
    );

    // The fixture has more than 100 words.
    assert!(
        word_count > 100,
        "word count was {word_count} — expected >100"
    );
}

/// Strikethrough syntax (~~text~~) must produce a CROSSED_OUT span.
#[test]
fn fixture_has_strikethrough() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::CROSSED_OUT)),
        "expected at least one CROSSED_OUT span (strikethrough) in the fixture"
    );
}

/// Link text must be underlined per the decoration spec.
#[test]
fn fixture_has_link_underline() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::UNDERLINED)),
        "expected at least one UNDERLINED span (link text) in the fixture"
    );
}

/// Horizontal rules (`---`) must set is_rule on their span.
#[test]
fn fixture_has_horizontal_rule() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values().flatten().any(|s| s.is_rule),
        "expected at least one is_rule span (---) in the fixture"
    );
}

/// Fenced code block content lines must receive the fenced_bg full-line background.
#[test]
fn fixture_has_fenced_code_bg() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
        "expected at least one span with fenced_bg full-line background"
    );
}

/// The fixture has H1–H4; all should carry heading_bg on at least one span.
/// Checks that multiple distinct heading lines are decorated (not just H1).
#[test]
fn fixture_multiple_heading_levels_decorated() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    let heading_lines = map
        .values()
        .filter(|spans| {
            spans
                .iter()
                .any(|s| s.full_line_bg == Some(theme.heading_bg))
        })
        .count();
    assert!(
        heading_lines >= 4,
        "expected ≥4 heading lines with heading_bg, got {heading_lines}"
    );
}

// ---------------------------------------------------------------------------
// Continuation indent (regression guard for the list-item clipping bug)
// ---------------------------------------------------------------------------

/// Unordered bullet spans must carry continuation_indent >= 2.
///
/// This is the integration-level guard for the list-item soft-wrap clipping
/// bug: if continuation_indent is zero, wrap_line_indented degenerates to
/// wrap_line and continuation rows overflow the right edge.
#[test]
fn fixture_unordered_list_has_continuation_indent() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    // Bullet spans anchor at char_start == 0, char_end == 1 ("- ").
    assert!(
        map.values()
            .flatten()
            .any(|s| s.char_start == 0 && s.char_end == 1 && s.continuation_indent >= 2),
        "expected at least one unordered bullet span with continuation_indent >= 2"
    );
}

/// Blockquote spans must carry continuation_indent == 2.
///
/// Guards that the `> ` prefix (2 columns) is accounted for when wrapping
/// long blockquote lines, preventing right-edge overflow on continuation rows.
#[test]
fn fixture_blockquote_has_continuation_indent() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.is_blockquote && s.continuation_indent == 2),
        "expected at least one blockquote span with continuation_indent == 2"
    );
}

/// Task-list items must produce a maximum continuation_indent >= 6.
///
/// The task checkbox prefix `- [ ] ` / `- [x] ` is 6 columns wide; the
/// continuation indent must reach at least that far so wrapped task items
/// align correctly under the text, not under the dash.
#[test]
fn fixture_task_list_has_max_continuation_indent() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    let max_ci = map
        .values()
        .flatten()
        .map(|s| s.continuation_indent)
        .max()
        .unwrap_or(0);
    assert!(
        max_ci >= 6,
        "expected max continuation_indent >= 6 for task-list items, got {max_ci}"
    );
}

// ---------------------------------------------------------------------------
// Bold + italic combined
// ---------------------------------------------------------------------------

/// `***text***` must produce a span with both BOLD and ITALIC modifiers.
///
/// The fixture contains "***Bold and italic combined with triple asterisks.***";
/// pulldown-cmark emits this as a single strong+emphasis range that
/// emit_bold_italic_spans handles by setting both modifiers on one span.
#[test]
fn fixture_has_bold_italic_combined() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values().flatten().any(|s| {
            s.style.add_modifier.contains(Modifier::BOLD)
                && s.style.add_modifier.contains(Modifier::ITALIC)
        }),
        "expected at least one span with both BOLD and ITALIC modifiers (***text***)"
    );
}

// ---------------------------------------------------------------------------
// Custom TOML theme propagation (end-to-end config → theme → decoration)
// ---------------------------------------------------------------------------

/// A custom palette TOML loads correctly and its accent propagates to every
/// accent-derived theme token, which the full decoration pipeline then uses.
///
/// The fixture `tests/fixtures/custom_theme.toml` overrides only `accent`.
/// We verify:
///   1. The file parses without warnings.
///   2. `heading_bg` (derived as blend(accent, bg, 0.15)) differs from the
///      default (proving the new accent reached theme derivation).
///   3. The decoration map contains at least one span whose `full_line_bg`
///      equals the custom `heading_bg` (proving the theme reached rendering).
#[test]
fn fixture_custom_toml_theme_propagates() {
    let text = include_str!("fixtures/sample.md");
    let toml_str = include_str!("fixtures/custom_theme.toml");

    let cfg: yame::config::Config =
        toml::from_str(toml_str).expect("custom_theme.toml must parse as a valid Config");
    let mut warnings = Vec::new();
    let custom_theme = Theme::from_config(&cfg.palette, &cfg.theme, &cfg.headings, &mut warnings);
    assert!(
        warnings.is_empty(),
        "unexpected parse warnings from custom_theme.toml: {warnings:?}"
    );

    // heading_bg is accent-derived; a red accent must move it away from the
    // default purple-tinted value.
    let default_theme = Theme::default_theme();
    assert_ne!(
        custom_theme.heading_bg, default_theme.heading_bg,
        "custom accent (#ff0000) must change heading_bg away from the Catppuccin default"
    );

    // Run the full decoration pipeline with the custom theme and confirm that
    // at least one heading span carries the custom heading_bg.
    let (map, _) = build_decoration_map(text, &custom_theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.full_line_bg == Some(custom_theme.heading_bg)),
        "expected at least one heading span with the custom heading_bg in the decoration map"
    );
}

// ---------------------------------------------------------------------------
// Syntax highlighting integration (HighlightCache → build_decoration_map)
// ---------------------------------------------------------------------------

/// With a live HighlightCache, a fenced Rust block must produce spans whose
/// foreground is Color::Rgb (syntect-sourced) rather than the plain theme.text
/// fallback colour.
#[test]
fn highlighted_rust_block_has_rgb_fg_spans() {
    let text = "# Doc\n\n```rust\nfn main() {}\n```\n";
    let theme = Theme::default_theme();
    let cache = HighlightCache::new(true, "base16-ocean.dark".into(), None);
    let (map, _) = build_decoration_map(text, &theme, true, Some(&cache));

    // Content line of the fenced block is line index 3 ("fn main() {}").
    // It should have at least one span whose fg is Color::Rgb(_,_,_).
    let content_line = map.get(&3).expect("line 3 (code content) must have spans");
    assert!(
        content_line
            .iter()
            .any(|s| matches!(s.style.fg, Some(Color::Rgb(_, _, _)))),
        "highlighted fenced block line must have at least one Color::Rgb fg span"
    );
}

/// With highlighting disabled, a fenced block must still get full_line_bg
/// (fenced_bg) but must NOT have any Color::Rgb fg spans beyond theme colours
/// (i.e. syntect didn't run).
#[test]
fn disabled_highlighting_fenced_block_has_bg_no_syntect_fg() {
    let text = "```rust\nlet x = 1;\n```\n";
    let theme = Theme::default_theme();
    let cache = HighlightCache::new(false, "base16-ocean.dark".into(), None);
    let (map, _) = build_decoration_map(text, &theme, true, Some(&cache));

    // Content line is line 1.
    let content_line = map.get(&1).expect("line 1 (code content) must have spans");
    // Must have the fenced_bg full-line background.
    assert!(
        content_line
            .iter()
            .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
        "disabled-highlighting fenced line must still have fenced_bg full_line_bg"
    );
    // Must NOT have a syntect fg span (no Color::Rgb that differs from the
    // plain theme colours — specifically the single span should be theme.text fg).
    // We verify this by checking the fg is theme.text, not an arbitrary Rgb value.
    assert!(
        content_line.iter().all(|s| {
            // Acceptable fgs: theme.text (the plain fence_bg_style fg) or None.
            s.style.fg.is_none() || s.style.fg == Some(theme.text)
        }),
        "disabled-highlighting fenced line must not have syntect-sourced fg colours"
    );
}

/// Passing `None` for the cache produces the same fenced_bg fallback as a
/// disabled cache — the two paths must be indistinguishable.
#[test]
fn no_cache_fenced_block_falls_back_to_fenced_bg() {
    let text = "```python\nprint('hello')\n```\n";
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);

    let content_line = map.get(&1).expect("line 1 must have spans");
    assert!(
        content_line
            .iter()
            .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
        "None-cache fenced block must have fenced_bg full_line_bg on content lines"
    );
}

/// The fence delimiter lines (``` opening and closing) must always use the
/// blended fence_delim_style regardless of whether highlighting is active.
#[test]
fn highlighted_block_fence_delimiters_still_dimmed() {
    let text = "```rust\nlet x = 1;\n```\n";
    let theme = Theme::default_theme();
    let cache = HighlightCache::new(true, "base16-ocean.dark".into(), None);
    let (map, _) = build_decoration_map(text, &theme, true, Some(&cache));

    // Opening fence is line 0, closing is line 2.
    for fence_line in [0usize, 2] {
        let spans = map
            .get(&fence_line)
            .unwrap_or_else(|| panic!("fence line {fence_line} must have spans"));
        assert!(
            spans
                .iter()
                .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
            "fence delimiter line {fence_line} must have fenced_bg full_line_bg"
        );
    }
}

/// An unknown language tag must silently fall back to fenced_bg-only
/// (no panic, no empty map) even when a cache is present.
#[test]
fn unknown_lang_tag_falls_back_silently() {
    let text = "```notareallanguage\nsome code here\n```\n";
    let theme = Theme::default_theme();
    let cache = HighlightCache::new(true, "base16-ocean.dark".into(), None);
    let (map, _) = build_decoration_map(text, &theme, true, Some(&cache));

    let content_line = map.get(&1).expect("line 1 must have spans");
    assert!(
        content_line
            .iter()
            .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
        "unknown lang fenced block must fall back to fenced_bg on content lines"
    );
}

// ---------------------------------------------------------------------------
// Inline code
// ---------------------------------------------------------------------------

/// Inline code spans (`` `code` ``) must receive fg == theme.code_color.
///
/// The fixture line "Normal paragraph text with **bold content**, *italic
/// content*, and `inline code`." contains inline code; decoration must assign
/// theme.code_color as the foreground on those spans.
#[test]
fn fixture_inline_code_has_code_color() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.style.fg == Some(theme.code_color)),
        "expected at least one span with fg == theme.code_color (inline code)"
    );
}

// ---------------------------------------------------------------------------
// Italic
// ---------------------------------------------------------------------------

/// At least one span in the fixture must carry the ITALIC modifier.
///
/// The fixture has `*italic content*` on the paragraph line; this guards that
/// italic is actually emitted as a distinct modifier rather than collapsed into
/// the bold path.
#[test]
fn fixture_italic_has_italic_modifier() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.style.add_modifier.contains(Modifier::ITALIC)),
        "expected at least one ITALIC span in the fixture"
    );
}

// ---------------------------------------------------------------------------
// Ordered list continuation indent
// ---------------------------------------------------------------------------

/// Ordered-list bullet spans must carry continuation_indent >= 3.
///
/// The "1. " prefix is 3 columns wide; continuation rows must indent to align
/// with the item text, not the digit.
#[test]
fn fixture_ordered_list_has_continuation_indent() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    // Ordered bullet spans start at char 0; continuation_indent encodes the
    // bullet width ("1. " = 3 cols, "10. " = 4 cols, etc.).
    assert!(
        map.values()
            .flatten()
            .any(|s| s.char_start == 0 && s.continuation_indent >= 3),
        "expected at least one ordered-list span with continuation_indent >= 3"
    );
}

// ---------------------------------------------------------------------------
// H1–H3 bottom border
// ---------------------------------------------------------------------------

/// H1 lines must have border_bottom set on at least one span.
///
/// The fixture's first line is "# Heading One" (H1). The heading renderer
/// draws a full-width underline after H1–H3 rows using the border_bottom field.
#[test]
fn fixture_h1_has_border_bottom() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    // Line 0 is "# Heading One".
    let h1_spans = map.get(&0).expect("line 0 (H1) must have decoration spans");
    assert!(
        h1_spans.iter().any(|s| s.border_bottom.is_some()),
        "H1 (line 0) must have border_bottom set on at least one span"
    );
}

// ---------------------------------------------------------------------------
// Blank line inside fenced code block (regression #133)
// ---------------------------------------------------------------------------

/// A blank line that falls inside a fenced code block must still receive the
/// fenced_bg full-line background, not lose it because the line is empty.
///
/// Regression guard for #133: the original bug stripped fenced_bg from empty
/// lines within a block because the span-emission path only ran when the line
/// was non-empty.
#[test]
fn blank_line_inside_fenced_block_keeps_fenced_bg() {
    // The blank line (index 2) is inside the fenced block.
    let text = "```rust\nlet x = 1;\n\nlet y = 2;\n```\n";
    let theme = Theme::default_theme();
    let (map, _) = build_decoration_map(text, &theme, true, None);
    let blank_line = map
        .get(&2)
        .expect("blank line inside fenced block must have spans");
    assert!(
        blank_line
            .iter()
            .any(|s| s.full_line_bg == Some(theme.fenced_bg)),
        "blank line inside fenced block must retain fenced_bg full_line_bg"
    );
}

/// Regression: syntect fg spans must not be silently dropped.
///
/// The original bug: a full-line background span (0..N) was emitted first.
/// `split_into_spans` sorted by char_start and advanced char_pos to N in one
/// step, then clipped every subsequent fg span to zero length and skipped it —
/// producing a plain fenced_bg block with no syntax colours at all.
///
/// The fix: emit fg spans directly (no separate background span); put
/// `full_line_bg` on the first fg span so the background still fills the column.
#[test]
fn highlighted_block_spans_are_not_swallowed_by_background() {
    let text = "```rust\nlet keyword = 1;\n```\n";
    let theme = crate::Theme::default_theme();
    let cache = HighlightCache::new(true, "base16-ocean.dark".into(), None);
    let (map, _) = build_decoration_map(text, &theme, true, Some(&cache));

    let content_line = map.get(&1).expect("line 1 (code content) must have spans");

    // Must have MORE than one span — the bug produced exactly one wide span
    // (the background) and nothing else.
    assert!(
        content_line.len() > 1,
        "highlighted line must have multiple spans (keyword + other tokens), got: {:?}",
        content_line.len()
    );

    // At least one span must have a syntect-sourced Rgb foreground.
    assert!(
        content_line
            .iter()
            .any(|s| matches!(s.style.fg, Some(Color::Rgb(_, _, _)))),
        "highlighted line must have at least one Color::Rgb fg span — \
         background span must not swallow the syntect fg spans"
    );

    // The first span (char 0) must carry full_line_bg for background fill.
    let first = content_line
        .iter()
        .min_by_key(|s| s.char_start)
        .expect("must have at least one span");
    assert_eq!(
        first.full_line_bg,
        Some(theme.fenced_bg),
        "first highlighted span must carry full_line_bg for background fill"
    );
}
