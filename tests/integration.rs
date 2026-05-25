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
        .filter(|spans| spans.iter().any(|s| s.full_line_bg == Some(theme.heading_bg)))
        .count();
    assert!(
        heading_lines >= 4,
        "expected ≥4 heading lines with heading_bg, got {heading_lines}"
    );
}
