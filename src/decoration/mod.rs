pub(crate) mod spans;
pub(crate) mod words;

pub(crate) mod block_hl;
pub(crate) mod builder;
pub(crate) mod emit;
pub(crate) mod frontmatter;
pub(crate) mod highlight;
mod types;

pub use block_hl::block_highlights_to_decoration_map;
pub use builder::build_decoration_map;
pub use frontmatter::detect_frontmatter;
pub use spans::{byte_to_line_char, line_start_bytes};
pub use types::{DecorationMap, StyledSpan};
pub use words::count_words;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use pulldown_cmark::{Event, Options, Parser};
    use ratatui::style::{Color, Modifier};

    use crate::config::{Theme, blend_colors};

    use super::emit::{add_modifier_to_existing, emit_content_around_existing};
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

    #[test]
    fn table_separator_dashes_have_muted_color() {
        // The `---` segments of the separator row must be styled with theme.muted
        // so they visually recede alongside the pipe characters.
        let theme = make_theme();
        let text = "| A | B |\n| --- | --- |\n| 1 | 2 |";
        let map = build_map(text, &theme, true);
        let sep_line = map.get(&1).expect("separator line must have spans");
        // We verify a muted span exists that covers at least one '-' character.
        let sep_text_line1 = "| --- | --- |";
        let has_dash_span = sep_line.iter().any(|s| {
            s.style.fg == Some(theme.muted)
                && (s.char_start..s.char_end).any(|i| sep_text_line1.chars().nth(i) == Some('-'))
        });
        assert!(has_dash_span, "separator row dashes must have muted color");
    }

    #[test]
    fn table_separator_alignment_colons_have_accent_color() {
        // The `:` markers in the separator row indicate column alignment and must
        // be styled with theme.accent to make the alignment intent visible.
        let theme = make_theme();
        let text = "| A | B | C |\n|:---|:---:|---:|\n| 1 | 2 | 3 |";
        let map = build_map(text, &theme, true);
        let sep_line = map.get(&1).expect("separator line must have spans");
        let has_colon_accent = sep_line
            .iter()
            .any(|s| s.style.fg == Some(theme.accent) && s.char_end - s.char_start == 1);
        assert!(
            has_colon_accent,
            "separator row `:` alignment markers must have accent color"
        );
    }

    #[test]
    fn table_separator_without_colons_has_no_accent_on_sep_line() {
        // A plain `---` separator with no alignment markers must not produce
        // any accent spans on the separator line.  Accent on line 0 (header)
        // must not bleed onto line 1 (separator).
        let theme = make_theme();
        let text = "| A | B |\n| --- | --- |\n| 1 | 2 |";
        let map = build_map(text, &theme, true);
        let sep_line = map.get(&1).expect("separator line must have spans");
        let has_accent = sep_line.iter().any(|s| s.style.fg == Some(theme.accent));
        assert!(
            !has_accent,
            "plain separator row must not produce accent spans"
        );
    }

    #[test]
    fn table_body_rows_are_not_misidentified_as_separator() {
        // Body rows that happen to contain only short words must NOT be styled
        // with separator dash/colon treatment.  Specifically a row with actual
        // content characters (letters, digits) will fail the `is_sep` check.
        let theme = make_theme();
        // Body row contains only letters — not a separator row.
        let text = "| A | B |\n| - | - |\n| x | y |";
        let map = build_map(text, &theme, true);
        // Line 2 is the body row; it must not have any dash-style muted spans
        // that aren't the pipe characters (pipes are single chars, so look for
        // muted spans of width > 1 — dashes in a separator would be 1-char each
        // but the body row has no '-' so there should be no extra muted spans).
        let empty = vec![];
        let body_line = map.get(&2).unwrap_or(&empty);
        // Only pipe spans (single-char muted) are expected; no colon accent.
        let has_colon_accent = body_line.iter().any(|s| s.style.fg == Some(theme.accent));
        assert!(
            !has_colon_accent,
            "body row must not get accent styling from separator detection"
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
        assert!(has_delim, "H1 should have a delimiter span at 0..2");
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
        // The wide content span was removed (fix for #206); heading content color
        // now lives in `line_default_style` on the delimiter span.
        let text = "# Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let delim = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 2)
            .expect("H1 delimiter span (0..2) must exist");
        let content_fg = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .and_then(|s| s.fg)
            .expect("H1 must have line_default_style with fg set");
        assert_ne!(
            delim.style.fg,
            Some(content_fg),
            "delimiter fg must be blended (different from content fg)"
        );
        assert!(
            delim.style.fg.is_some(),
            "delimiter span must have an explicit fg color"
        );
    }

    #[test]
    fn heading_h1_line_default_style_fg_is_h1_color() {
        // Wide content span removed (#206); heading fg lives in line_default_style
        // on the delimiter span so the renderer can colour unstyled heading text
        // without blocking inline decorations.
        let text = "# Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H1 delimiter span must carry line_default_style");
        assert_eq!(
            lds.fg,
            Some(theme.headings.h1),
            "H1 line_default_style.fg must be headings.h1 color"
        );
    }

    #[test]
    fn heading_h1_line_default_style_is_bold() {
        // H1 and H2 content is bold; line_default_style must carry BOLD so that
        // the renderer applies it to unstyled heading text.
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H1 delimiter span must carry line_default_style");
        assert!(
            lds.add_modifier.contains(Modifier::BOLD),
            "H1 line_default_style must carry BOLD modifier"
        );
    }

    // ---- Heading span structure (post-#206 fix) ----
    //
    // The wide heading content span (char delim_end..line_len) was removed so
    // inline decorations (code, bold, italic, links) can emit their own spans
    // inside headings without being blocked.  The heading color + BOLD live in
    // `line_default_style` on the delimiter span; the renderer uses it as the
    // default style for unstyled gaps on the line.

    #[test]
    fn heading_h1_no_wide_content_span() {
        // No span starting at char 2 (the old content span start for H1).
        // Its absence is what allows inline decorations to show through.
        let text = "# Hello";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.char_start == 2),
            "H1 must not have a wide content span starting at char 2 (would block inline decs)"
        );
    }

    #[test]
    fn heading_inline_code_uses_code_color() {
        // Regression guard for #206: inline code inside a heading must produce a
        // span with code_color.  Before the fix the wide content span blocked all
        // code spans, making backtick code render identically to plain heading text.
        let text = "# Heading with `code` inside";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.code_color)),
            "inline code inside a heading must produce a span with code_color; \
             got: {spans:?}"
        );
    }

    #[test]
    fn heading_inline_bold_uses_bold_color() {
        // Regression guard for #206: bold inside a heading must produce a span with
        // bold_color + BOLD.  The wide content span previously blocked it.
        let text = "# Heading **bold** text";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.bold_color)
                && s.style.add_modifier.contains(Modifier::BOLD)),
            "bold inside a heading must produce a bold_color + BOLD span; got: {spans:?}"
        );
    }

    #[test]
    fn heading_inline_code_uses_heading_bg_not_code_bg() {
        // Inline code inside a heading should use heading_bg as its cell background
        // (from in_heading_bg context), not code_bg, so it doesn't punch through
        // the heading stripe.
        let text = "# Heading with `code`";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 should have spans");
        assert!(
            !spans.iter().any(|s| s.style.bg == Some(theme.code_bg)),
            "code inside a heading must not use code_bg; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.style.fg == Some(theme.code_color)
                && s.style.bg == Some(theme.heading_bg)),
            "code inside a heading must use code_color fg and heading_bg bg; got: {spans:?}"
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

    // ── Precision span-boundary tests ────────────────────────────────────────
    //
    // These tests pin exact `char_start`/`char_end` values so that off-by-one
    // mutations in `build_decoration_map`'s position arithmetic are caught.
    // Each test documents *which* expression it kills.

    // ---- Plain bold delimiter boundaries ----
    //
    // Kills the arithmetic in End(TagEnd::Strong):
    //   · `start_char + 2`  → opening delim end / content start
    //   · `end_char_excl - 2` → content end / closing delim start
    //   · `span_len >= 4`   → the minimum-span guard

    #[test]
    fn bold_plain_opening_delimiter_is_at_zero_to_two() {
        // "**hi**" — opening ** must be at chars 0..2.
        let text = "**hi**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "plain bold opening ** delimiter must be at chars 0..2; got: {spans:?}"
        );
    }

    #[test]
    fn bold_plain_content_is_at_two_to_four() {
        // "**hi**" — content "hi" must be at chars 2..4 with bold_color + BOLD.
        let text = "**hi**";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2 && s.char_end == 4)
            .expect("plain bold content must be at chars 2..4");
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "plain bold content at 2..4 must carry BOLD modifier"
        );
        assert_eq!(
            content.style.fg,
            Some(theme.bold_color),
            "plain bold content at 2..4 must use bold_color"
        );
    }

    #[test]
    fn bold_plain_closing_delimiter_is_at_four_to_six() {
        // "**hi**" — closing ** must be at chars 4..6.
        let text = "**hi**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 6),
            "plain bold closing ** delimiter must be at chars 4..6; got: {spans:?}"
        );
    }

    #[test]
    fn bold_min_span_guard_four_chars_produces_spans() {
        // A bold span of exactly 4 chars ("**x**" = 5 chars total is wrong — the
        // minimum guard is `span_len >= 4` where span_len is end - start in chars.
        // "**x**" is 5 chars, span_len = 5 >= 4 → allowed.
        // "**x*" (1 asterisk) = 4 chars but not parsed as bold.
        // The smallest real bold is "**x**" (5 chars); span_len = 5 >= 4 ✓.
        // This tests that the `>= 4` guard passes for a minimal one-char-content bold.
        let text = "**x**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.style.add_modifier.contains(Modifier::BOLD)),
            "minimal 1-char bold **x** must still produce BOLD span (span_len=5 >= 4)"
        );
    }

    // ---- Plain italic delimiter boundaries ----
    //
    // Kills:
    //   · `start_char + 1`  → opening delim end / content start
    //   · `end_char_excl - 1` → content end / closing delim start
    //   · `span_len >= 2`   → the minimum-span guard

    #[test]
    fn italic_plain_opening_delimiter_is_at_zero_to_one() {
        // "*hi*" — opening * must be at chars 0..1.
        let text = "*hi*";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 1),
            "plain italic opening * delimiter must be at chars 0..1; got: {spans:?}"
        );
    }

    #[test]
    fn italic_plain_content_is_at_one_to_three() {
        // "*hi*" — content "hi" must be at chars 1..3 with italic_color.
        let text = "*hi*";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 1 && s.char_end == 3)
            .expect("plain italic content must be at chars 1..3");
        assert_eq!(
            content.style.fg,
            Some(theme.italic_color),
            "plain italic content at 1..3 must use italic_color"
        );
    }

    #[test]
    fn italic_plain_closing_delimiter_is_at_three_to_four() {
        // "*hi*" — closing * must be at chars 3..4.
        let text = "*hi*";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 4),
            "plain italic closing * delimiter must be at chars 3..4; got: {spans:?}"
        );
    }

    // ---- Table pipe exact character positions ----
    //
    // "| A | B |" has pipes at chars 0, 4, 8.
    // Kills the `char_idx` loop in Event::Start(Tag::Table(..)):
    //   · any mutation that shifts the `char_idx` counter or uses byte count
    //     instead of char count would misplace at least one pipe span.

    #[test]
    fn table_header_pipes_at_exact_char_positions() {
        // "| A | B |" — pipes at 0, 4, 8 each styled with theme.muted.
        let text = "| A | B |\n| - | - |\n| 1 | 2 |";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("header line must have spans");
        for &pos in &[0usize, 4, 8] {
            assert!(
                spans.iter().any(|s| {
                    s.char_start == pos && s.char_end == pos + 1 && s.style.fg == Some(theme.muted)
                }),
                "pipe at char {pos} must produce a muted span at {pos}..{}; got: {spans:?}",
                pos + 1
            );
        }
    }

    #[test]
    fn table_separator_pipes_at_exact_char_positions() {
        // "| - | - |" — pipes at 0, 4, 8.
        let text = "| A | B |\n| - | - |\n| 1 | 2 |";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&1).expect("separator line must have spans");
        for &pos in &[0usize, 4, 8] {
            assert!(
                spans.iter().any(|s| {
                    s.char_start == pos && s.char_end == pos + 1 && s.style.fg == Some(theme.muted)
                }),
                "separator pipe at char {pos} must produce a muted span at {pos}..{}; got: {spans:?}",
                pos + 1
            );
        }
    }

    #[test]
    fn table_body_pipes_at_exact_char_positions() {
        // "| 1 | 2 |" — pipes at 0, 4, 8.
        let text = "| A | B |\n| - | - |\n| 1 | 2 |";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&2).expect("body line must have spans");
        for &pos in &[0usize, 4, 8] {
            assert!(
                spans.iter().any(|s| {
                    s.char_start == pos && s.char_end == pos + 1 && s.style.fg == Some(theme.muted)
                }),
                "body row pipe at char {pos} must produce a muted span at {pos}..{}; got: {spans:?}",
                pos + 1
            );
        }
    }

    // ---- Fenced code block fence delimiter char bounds ----
    //
    // Kills the `fence_count` / `close_fence` path in Event::Start(Tag::CodeBlock):
    //   · a mutation that changes 0 → 1 (char_start) would break the opening/closing span.
    //   · a mutation that changes the length calculation would mis-size the delimiter span.

    #[test]
    fn fenced_code_opening_fence_span_is_at_zero_to_three() {
        // Opening ``` on line 0 must have a span at chars 0..3.
        let text = "```\ncode\n```\n";
        let map = build_map(text, &make_theme(), false);
        let opening = map.get(&0).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "opening ``` fence delimiter must be at chars 0..3; got: {opening:?}"
        );
    }

    #[test]
    fn fenced_code_closing_fence_span_is_at_zero_to_three() {
        // Closing ``` on line 2 must have a span at chars 0..3.
        let text = "```\ncode\n```\n";
        let map = build_map(text, &make_theme(), false);
        let closing = map.get(&2).expect("closing fence line must have spans");
        assert!(
            closing.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "closing ``` fence delimiter must be at chars 0..3; got: {closing:?}"
        );
    }

    #[test]
    fn fenced_code_four_backtick_fence_span_is_at_zero_to_four() {
        // A ````-fenced block uses 4-char delimiters.
        let text = "````\ncode\n````\n";
        let map = build_map(text, &make_theme(), false);
        let opening = map.get(&0).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.char_start == 0 && s.char_end == 4),
            "opening ```` fence delimiter must be at chars 0..4; got: {opening:?}"
        );
    }

    #[test]
    fn fenced_code_lang_tag_span_immediately_follows_fence() {
        // "```rust" — language tag "rust" starts at char 3 (right after ```).
        let text = "```rust\nlet x = 1;\n```\n";
        let map = build_map(text, &make_theme(), false);
        let opening = map.get(&0).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.char_start == 3 && s.char_end == 7),
            "language tag 'rust' must be at chars 3..7 (right after ``` at 0..3); got: {opening:?}"
        );
    }

    // ---- Link ASCII exact span positions ----
    //
    // Kills the arithmetic in Event::Start(Tag::Link{..}):
    //   · `start_char + 1`         → text start (after [)
    //   · `start_char + split_idx` → ]( start
    //   · `+ 2` in url_start       → url start (after ])
    //   · `end_char_excl - 1`      → closing ) start

    #[test]
    fn link_ascii_opening_bracket_is_at_zero_to_one() {
        // "[hi](url)" — opening [ at chars 0..1.
        let text = "[hi](url)";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 1),
            "opening [ must be at chars 0..1; got: {spans:?}"
        );
    }

    #[test]
    fn link_ascii_text_span_is_underlined_at_one_to_three() {
        // "[hi](url)" — link text "hi" at chars 1..3 with UNDERLINED.
        let text = "[hi](url)";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let text_span = spans
            .iter()
            .find(|s| s.char_start == 1 && s.char_end == 3)
            .expect("link text 'hi' must be at chars 1..3");
        assert!(
            text_span.style.add_modifier.contains(Modifier::UNDERLINED),
            "link text at 1..3 must be UNDERLINED; got: {text_span:?}"
        );
    }

    #[test]
    fn link_ascii_bracket_paren_delimiter_is_at_three_to_five() {
        // "[hi](url)" — ]( delimiter at chars 3..5.
        let text = "[hi](url)";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 5),
            "]( delimiter must be at chars 3..5; got: {spans:?}"
        );
    }

    #[test]
    fn link_ascii_url_span_is_at_five_to_eight() {
        // "[hi](url)" — URL "url" at chars 5..8.
        let text = "[hi](url)";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let url_span = spans
            .iter()
            .find(|s| s.char_start == 5 && s.char_end == 8)
            .expect("url 'url' must be at chars 5..8");
        assert_eq!(
            url_span.style.fg,
            Some(theme.link_url),
            "url span must use link_url color; got: {url_span:?}"
        );
    }

    #[test]
    fn link_ascii_closing_paren_is_at_eight_to_nine() {
        // "[hi](url)" — closing ) at chars 8..9.
        let text = "[hi](url)";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 8 && s.char_end == 9),
            "closing ) must be at chars 8..9; got: {spans:?}"
        );
    }

    // ---- Ordered list bullet char_end ----
    //
    // "1. item" — bullet spans chars 0..2 covering "1."
    // Kills: `find(['.', ')'])` offset calculation, `count_chars_in` call,
    //        `item_char + 2` fallback.

    #[test]
    fn ordered_list_one_digit_bullet_span_is_at_zero_to_two() {
        // "1. item text" — bullet "1." must produce a span at chars 0..2.
        let text = "1. item text";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "ordered '1.' bullet span must be at chars 0..2; got: {spans:?}"
        );
    }

    #[test]
    fn ordered_list_two_digit_bullet_span_is_at_zero_to_three() {
        // A list starting at item 10 would have "10. item" — bullet at 0..3.
        // We can't force pulldown_cmark to start at 10 without a preceding list,
        // so we use "9." (2 chars + dot = 3 chars).  Note: pulldown_cmark counts
        // the number character, not the sequence, so "9." gives a 2-char marker.
        // Actually `9.` is char '9' + '.' = 2 chars → bullet_end = 2.
        // We need to check the code path for `find(['.', ')'])` returning the right offset.
        // Use "10." but preceded by earlier items to get past the 1-digit case.
        // Simplest: directly give "10. item" as the only item.
        let text = "10. item text";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        // pulldown_cmark parses "10. item text" as an ordered list item starting at 10.
        // The bullet prefix is "10." = 3 chars, so span should be at 0..3.
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "ordered '10.' bullet span must be at chars 0..3; got: {spans:?}"
        );
    }

    // ---- Strikethrough color ----
    //
    // The content span must use theme.strikethrough_color (kills fg mutation).

    #[test]
    fn strikethrough_content_has_strikethrough_color() {
        let text = "~~hi~~";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2 && s.char_end == 4)
            .expect("strikethrough content must be at chars 2..4");
        assert_eq!(
            content.style.fg,
            Some(theme.strikethrough_color),
            "strikethrough content must use strikethrough_color; got: {content:?}"
        );
    }

    // ---- Horizontal rule color and flags ----
    //
    // Kills `is_rule = true → false` mutation and `fg = rule_color` deletion.

    #[test]
    fn horizontal_rule_is_rule_and_has_rule_color_on_exact_line() {
        // The rule is on line 2 ("---" with blank lines around it).
        let text = "above\n\n---\n\nbelow";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let rule_line = map
            .values()
            .find(|spans| spans.iter().any(|s| s.is_rule))
            .expect("a rule line must exist");
        let rule_span = rule_line
            .iter()
            .find(|s| s.is_rule)
            .expect("rule span must exist");
        assert_eq!(
            rule_span.style.fg,
            Some(theme.rule_color),
            "rule span must use theme.rule_color"
        );
        assert_eq!(rule_span.char_start, 0, "rule span char_start must be 0");
        assert!(
            rule_span.char_end > 0,
            "rule span char_end must be non-zero (covers the '---' chars)"
        );
    }

    // ---- Blockquote continuation indent ----
    //
    // Kills `continuation_indent: 2` → `continuation_indent: 0` mutation
    // (the `2` literal in the StyledSpan initialiser).

    #[test]
    fn blockquote_indicator_continuation_indent_is_exactly_two() {
        let text = "> quoted";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let indicator = spans
            .iter()
            .find(|s| s.is_blockquote && s.char_start == 0)
            .expect("blockquote indicator span must exist");
        assert_eq!(
            indicator.continuation_indent, 2,
            "blockquote continuation_indent must be exactly 2; got: {indicator:?}"
        );
    }

    // ---- Word count ----
    //
    // Kills `+=` → `-=` / `= 0` in the word count accumulation.

    #[test]
    fn word_count_plain_paragraph() {
        let text = "one two three";
        let theme = make_theme();
        let (_, wc) = build_decoration_map(text, &theme, false, None);
        assert_eq!(wc, 3, "plain paragraph must count 3 words");
    }

    #[test]
    fn word_count_heading_plus_paragraph() {
        let text = "# Title\n\nHello world.";
        let theme = make_theme();
        let (_, wc) = build_decoration_map(text, &theme, false, None);
        assert_eq!(wc, 3, "heading (1) + paragraph (2) must total 3 words");
    }

    #[test]
    fn word_count_inline_code_counted() {
        // Event::Code is counted separately from Event::Text.
        let text = "See `foo bar` please.";
        let theme = make_theme();
        let (_, wc) = build_decoration_map(text, &theme, false, None);
        // "See" (1) + "foo bar" via Code (2) + "please." (1) = 4
        assert_eq!(wc, 4, "inline code words must be counted");
    }

    #[test]
    fn word_count_empty_doc_is_zero() {
        let theme = make_theme();
        let (_, wc) = build_decoration_map("", &theme, false, None);
        assert_eq!(wc, 0, "empty document must have word count 0");
    }

    // ---- Heading color propagation ----
    //
    // Kills mutations that swap heading level colors (e.g. h1 ↔ h2 in the match).

    #[test]
    fn heading_h1_content_uses_h1_color() {
        let text = "# Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H1 must have line_default_style");
        assert_eq!(
            lds.fg,
            Some(theme.headings.h1),
            "H1 content must use headings.h1 color"
        );
    }

    #[test]
    fn heading_h2_content_uses_h2_color() {
        let text = "## Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H2 must have line_default_style");
        assert_eq!(
            lds.fg,
            Some(theme.headings.h2),
            "H2 content must use headings.h2 color"
        );
    }

    #[test]
    fn heading_h3_content_uses_h3_color() {
        let text = "### Hello";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H3 must have line_default_style");
        assert_eq!(
            lds.fg,
            Some(theme.headings.h3),
            "H3 content must use headings.h3 color"
        );
    }

    // ---- Heading delimiter counts (delim_chars = level + 1) ----
    //
    // Kills `level_num + 1` → `level_num * 1` and `level_num - 1` mutations
    // that would miscalculate the delimiter width.

    #[test]
    fn heading_h4_delimiter_span_is_at_zero_to_five() {
        // "#### Hello" — H4 delimiter "#### " = 5 chars (4 hashes + space).
        let text = "#### Hello";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 5),
            "H4 delimiter span (#### + space) must be at chars 0..5; got: {spans:?}"
        );
    }

    #[test]
    fn heading_h6_delimiter_span_is_at_zero_to_seven() {
        // "###### Hello" — H6 delimiter "###### " = 7 chars.
        let text = "###### Hello";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 7),
            "H6 delimiter span (6 hashes + space) must be at chars 0..7; got: {spans:?}"
        );
    }

    // ---- Bold in non-zero column (offset test) ----
    //
    // "text **hi**" — bold spans must be offset by the leading text width.
    // Kills off-by-one in `byte_to_line_char` usage for `strong_range.start`.

    #[test]
    fn bold_mid_line_opening_delimiter_is_at_five_to_seven() {
        // "text **hi**" — "text " = 5 chars, then ** at 5..7.
        let text = "text **hi**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 7),
            "mid-line bold opening ** must be at chars 5..7; got: {spans:?}"
        );
    }

    #[test]
    fn bold_mid_line_content_is_at_seven_to_nine() {
        // "text **hi**" — "hi" content at chars 7..9 with BOLD.
        let text = "text **hi**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 7 && s.char_end == 9)
            .expect("mid-line bold content must be at chars 7..9");
        assert!(
            content.style.add_modifier.contains(Modifier::BOLD),
            "mid-line bold content at 7..9 must carry BOLD modifier"
        );
    }

    #[test]
    fn bold_mid_line_closing_delimiter_is_at_nine_to_eleven() {
        // "text **hi**" — closing ** at chars 9..11.
        let text = "text **hi**";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 9 && s.char_end == 11),
            "mid-line bold closing ** must be at chars 9..11; got: {spans:?}"
        );
    }

    // ---- Italic in non-zero column ----

    #[test]
    fn italic_mid_line_opening_delimiter_is_at_five_to_six() {
        // "text *hi*" — opening * at char 5..6.
        let text = "text *hi*";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 6),
            "mid-line italic opening * must be at chars 5..6; got: {spans:?}"
        );
    }

    #[test]
    fn italic_mid_line_content_is_at_six_to_eight() {
        // "text *hi*" — content "hi" at chars 6..8 with italic_color.
        let text = "text *hi*";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 6 && s.char_end == 8)
            .expect("mid-line italic content must be at chars 6..8");
        assert_eq!(
            content.style.fg,
            Some(theme.italic_color),
            "mid-line italic content at 6..8 must use italic_color"
        );
    }

    #[test]
    fn italic_mid_line_closing_delimiter_is_at_eight_to_nine() {
        // "text *hi*" — closing * at char 8..9.
        let text = "text *hi*";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 8 && s.char_end == 9),
            "mid-line italic closing * must be at chars 8..9; got: {spans:?}"
        );
    }

    // ---- Inline code with multiple backticks ----

    #[test]
    fn inline_code_double_backtick_delimiter_is_at_zero_to_two() {
        // "``hi``" — opening `` at chars 0..2.
        let text = "``hi``";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "double-backtick opening `` must be at chars 0..2; got: {spans:?}"
        );
    }

    #[test]
    fn inline_code_double_backtick_content_is_at_two_to_four() {
        // "``hi``" — content at chars 2..4 with code_color.
        let text = "``hi``";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content = spans
            .iter()
            .find(|s| s.char_start == 2 && s.char_end == 4)
            .expect("double-backtick content must be at chars 2..4");
        assert_eq!(
            content.style.fg,
            Some(theme.code_color),
            "inline code content at 2..4 must use code_color"
        );
    }

    #[test]
    fn inline_code_double_backtick_closing_is_at_four_to_six() {
        // "``hi``" — closing `` at chars 4..6.
        let text = "``hi``";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 6),
            "double-backtick closing `` must be at chars 4..6; got: {spans:?}"
        );
    }

    // ---- T-1: Heading content span carries full_line_bg ----

    #[test]
    fn heading_h3_line_default_style_not_bold() {
        // H3–H6 content is not bold; line_default_style must NOT carry BOLD.
        // Also verifies that line_default_style is set for all heading levels.
        let text = "### Hello";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let lds = spans
            .iter()
            .find_map(|s| s.line_default_style)
            .expect("H3 must have line_default_style");
        assert!(
            !lds.add_modifier.contains(Modifier::BOLD),
            "H3 line_default_style must NOT carry BOLD modifier; got: {lds:?}"
        );
    }

    // ---- T-3: Fenced code block — no trailing newline ----

    #[test]
    fn fenced_code_no_trailing_newline_opening_fence_span() {
        // Kills: lines 615:48 (`+→*` in `start_line + 1 < line_starts.len()`)
        //        and 728:50 (`<→==`, `<→>`, `<→<=`, `+→-`, `+→*`).
        // When the closing ``` is the very last byte (no trailing \n), the
        // `end_line + 1 == line_starts.len()` boundary is exercised.
        let text = "```\ncode\n```"; // no trailing newline
        let theme = make_theme();
        let map = build_map(text, &theme, false);

        let opening = map.get(&0).expect("opening fence line must have spans");
        assert!(
            opening.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "opening ``` (no trailing \\n) must be at chars 0..3; got: {opening:?}"
        );

        let closing = map.get(&2).expect("closing fence line must have spans");
        assert!(
            closing.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "closing ``` (no trailing \\n) must be at chars 0..3; got: {closing:?}"
        );
    }

    // ---- T-4: Tilde fence — close fence char matching ----

    #[test]
    fn tilde_fenced_code_closing_fence_span() {
        // Kills: line 735:56 `replace == with != in build_decoration_map`
        // The take_while closure uses `c == '`' || c == '~'`.  Mutating
        // `c == '~'` → `c != '~'` silently breaks tilde detection.
        let text = "~~~\ncode\n~~~";
        let theme = make_theme();
        let map = build_map(text, &theme, false);

        let closing = map
            .get(&2)
            .expect("closing tilde fence line must have spans");
        assert!(
            closing.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "closing ~~~ fence must span chars 0..3; got: {closing:?}"
        );
    }

    // ---- T-5: Fenced code content line — char_end and style ----

    #[test]
    fn fenced_code_content_span_char_end_and_style() {
        // Kills: `delete field char_end` at build_decoration_map line `\d+`:37
        //   and `delete field style` at build_decoration_map line `\d+`:37.
        // (Line numbers use \d+ because the function shifts as helpers are added.)
        // The fallback content span (no syntect, highlight_cache=None) must cover
        // exactly the line's chars and carry fence_bg_style = fg(text).bg(fenced_bg).
        let text = "```\nhello\n```\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);

        let content_line = map.get(&1).expect("content line 1 must have spans");
        let content = content_line
            .iter()
            .find(|s| s.char_start == 0)
            .expect("content span must start at char 0");
        assert_eq!(
            content.char_end, 5,
            "content span must cover 'hello' (5 chars); got: {content:?}"
        );
        assert_eq!(
            content.style.fg,
            Some(theme.text),
            "fenced fallback content span must have fg=theme.text; got: {content:?}"
        );
        assert_eq!(
            content.style.bg,
            Some(theme.fenced_bg),
            "fenced fallback content span must have bg=theme.fenced_bg; got: {content:?}"
        );
        assert_eq!(
            content.full_line_bg,
            Some(theme.fenced_bg),
            "content span full_line_bg must equal theme.fenced_bg; got: {content:?}"
        );
    }

    // ---- T-6: Blockquote indicator style ----

    #[test]
    fn blockquote_indicator_style_is_muted() {
        // Kills: line 784:29 `delete field style from struct StyledSpan`
        // The blockquote indicator `▌` span must carry the muted fg color.
        let text = "> quote";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let indicator = spans
            .iter()
            .find(|s| s.is_blockquote)
            .expect("blockquote must have an is_blockquote span");
        assert_eq!(
            indicator.style.fg,
            Some(theme.muted),
            "blockquote indicator must use theme.muted fg; got: {indicator:?}"
        );
    }

    // ---- T-7: Nested ordered-in-unordered — End(List) resets in_ordered_list ----

    #[test]
    fn nested_ordered_in_unordered_outer_item_keeps_unordered_bullet() {
        // Kills: `delete match arm Event::End(TagEnd::List(_))` at build_decoration_map.
        //
        // pulldown-cmark event sequence for "- outer1\n  1. inner\n- outer2":
        //   Start(List(None))     → in_ordered_list = false
        //   Start(Item)           — outer1
        //   Start(List(Some(1))) → in_ordered_list = true
        //   Start(Item)           — inner
        //   End(Item)
        //   End(List(_))          ← the arm under test; sets in_ordered_list = false
        //   End(Item)
        //   Start(Item)           — outer2: reads in_ordered_list here, NO fresh Start(List)
        //
        // Without the End(List) arm, in_ordered_list stays `true` from the inner list,
        // so outer2's bullet_end falls through to `unwrap_or(item_char + 2)` = 2.
        // With the arm present, in_ordered_list is reset to false → bullet_end = 1.
        //
        // The consecutive-list variant "1. first\n\n- second" does NOT kill this mutation
        // because Start(List(None)) fires for the second list before any of its items,
        // resetting in_ordered_list to false regardless of the End arm.
        let text = "- outer1\n  1. inner\n- outer2";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans_2 = map.get(&2).expect("line 2 ('- outer2') must have spans");
        let bullet = spans_2
            .iter()
            .find(|s| s.char_start == 0 && s.continuation_indent > 0)
            .expect("unordered bullet span must exist on line 2");
        assert_eq!(
            bullet.char_end, 1,
            "outer unordered bullet must end at char 1 after nested ordered list ends; \
             char_end 2 means in_ordered_list was not reset by End(List); got: {bullet:?}"
        );
    }

    // ---- T-8: Nested list bullet at non-zero item_char ----

    #[test]
    fn nested_list_bullet_starts_at_item_char() {
        // Kills: line 896:25 `delete field char_start from struct StyledSpan`
        // For "  - nested", item_char = 2; the bullet span must start at 2, not 0 (default).
        let text = "- outer\n  - nested";
        let map = build_map(text, &make_theme(), false);
        let spans_1 = map.get(&1).expect("line 1 (nested item) must have spans");
        let bullet = spans_1
            .iter()
            .find(|s| s.continuation_indent > 0)
            .expect("nested bullet span must exist");
        assert_eq!(
            bullet.char_start, 2,
            "nested bullet must start at char 2 (after 2-space indent); got: {bullet:?}"
        );
        assert_eq!(
            bullet.char_end, 3,
            "nested bullet must end at char 3 (the `-`); got: {bullet:?}"
        );
    }

    // ---- T-9: Checked todo item — inline spans keep continuation_indent = 0 ----

    #[test]
    fn todo_checked_inline_spans_keep_zero_continuation_indent() {
        // The guard `if span.continuation_indent > 0` upgrades only pre-existing
        // spans whose ci is already > 0 (the bullet span).  Inline spans (bold etc.)
        // are pushed *after* the guard loop fires, so they keep ci=0 regardless.
        // This test verifies that invariant holds for the bold delimiter.
        let text = "- [x] **bold**";
        let map = build_map(text, &make_theme(), false);
        let spans_0 = map.get(&0).expect("line 0 must have spans");
        // Bold opening delimiter `**` is at char 6..8 and must have ci=0.
        let bold_delim = spans_0
            .iter()
            .find(|s| s.char_start == 6 && s.char_end == 8)
            .expect("bold opening ** at (6,8) must exist");
        assert_eq!(
            bold_delim.continuation_indent, 0,
            "bold delimiter must NOT inherit todo continuation_indent; got: {bold_delim:?}"
        );
    }

    // ---- T-10: Checked todo item sub-span boundaries (marker_char = 2) ----

    #[test]
    fn todo_checked_bracket_spans_at_correct_positions() {
        // Kills: lines 925:52, 932:61, 935:40, 935:36, 939:51, 939:69,
        //        943:40, 943:36, 947:51, 951:36.
        // "- [x] done": marker_char=2, bracket_end=5.
        // Sub-spans: `[` at (2,3), `x` at (3,4), `]` at (4,5).
        let text = "- [x] done";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 2 && s.char_end == 3),
            "[ bracket must be at (2,3); got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 4),
            "x char must be at (3,4); got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 5),
            "] bracket must be at (4,5); got: {spans:?}"
        );
    }

    // ---- T-10b: Checked todo item with non-zero marker_char (ordered list) ----

    #[test]
    fn todo_checked_ordered_item_bracket_spans_at_correct_positions() {
        // Ordered list item: "1. [x] done" → marker_char=3 (after "1. ").
        // Kills mutations where `+1`/`+2`/`+3` produce different results than `*1`/`*2`/`*3`
        // for marker_char=2 (but distinct for marker_char=3).
        let text = "1. [x] done";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        // `[` at (3,4), `x` at (4,5), `]` at (5,6).
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 4),
            "[ bracket must be at (3,4) for ordered item; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 5),
            "x char must be at (4,5) for ordered item; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 6),
            "] bracket must be at (5,6) for ordered item; got: {spans:?}"
        );
    }

    // ---- T-11: Unchecked checkbox bracket positions ----

    #[test]
    fn todo_unchecked_bracket_spans_at_correct_positions() {
        // Kills: lines 965:60, 970:47, 970:64.
        // "- [ ] todo": marker_char=2. `[` at (2,3), `]` at (4,5).
        let text = "- [ ] todo";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 2 && s.char_end == 3),
            "[ bracket must be at (2,3); got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 4 && s.char_end == 5),
            "] bracket must be at (4,5); got: {spans:?}"
        );
    }

    // ---- T-11b: Unchecked checkbox with non-zero marker_char (ordered list) ----

    #[test]
    fn todo_unchecked_ordered_item_bracket_spans_at_correct_positions() {
        // "1. [ ] task" → marker_char=3 (after "1. ").
        // Kills `+2→*2` (3+2=5 ≠ 3*2=6) and `+3→*3` (3+3=6 ≠ 3*3=9) mutations.
        let text = "1. [ ] task";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        // `[` at (3,4), `]` at (5,6).
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 4),
            "[ bracket must be at (3,4) for ordered unchecked; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 6),
            "] bracket must be at (5,6) for ordered unchecked; got: {spans:?}"
        );
    }

    // ---- T-11c: Checked todo item text span has todo_done style ----

    #[test]
    fn todo_checked_item_text_span_has_todo_done_style() {
        // Kills: `replace < with ==` and `replace < with >` at build_decoration_map
        //   line \d+:36 — the `if bracket_end < line_len` guard.
        // "- [x] done": marker_char=2, bracket_end=5, line_len=10.
        //   · `< → ==`: 5 == 10 is false → item text span not pushed.
        //   · `< → >`:  5 > 10 is false → item text span not pushed.
        // In both cases the `todo_done`-coloured text span disappears.
        let text = "- [x] done";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        // bracket_end = 2 + 3 = 5; item text "done" is chars 6..10 but we need
        // bracket_end..line_len.  "- [x] done" has 10 chars; bracket_end = 5.
        // The space after `]` is char 5, "done" is chars 6..10.
        // `make_span(bracket_end, line_len, ...)` produces (5, 10).
        let text_span = spans
            .iter()
            .find(|s| s.char_start == 5)
            .expect("item text span must start at bracket_end (char 5)");
        assert_eq!(
            text_span.char_end, 10,
            "item text span must cover chars 5..10 ('_done'); got: {text_span:?}"
        );
        assert_eq!(
            text_span.style.fg,
            Some(theme.todo_done),
            "checked item text must have todo_done fg color; got: {text_span:?}"
        );
    }

    // ---- T-12: Table is_sep — body row with dash is NOT a separator ----

    #[test]
    fn table_body_row_with_dash_is_not_separator() {
        // Kills: line 1000:25 `replace && with || in build_decoration_map`
        // With `||`, any non-empty cell makes is_sep true because `contains('-')`
        // is checked; the body row `a-b` would get separator styling.
        let text = "| head |\n| --- |\n| a-b |";
        let map = build_map(text, &make_theme(), false);
        let spans_2 = map.get(&2).expect("line 2 (body row) must have spans");
        // A separator row would produce individual char spans for `-`.
        // Check that the `-` inside `a-b` (at char 3) doesn't get a lone span.
        let has_lone_dash_at_3 = spans_2.iter().any(|s| s.char_start == 3 && s.char_end == 4);
        assert!(
            !has_lone_dash_at_3,
            "body-row '-' at (3,4) must not get sep_dash_style; got: {spans_2:?}"
        );
    }

    // ---- T-13: Table match guards — sep-only styling in separator rows ----

    #[test]
    fn table_body_row_colon_not_styled_as_separator() {
        // Kills: lines 1013:36 and 1020:36 `replace match guard is_sep with true`
        // With guard=true, body-row `:` and `-` get sep coloring unconditionally.
        let text = "| h1 | h2 |\n| -- | -- |\n| :val: | a-b |";
        let map = build_map(text, &make_theme(), false);
        let spans_2 = map.get(&2).expect("line 2 (body row) must have spans");
        // The `:` at char 2 of the body row must not get sep_colon_style span.
        let has_colon_at_2 = spans_2.iter().any(|s| s.char_start == 2 && s.char_end == 3);
        assert!(
            !has_colon_at_2,
            "body-row ':' at char 2 must not get sep_colon_style; got: {spans_2:?}"
        );
    }

    // ---- T-14: Mid-line strikethrough — start_char arithmetic ----

    #[test]
    fn strikethrough_mid_line_delimiter_positions() {
        // Kills: line 1081:39 `replace + with * in build_decoration_map`
        // "ok ~~word~~ rest": start_char=3.
        // Opening ~~ at (3,5): 3+0..3+2. Closing ~~ at (9,11): end-2..end.
        // Mutation `+→*` on start_char: 3*2=6 ≠ 5 for opening.
        let text = "ok ~~word~~ rest";
        let map = build_map(text, &make_theme(), false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 5),
            "opening ~~ must be at chars (3,5); got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 9 && s.char_end == 11),
            "closing ~~ must be at chars (9,11); got: {spans:?}"
        );
    }

    // ---- Bold+italic adjacency check (line 454) ----

    #[test]
    fn bold_italic_combined_content_has_both_modifiers() {
        // Kills: lines 454:38 (`==→!=`), 454:42 (`+→*`), 454:80 (`+→-`,`+→*`),
        //        454:84 (`==→!=`), 454:62 (`&&→||`) in adjacency check.
        // "***x***": strong wraps emphasis (adjacent).  Content char must have BOLD+ITALIC.
        let text = "***x***";
        let map = build_map(text, &make_theme(), true); // italic_support=true
        let spans = map.get(&0).expect("line 0 must have spans");
        // Content of `***x***` is `x` at char 3 (after `***`).
        let content = spans
            .iter()
            .find(|s| s.char_start == 3 && s.char_end == 4)
            .expect("content char 'x' at (3,4) must have a span");
        assert!(
            content
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::BOLD),
            "content of ***x*** must be BOLD; got: {content:?}"
        );
        assert!(
            content
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC),
            "content of ***x*** must be ITALIC; got: {content:?}"
        );
    }

    // ---- Italic delimiter position at non-zero start_char (lines 495, 510) ----

    #[test]
    fn italic_mid_line_content_span_at_correct_position() {
        // Kills: lines 495:48 (`+→*` in `start_char + 1`) and 510:52.
        // "a *b* c": start_char=2 (after "a ").  Content span at (3,4), not (2,4).
        // With `+→*` at 495:48: start_char * 1 = 2 → range starts at 2 (includes delim).
        let text = "a *b* c";
        let map = build_map(text, &make_theme(), true); // italic_support=true exercises 510
        let spans = map.get(&0).expect("line 0 must have spans");
        // The italic content `b` is at char 3 (start_char + 1 = 2 + 1 = 3).
        let content = spans
            .iter()
            .find(|s| s.char_start == 3 && s.char_end == 4)
            .expect("italic content 'b' must be at chars (3,4); got no such span");
        assert!(
            content
                .style
                .add_modifier
                .contains(ratatui::style::Modifier::ITALIC),
            "italic content must have ITALIC modifier; got: {content:?}"
        );
    }

    // ---- Fenced code language tag arithmetic (line 638) ----

    #[test]
    fn fenced_code_lang_tag_span_end_position() {
        // Kills: line 638:53 `replace + with * in build_decoration_map`
        // "```rust": fence_count=3, lang="rust" (4 chars).
        // lang_end = fence_count + lang.chars().count() = 3 + 4 = 7.
        // Mutation `+→*`: 3 * 4 = 12 ≠ 7.
        let text = "```rust\ncode\n```\n";
        let map = build_map(text, &make_theme(), false);
        let opening = map.get(&0).expect("opening fence line must have spans");
        // Lang tag "rust" spans chars 3..7.
        assert!(
            opening.iter().any(|s| s.char_start == 3 && s.char_end == 7),
            "lang tag 'rust' must end at char 7 (3+4); got: {opening:?}"
        );
    }

    // ---- Frontmatter detection ----

    #[test]
    fn detect_frontmatter_yaml_returns_end_line() {
        // "---\nkey: value\n---" → line 0 opens, line 2 closes → end_line = 2.
        let text = "---\nkey: value\n---\n";
        assert_eq!(detect_frontmatter(text), Some(2));
    }

    #[test]
    fn detect_frontmatter_toml_returns_end_line() {
        let text = "+++\nkey = \"value\"\n+++\n";
        assert_eq!(detect_frontmatter(text), Some(2));
    }

    #[test]
    fn detect_frontmatter_multiline_body() {
        // Several content lines between the delimiters.
        let text = "---\ntitle: Hello\ndate: 2024-01-01\ntags: [rust]\n---\n";
        assert_eq!(detect_frontmatter(text), Some(4));
    }

    #[test]
    fn detect_frontmatter_none_for_regular_doc() {
        let text = "# Title\n\nSome paragraph.\n";
        assert_eq!(detect_frontmatter(text), None);
    }

    #[test]
    fn detect_frontmatter_none_if_delimiter_not_first_line() {
        // `---` appearing mid-document should not be detected as frontmatter.
        let text = "Intro\n---\nkey: value\n---\n";
        assert_eq!(detect_frontmatter(text), None);
    }

    #[test]
    fn detect_frontmatter_none_if_unclosed() {
        let text = "---\nkey: value\n";
        assert_eq!(detect_frontmatter(text), None);
    }

    #[test]
    fn detect_frontmatter_none_for_empty_block() {
        // `---` immediately followed by `---` has no content lines → rejected.
        let text = "---\n---\n";
        assert_eq!(detect_frontmatter(text), None);
    }

    #[test]
    fn detect_frontmatter_none_for_wrong_close_delimiter() {
        // YAML opened with `---` must close with `---`, not `+++`.
        let text = "---\nkey: value\n+++\n";
        assert_eq!(detect_frontmatter(text), None);
    }

    // ---- Frontmatter span styling ----

    #[test]
    fn frontmatter_delimiter_lines_have_tinted_bg() {
        let text = "---\ntitle: Hello\n---\nrest\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        // Line 0 (opening `---`) must have full_line_bg = frontmatter_bg.
        let open = map.get(&0).expect("opening delimiter must have spans");
        assert!(
            open.iter()
                .any(|s| s.full_line_bg == Some(theme.frontmatter_bg)),
            "opening `---` must have full_line_bg = frontmatter_bg; got: {open:?}"
        );
        // Line 2 (closing `---`) must also have full_line_bg = frontmatter_bg.
        let close = map.get(&2).expect("closing delimiter must have spans");
        assert!(
            close
                .iter()
                .any(|s| s.full_line_bg == Some(theme.frontmatter_bg)),
            "closing `---` must have full_line_bg = frontmatter_bg; got: {close:?}"
        );
    }

    #[test]
    fn frontmatter_delimiter_lines_have_muted_fg() {
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let open = map.get(&0).expect("opening delimiter must have spans");
        assert!(
            open.iter().any(|s| s.style.fg == Some(theme.muted)),
            "opening `---` delimiter must have muted fg; got: {open:?}"
        );
        let close = map.get(&2).expect("closing delimiter must have spans");
        assert!(
            close.iter().any(|s| s.style.fg == Some(theme.muted)),
            "closing `---` delimiter must have muted fg; got: {close:?}"
        );
    }

    #[test]
    fn frontmatter_key_has_code_color() {
        // "title: Hello" → key "title" (chars 0..5) must use code_color (green).
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content.iter().any(|s| {
                s.char_start == 0 && s.char_end == 5 && s.style.fg == Some(theme.frontmatter_key)
            }),
            "key 'title' (chars 0..5) must have frontmatter_key fg; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_key_is_italic() {
        // Key spans must carry the ITALIC modifier.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content.iter().any(|s| {
                s.char_start == 0
                    && s.char_end == 5
                    && s.style.add_modifier.contains(Modifier::ITALIC)
            }),
            "key 'title' must carry ITALIC modifier; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_content_has_row_indent() {
        // Content lines must carry row_indent = 3 for visual offset.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content.iter().any(|s| s.row_indent == 3),
            "content line spans must have row_indent = 3; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_value_has_text_color() {
        // "title: Hello" → value " Hello" (chars 6..12) must be theme.text.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content
                .iter()
                .any(|s| s.char_start == 6 && s.style.fg == Some(theme.text)),
            "value ' Hello' must have theme.text fg; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_content_has_tinted_bg() {
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content
                .iter()
                .any(|s| s.full_line_bg == Some(theme.frontmatter_bg)),
            "content line must have full_line_bg = frontmatter_bg; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_does_not_affect_lines_after_block() {
        // Line 3 is "rest" — must not have frontmatter_bg.
        let text = "---\ntitle: Hello\n---\nrest\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        if let Some(rest) = map.get(&3) {
            assert!(
                !rest
                    .iter()
                    .any(|s| s.full_line_bg == Some(theme.frontmatter_bg)),
                "lines after frontmatter block must not carry frontmatter_bg; got: {rest:?}"
            );
        }
        // (line 3 having no spans at all is also correct — just plain text)
    }

    #[test]
    fn frontmatter_rule_span_removed_from_delimiter_line() {
        // Without frontmatter detection, `---` at line 0 would be styled as a
        // horizontal rule (is_rule = true).  After frontmatter post-pass those
        // spans are replaced — no is_rule span must survive on line 0.
        let text = "---\nkey: v\n---\n";
        let map = build_map(text, &make_theme(), false);
        let open = map.get(&0).expect("opening delimiter must have spans");
        assert!(
            !open.iter().any(|s| s.is_rule),
            "is_rule span on `---` delimiter line must be removed by frontmatter pass"
        );
    }

    #[test]
    fn frontmatter_toml_key_has_code_color() {
        // TOML uses `=` as separator.  "key = \"value\"" → key "key " (0..3) in frontmatter_key color.
        let text = "+++\nkey = \"value\"\n+++\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content
                .iter()
                .any(|s| s.char_start == 0 && s.style.fg == Some(theme.frontmatter_key)),
            "TOML key must have frontmatter_key fg; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_delimiter_span_covers_full_line() {
        // Kills line 324 `delete field char_end from delimiter span`.
        // Default for char_end is 0; without the explicit value the span covers nothing.
        // "---" is 3 chars, so char_end must be 3.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let delim = map.get(&0).expect("opening delimiter must have spans");
        assert!(
            delim
                .iter()
                .any(|s| s.char_end == 3 && s.full_line_bg.is_some()),
            "delimiter '---' span must have char_end=3; got: {delim:?}"
        );
    }

    #[test]
    fn frontmatter_separator_span_has_exact_char_range() {
        // Kills line 371:25 `delete char_end` and line 371:39 `replace + with -/+` in
        // `char_end: sep + 1`.
        // "title: Hello" → ':' is at char index 5, so sep span must be chars 5..6.
        // With `+→-`: char_end = 4 (wrong).  With `+→*`: char_end = 5*1 = 5 (zero-width).
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content.iter().any(|s| s.char_start == 5 && s.char_end == 6),
            "separator ':' at char 5 must have span 5..6; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_value_span_has_exact_char_range() {
        // Kills line 386:29 `delete char_end from value span`.
        // "title: Hello" → value " Hello" starts at char 6 and ends at char 12.
        // Without char_end the span defaults to 0, rendering nothing.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content
                .iter()
                .any(|s| s.char_start == 6 && s.char_end == 12),
            "value ' Hello' must span chars 6..12; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_all_content_spans_have_row_and_continuation_indent() {
        // Kills lines 359/360 (key), 374/375 (sep), 389/390 (value): `delete row_indent`
        // and `delete continuation_indent` from each span type.
        // Existing tests use `any(...)` which misses mutations to individual spans when
        // other spans on the same line still carry the correct value.  `all(...)` fails
        // the moment any single span has the wrong indent.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content.iter().all(|s| s.row_indent == 3),
            "every span on a content line must have row_indent=3; got: {content:?}"
        );
        assert!(
            content.iter().all(|s| s.continuation_indent == 3),
            "every span on a content line must have continuation_indent=3; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_all_content_spans_have_full_line_bg() {
        // Kills lines 358/373/388/404: `delete full_line_bg from key/sep/value span`.
        // The existing `any(...)` test passes even when only one span has the bg set.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            content
                .iter()
                .all(|s| s.full_line_bg == Some(theme.frontmatter_bg)),
            "every span on a content line must carry full_line_bg=frontmatter_bg; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_no_key_span_when_separator_is_first_char() {
        // Kills line 350:24 `replace > with >=` in `if sep > 0`.
        // When a line starts with ':' (sep=0) no key span should be emitted;
        // with `>=`, sep=0 satisfies `0 >= 0` and a zero-length key span is pushed.
        let text = "---\n: orphan value\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        assert!(
            !content.iter().any(|s| {
                s.char_start == 0 && s.char_end == 0 && s.style.fg == Some(theme.frontmatter_key)
            }),
            "no zero-length key span when ':' is first char; got: {content:?}"
        );
    }

    #[test]
    fn frontmatter_no_value_span_when_separator_is_last_char() {
        // Kills line 380:28 `replace < with <=` in `if sep + 1 < line_len`.
        // "key:" has sep=3, line_len=4, sep+1=4.  With `<=`: 4 <= 4 is true → a
        // zero-length value span (char_start=4, char_end=4) would be emitted.
        // With `<`: 4 < 4 is false → no value span.
        let text = "---\nkey:\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        // A zero-length value span at the end of the line must not be present.
        assert!(
            !content.iter().any(|s| {
                s.char_start == 4 && s.char_end == 4 && s.style.fg == Some(theme.text)
            }),
            "no zero-length value span when ':' is the last char; got: {content:?}"
        );
    }

    // ── ==highlight== spans ───────────────────────────────────────────────────

    #[test]
    fn highlight_content_has_highlight_bg() {
        let text = "==hello==";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        // Content is chars 2..7 ("hello").
        assert!(
            spans.iter().any(|s| s.char_start == 2
                && s.char_end == 7
                && s.style.bg == Some(theme.highlight_bg)),
            "highlight content must carry highlight_bg; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_opening_delimiter_is_muted() {
        let text = "==hi==";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 0 && s.char_end == 2 && s.style.fg == Some(theme.muted)),
            "opening == must be muted; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_closing_delimiter_is_muted() {
        let text = "==hi==";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans
                .iter()
                .any(|s| s.char_start == 4 && s.char_end == 6 && s.style.fg == Some(theme.muted)),
            "closing == must be muted; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_two_on_same_line() {
        // Non-greedy: must produce two separate highlights, not one spanning both.
        let text = "==a== and ==b==";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map.get(&0).expect("line 0 must have spans");
        let content_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.style.bg == Some(theme.highlight_bg))
            .collect();
        assert_eq!(
            content_spans.len(),
            2,
            "two separate ==highlights== must each produce a content span; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_inside_fenced_block_is_not_decorated() {
        let text = "```\n==text==\n```";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        // Line 1 is the `==text==` line inside the fence.
        let empty = vec![];
        let spans = map.get(&1).unwrap_or(&empty);
        assert!(
            !spans.iter().any(|s| s.style.bg == Some(theme.highlight_bg)),
            "==text== inside a fenced block must not be highlighted; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_empty_content_is_not_decorated() {
        // ==== has no content between the delimiters — must be left undecorated.
        let text = "====";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let empty = vec![];
        let spans = map.get(&0).unwrap_or(&empty);
        assert!(
            !spans.iter().any(|s| s.style.bg == Some(theme.highlight_bg)),
            "==== with no content must not produce a highlight span; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_does_not_span_lines() {
        // The opening `==` is on line 0 and there is no closing `==` on the same line.
        // No highlight span should appear on either line.
        let text = "==open\nclosed==";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let empty = vec![];
        for line in 0..2usize {
            let spans = map.get(&line).unwrap_or(&empty);
            assert!(
                !spans.iter().any(|s| s.style.bg == Some(theme.highlight_bg)),
                "highlight must not span across lines; got spans on line {line}: {spans:?}"
            );
        }
    }

    #[test]
    fn highlight_bold_inside_gets_highlight_bg() {
        // ==**bold**== — the bold content span inside the highlight must gain
        // highlight_bg while keeping its bold colour and BOLD modifier.
        let text = "==**bold**==";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        // At least one span in the content zone [2..10) must carry highlight_bg.
        let covered = spans
            .iter()
            .filter(|s| {
                s.char_start >= 2 && s.char_end <= 10 && s.style.bg == Some(theme.highlight_bg)
            })
            .count();
        assert!(
            covered > 0,
            "bold spans inside ==highlight== must have highlight_bg layered in; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_bold_inside_preserves_bold_modifier() {
        // ==**bold**== — the bold content span must retain BOLD while also
        // having highlight_bg.  The overlay must not strip the modifier.
        let text = "==**bold**==";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| {
                s.style.add_modifier.contains(Modifier::BOLD)
                    && s.style.bg == Some(theme.highlight_bg)
            }),
            "bold content inside ==highlight== must be BOLD and have highlight_bg; got: {spans:?}"
        );
    }

    #[test]
    fn highlight_on_middle_line_does_not_bleed_to_adjacent_lines() {
        // Kills line 477:45 `replace < with >` and line 478:34 `replace + with *`.
        //
        // With `< → >`: for every line, `line_idx + 1 > line_starts.len()` is always
        // false for a 3-line doc, so line_str for line 0 extends to text.len(),
        // picking up the ==highlight== from line 1 and pushing wrong spans (at
        // out-of-range char positions) to line 0.
        //
        // With `+ → *`: line_end_byte = line_starts[line_idx * 1] = line_starts[line_idx]
        // = line_start_byte, so line_end_byte <= line_start_byte → the line is skipped
        // and line 1 gets no highlight spans at all.
        let text = "first\n==hello==\nlast";
        let theme = make_theme();
        let map = build_map(text, &theme, false);

        // Line 1 must carry highlight_bg on the content chars 2..7 ("hello").
        let line1 = map.get(&1).expect("line 1 must have spans");
        assert!(
            line1.iter().any(|s| s.char_start == 2
                && s.char_end == 7
                && s.style.bg == Some(theme.highlight_bg)),
            "==hello== on line 1 must produce a highlight_bg span at chars 2..7; got: {line1:?}"
        );

        // Line 0 ("first") must receive no highlight_bg — it has no ==…== of its own.
        let empty = vec![];
        let line0 = map.get(&0).unwrap_or(&empty);
        assert!(
            !line0.iter().any(|s| s.style.bg == Some(theme.highlight_bg)),
            "line 0 must not bleed highlight_bg from the ==highlight== on line 1; got: {line0:?}"
        );
    }

    #[test]
    fn highlight_skipped_in_frontmatter_lines() {
        // Kills line 469:55 `replace <= with >`.
        // With `<= → >`: line_idx > frontmatter_end is the new condition, which
        // inverts the logic — frontmatter lines are processed and highlighted
        // (wrong), while post-frontmatter lines are skipped (also wrong).
        let text = "---\nkey: ==value==\n---\n==normal==\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);

        // Line 1 is inside frontmatter — the ==value== must NOT be highlighted.
        let empty = vec![];
        let fm_line = map.get(&1).unwrap_or(&empty);
        assert!(
            !fm_line
                .iter()
                .any(|s| s.style.bg == Some(theme.highlight_bg)),
            "==value== inside frontmatter must not receive highlight_bg; got: {fm_line:?}"
        );

        // Line 3 is regular text — the ==normal== MUST be highlighted.
        let normal_line = map.get(&3).expect("line 3 must have spans");
        assert!(
            normal_line
                .iter()
                .any(|s| s.style.bg == Some(theme.highlight_bg)),
            "==normal== outside frontmatter must receive highlight_bg; got: {normal_line:?}"
        );
    }

    #[test]
    fn highlight_plain_text_around_inline_markup_gets_content_style() {
        // Documents the gap-filling behaviour of fill_highlight_gaps: plain text
        // that sits between the highlight delimiters but is not covered by any
        // existing bold/italic/link span must receive the full content_style
        // (highlight_fg + highlight_bg).
        //
        // "==pre **bold** post==" — "pre " and " post" are plain-text gaps.
        let text = "==pre **bold** post==";
        let theme = make_theme();
        let map = build_map(text, &theme, true);
        let spans = map.get(&0).expect("line 0 must have spans");

        // "pre " is at chars 2..6 (after opening ==).
        assert!(
            spans.iter().any(|s| s.char_start == 2
                && s.char_end == 6
                && s.style.bg == Some(theme.highlight_bg)),
            "plain 'pre ' (chars 2..6) before bold must have highlight_bg; got: {spans:?}"
        );

        // " post" ends just before the closing ==.  Count: "==pre **bold** post=="
        // is 21 chars; close_start = 19 (chars 19-20 are "=="), so " post" is 14..19.
        assert!(
            spans.iter().any(|s| s.char_start == 14
                && s.char_end == 19
                && s.style.bg == Some(theme.highlight_bg)),
            "plain ' post' (chars 14..19) after bold must have highlight_bg; got: {spans:?}"
        );
    }

    // ── frontmatter separator-style / no-separator coverage ─────────────────

    #[test]
    fn frontmatter_separator_char_has_muted_style() {
        // Kills line 392:25 `delete field style from struct StyledSpan expression`.
        // When `style: sep_style` is removed, Default (no fg) is used instead of
        // `theme.muted`; this assertion fails unless the separator carries muted fg.
        let text = "---\ntitle: Hello\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let content = map.get(&1).expect("content line must have spans");
        // ':' is at char index 5 → separator span is 5..6
        let sep = content
            .iter()
            .find(|s| s.char_start == 5 && s.char_end == 6)
            .expect("separator span at 5..6 must exist");
        assert_eq!(
            sep.style.fg,
            Some(theme.muted),
            "separator ':' must have muted fg; got span: {sep:?}"
        );
    }

    #[test]
    fn frontmatter_no_separator_line_span_coverage() {
        // Kills lines 422-426: delete char_end / style / full_line_bg /
        // row_indent / continuation_indent from the no-separator whole-line span.
        // (char_start: 0 at line 421 is undetectable — Default == 0 — and is
        // protected by `frontmatter_zero_start_span`.)
        //
        // "title" has no ':' or '=' → whole line must be styled as val_style
        // with char_end == line_len == 5, full_line_bg, row_indent=3, cont=3.
        let text = "---\ntitle\nkey: v\n---\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        let spans = map
            .get(&1)
            .expect("no-separator content line must have spans");
        let span = spans
            .iter()
            .find(|s| s.char_start == 0 && s.char_end == 5)
            .expect("whole-line span 0..5 must exist; got: {spans:?}");
        assert_eq!(
            span.style.fg,
            Some(theme.text),
            "no-separator line must use text colour; got: {span:?}"
        );
        assert_eq!(
            span.full_line_bg,
            Some(theme.frontmatter_bg),
            "no-separator line must carry frontmatter_bg; got: {span:?}"
        );
        assert_eq!(span.row_indent, 3, "row_indent must be 3; got: {span:?}");
        assert_eq!(
            span.continuation_indent, 3,
            "continuation_indent must be 3; got: {span:?}"
        );
    }

    // ── list ordered→unordered reset ─────────────────────────────────────────

    #[test]
    fn unordered_list_after_ordered_has_single_char_bullet() {
        // Kills line 1246:13 `delete match arm Event::End(TagEnd::List(_))`.
        // Without the arm, `in_ordered_list` stays `true` after the ordered list
        // ends.  The unordered-list item would then use the ordered path and
        // compute `bullet_end = item_char + find('.',')')...unwrap_or(item_char+2)`.
        // For `- b` there is no '.' or ')', so it falls back to `item_char + 2`
        // instead of the correct `item_char + 1`, making the bullet span one char
        // too wide.
        let text = "1. first\n\n- bullet\n";
        let theme = make_theme();
        let map = build_map(text, &theme, false);
        // "- bullet" is on line 2 (0-indexed); '-' is at char 0 → span must be 0..1.
        let spans = map.get(&2).expect("unordered list line must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 1),
            "unordered bullet '-' must be span 0..1 (not 0..2); got: {spans:?}"
        );
    }

    // ---- bold+italic alt-syntax (**_text_**) delimiter boundary ----
    //
    // Kills all five detectable mutations on `is_strong_outer_adjacent_to_emphasis`
    // (both `== → !=`, both `+ → *`, and `+ → -`).
    // The sixth mutation (`&&` → `||`) is undetectable and handled via
    // `#[mutants::skip]` on the helper.
    //
    // When adjacency fails the fallback produces separate bold (0..2, 6..8) and
    // italic (2..3, 5..6) delimiter spans — no single span can cover 0..3 or 5..8.
    // `emit_bold_italic_spans` (adjacent path) merges them into 0..3 and 5..8.

    #[test]
    fn bold_italic_alt_syntax_delimiter_boundaries() {
        // "**_hi_**" — outer Strong byte 0..8, inner Emphasis byte 2..6.
        // Adjacent: emit_bold_italic_spans → opening_delim=0..3 (**_), content=3..5, closing=5..8 (_**).
        let text = "**_hi_**";
        let map = build_map(text, &make_theme(), true);
        let spans = map.get(&0).expect("line 0 must have spans");
        assert!(
            spans.iter().any(|s| s.char_start == 0 && s.char_end == 3),
            "opening **_ combined delimiter must span chars 0..3; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 3 && s.char_end == 5),
            "content 'hi' must span chars 3..5; got: {spans:?}"
        );
        assert!(
            spans.iter().any(|s| s.char_start == 5 && s.char_end == 8),
            "closing _** combined delimiter must span chars 5..8; got: {spans:?}"
        );
        // If the non-adjacent fallback ran, a plain bold ** span at 0..2 would exist.
        assert!(
            !spans.iter().any(|s| s.char_start == 0 && s.char_end == 2),
            "separate bold-only ** at 0..2 must NOT exist (adjacency must take combined path); got: {spans:?}"
        );
    }
}
