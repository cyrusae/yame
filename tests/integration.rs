use ratatui::style::Modifier;
use yame::config::Theme;
use yame::decoration::build_decoration_map;

/// Full decoration pass against the fixture file.
/// Confirms headings, bold, blockquotes, and word count all survive the pipeline.
#[test]
fn fixture_decoration_roundtrip() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let (map, word_count) = build_decoration_map(text, &theme, true);

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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &theme, true);
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
    let (map, _) = build_decoration_map(text, &custom_theme, true);
    assert!(
        map.values()
            .flatten()
            .any(|s| s.full_line_bg == Some(custom_theme.heading_bg)),
        "expected at least one heading span with the custom heading_bg in the decoration map"
    );
}
