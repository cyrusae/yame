use ratatui::style::Modifier;
use yame::config::Theme;
use yame::decoration::{build_decoration_map, count_words};

/// Full decoration pass against the fixture file.
/// Confirms headings, bold, blockquotes, and word count all survive the pipeline.
#[test]
fn fixture_decoration_roundtrip() {
    let text = include_str!("fixtures/sample.md");
    let theme = Theme::default_theme();
    let map = build_decoration_map(text, &theme, true, 9999); // cursor far away

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
        count_words(text) > 100,
        "word count was {} — expected >100",
        count_words(text)
    );
}
