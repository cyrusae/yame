/// Headless single-frame renderer for `yame --preview <file>`.
///
/// Loads a file, builds the full decoration / syntax-highlight map (the same
/// pipeline used in the interactive editor), then emits styled ANSI output to
/// stdout — one line at a time, no alternate screen, no raw mode.
///
/// Designed for use as an lf (or any file-manager) previewer:
///
/// ```text
/// # ~/.config/lf/lfrc
/// set previewer yame --preview
/// ```
///
/// Width comes from `$COLUMNS` → `crossterm::terminal::size()` → 80 fallback.
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

use yame::app::{FileMode, is_likely_binary, load_file, resolve_file_mode};
use yame::config::{Theme, load_config, supports_italic};
use yame::decoration::{
    DecorationMap, StyledSpan, block_highlights_to_decoration_map, build_decoration_map,
    count_words,
};
use yame::renderer::{split_into_spans, wrap_char_ranges, wrap_line_indented};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Render `path` to stdout with ANSI colour codes and exit.
///
/// Prints an error message to stderr and returns an `Err` if the file is
/// binary, unreadable, or if stdout cannot be written to.
#[mutants::skip] // Filesystem + stdout I/O — mutations masked by OS state.
pub(super) fn render_preview(path: &Path) -> io::Result<()> {
    if is_likely_binary(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("'{}' appears to be a binary file", path.display()),
        ));
    }

    // ── Theme & config ───────────────────────────────────────────────────────
    let (config, _warnings) = load_config();
    let italic_support = supports_italic();
    let mut warn_sink = vec![];
    let theme = Theme::from_config(
        &config.palette,
        &config.theme,
        &config.headings,
        &mut warn_sink,
    );

    // ── Load file ────────────────────────────────────────────────────────────
    let tab_width = config.layout.tab_width.unwrap_or(4) as usize;
    let textarea = load_file(path, tab_width)?;
    let text = textarea.lines().join("\n");

    // ── Syntax-highlight cache ───────────────────────────────────────────────
    let highlight_cache = config.highlighting.enabled.then(|| {
        let palette_theme = config
            .highlighting
            .use_palette_colors
            .then(|| yame::highlighting::build_palette_theme(&theme));
        Arc::new(yame::highlighting::HighlightCache::new(
            true,
            config.highlighting.syntect_theme.clone(),
            palette_theme,
        ))
    });

    // ── Decoration map ───────────────────────────────────────────────────────
    let file_mode = resolve_file_mode(path, &config.filetype);
    let (decoration_map, _word_count) = match &file_mode {
        FileMode::Markdown => {
            build_decoration_map(&text, &theme, italic_support, highlight_cache.as_deref())
        }
        FileMode::PlainHighlight(lang) => {
            let map = highlight_cache
                .as_deref()
                .and_then(|cache| cache.highlight_block(lang, &text))
                .map(|hl| block_highlights_to_decoration_map(&hl, 0))
                .unwrap_or_default();
            let wc = count_words(&text);
            (map, wc)
        }
        FileMode::PlainText => (DecorationMap::default(), count_words(&text)),
    };

    // ── Render ───────────────────────────────────────────────────────────────
    let term_width = preview_width();
    let default_style = Style::default().fg(theme.text);

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    render_lines(
        textarea.lines(),
        &decoration_map,
        default_style,
        &theme,
        term_width,
        &mut out,
    )?;

    out.flush()
}

// ---------------------------------------------------------------------------
// Core line rendering (pub(crate) for unit-testing)
// ---------------------------------------------------------------------------

/// Render every logical line to `out` using ANSI escape codes.
///
/// Supports the full set of decoration properties used by the interactive
/// renderer, giving feature parity with the main editor:
///
/// - **`full_line_bg`** — heading / code-block / frontmatter background;
///   emitted as a 24-bit ANSI bg sequence that extends to the end of each
///   terminal line via `\x1b[K`.
/// - **`border_bottom`** — H1–H3 heading underline; emitted as a full-width
///   `─` separator rule in the border colour on the line immediately after
///   the heading.
/// - **`row_indent`** — visual offset for the first row of frontmatter
///   content; emitted as leading spaces.
/// - **`continuation_indent`** — left-margin for soft-wrapped continuation
///   rows of list items and blockquotes; emitted as leading spaces, and the
///   wrap width is narrowed by the same amount so content never overflows.
/// - Inline **`style.bg`** (inline code, etc.) — included in the ANSI
///   sequence emitted by [`style_open`] so each span carries its own bg.
///
/// A trailing newline is emitted after every logical line.  Rules and
/// borders each occupy exactly one output line.
pub(crate) fn render_lines(
    lines: &[String],
    decoration_map: &DecorationMap,
    default_style: Style,
    theme: &Theme,
    term_width: usize,
    out: &mut impl Write,
) -> io::Result<()> {
    for (line_idx, line) in lines.iter().enumerate() {
        let deco = decoration_map.get(&line_idx);

        // ── Horizontal rule ──────────────────────────────────────────────────
        let is_rule = deco.map(|s| s.iter().any(|sp| sp.is_rule)).unwrap_or(false);
        if is_rule {
            let rule_style = deco
                .and_then(|s| s.iter().find(|sp| sp.is_rule))
                .map(|sp| sp.style)
                .unwrap_or_else(|| Style::default().fg(theme.muted));
            let rule = "─".repeat(term_width.max(1));
            let open = style_open(&rule_style);
            if open.is_empty() {
                writeln!(out, "{rule}")?;
            } else {
                writeln!(out, "{open}{rule}\x1b[0m")?;
            }
            continue;
        }

        // ── Line-level decoration properties ────────────────────────────────
        let full_line_bg: Option<Color> = deco.and_then(|d| d.iter().find_map(|s| s.full_line_bg));
        let border_bottom: Option<Color> =
            deco.and_then(|d| d.iter().find_map(|s| s.border_bottom));
        let cont_indent: usize = deco
            .map(|d| d.iter().map(|s| s.continuation_indent).max().unwrap_or(0))
            .unwrap_or(0) as usize;
        let row_indent: usize = deco
            .map(|d| d.iter().map(|s| s.row_indent).max().unwrap_or(0))
            .unwrap_or(0) as usize;

        // Blockquote lines use theme.blockquote_color as their default fg, just
        // as the interactive renderer does (see renderer/mod.rs `row_default`).
        // Must be read from all logical-line spans (not filtered per visual row)
        // so continuation rows also inherit the colour.
        let is_blockquote_line = deco
            .map(|decs| decs.iter().any(|s| s.is_blockquote))
            .unwrap_or(false);

        // When a line has a background, incorporate it into the default style
        // so that gap spans (plain text between decorations) also carry the
        // line background.  This ensures `emit_spans` never produces a
        // background-free gap even after a mid-line `\x1b[0m` reset.
        let line_default_style = {
            let base = if is_blockquote_line {
                default_style.fg(theme.blockquote_color)
            } else {
                default_style
            };
            match full_line_bg {
                Some(bg) => base.bg(bg),
                None => base,
            }
        };

        // ── Word-wrap accounting for indentation ─────────────────────────────
        // term_width == 0 is used in tests to disable wrapping entirely.
        let (visual_rows, char_ranges): (Vec<&str>, Vec<(usize, usize)>) = if term_width == 0 {
            (vec![line.as_str()], vec![(0, line.chars().count())])
        } else {
            let first_w = term_width.saturating_sub(row_indent).max(1);
            let cont_w = term_width.saturating_sub(cont_indent).max(1);
            let rows = wrap_line_indented(line, first_w, cont_w);
            let ranges = wrap_char_ranges(line, &rows);
            (rows, ranges)
        };

        // ── Emit each visual row ─────────────────────────────────────────────
        for (wrap_idx, (&row_str, &(char_start, char_len))) in
            visual_rows.iter().zip(char_ranges.iter()).enumerate()
        {
            let char_end = char_start + char_len;
            // First row uses row_indent (frontmatter visual offset); subsequent
            // rows use cont_indent (list / blockquote continuation alignment).
            let indent = if wrap_idx == 0 {
                row_indent
            } else {
                cont_indent
            };

            // Narrow decoration spans to this visual row's char range and
            // adjust char positions to be relative to the row start — exactly
            // the same transform used by the interactive renderer.
            let row_spans: Vec<StyledSpan> = deco
                .map(|decs| {
                    decs.iter()
                        .filter(|s| s.char_end > char_start && s.char_start < char_end)
                        .map(|s| StyledSpan {
                            char_start: s.char_start.saturating_sub(char_start),
                            char_end: s.char_end.saturating_sub(char_start).min(char_len),
                            style: s.style,
                            ..Default::default()
                        })
                        .collect()
                })
                .unwrap_or_default();

            let ratatui_spans = split_into_spans(row_str, &row_spans, line_default_style);

            // Open line background (still active for the indent spaces below).
            if let Some(Color::Rgb(r, g, b)) = full_line_bg {
                write!(out, "\x1b[48;2;{r};{g};{b}m")?;
            }
            // Leading indent.
            if indent > 0 {
                write!(out, "{}", " ".repeat(indent))?;
            }
            // Styled content (each span manages its own open + close).
            emit_spans(&ratatui_spans, out)?;
            // Re-open line bg and clear to end-of-line to fill trailing space,
            // then reset so the next line starts clean.
            if let Some(Color::Rgb(r, g, b)) = full_line_bg {
                write!(out, "\x1b[48;2;{r};{g};{b}m\x1b[K\x1b[0m")?;
            }
            writeln!(out)?;
        }

        // ── Heading bottom border ────────────────────────────────────────────
        // Emitted as a full-width `─` separator immediately after the last
        // visual row of the heading — equivalent to the underline the
        // interactive renderer draws on heading cells.
        if let Some(Color::Rgb(r, g, b)) = border_bottom {
            let w = term_width.max(1);
            writeln!(out, "\x1b[38;2;{r};{g};{b}m{}\x1b[0m", "─".repeat(w))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a sequence of styled `Span`s to `out`, resetting ANSI state after
/// each coloured chunk.  A trailing newline is NOT emitted here.
fn emit_spans(spans: &[Span<'_>], out: &mut impl Write) -> io::Result<()> {
    for span in spans {
        let open = style_open(&span.style);
        if open.is_empty() {
            write!(out, "{}", span.content)?;
        } else {
            write!(out, "{open}{}\x1b[0m", span.content)?;
        }
    }
    Ok(())
}

/// Determine the preview column width.
///
/// Resolution order:
/// 1. `$COLUMNS` environment variable (set by most shells).
/// 2. `crossterm::terminal::size()` (works without raw mode on most systems).
/// 3. Hard-coded default of 80.
pub(crate) fn preview_width() -> usize {
    if let Some(n) = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| parse_columns(&s))
    {
        return n;
    }
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

/// Parse a `$COLUMNS` string into a positive column count.
///
/// Returns `None` if the string is not a valid integer or parses to zero.
fn parse_columns(s: &str) -> Option<usize> {
    s.trim().parse::<usize>().ok().filter(|&n| n > 0)
}

/// Convert a ratatui [`Style`] to an ANSI SGR opening escape sequence.
///
/// Emits foreground colour, background colour, and the four common text
/// modifiers (bold, italic, underline, strikethrough).
///
/// Returns an empty string when the style carries no visual information so
/// callers can skip the reset too.
pub(crate) fn style_open(style: &Style) -> String {
    let mut codes = String::new();

    if style.add_modifier.contains(Modifier::BOLD) {
        codes.push_str("1;");
    }
    if style.add_modifier.contains(Modifier::ITALIC) {
        codes.push_str("3;");
    }
    if style.add_modifier.contains(Modifier::UNDERLINED) {
        codes.push_str("4;");
    }
    if style.add_modifier.contains(Modifier::CROSSED_OUT) {
        codes.push_str("9;");
    }
    if let Some(Color::Rgb(r, g, b)) = style.fg {
        codes.push_str(&format!("38;2;{r};{g};{b};"));
    }
    if let Some(Color::Rgb(r, g, b)) = style.bg {
        codes.push_str(&format!("48;2;{r};{g};{b};"));
    }

    if codes.is_empty() {
        return String::new();
    }

    codes.pop(); // strip trailing semicolon
    format!("\x1b[{codes}m")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Span;

    use yame::config::Theme;
    use yame::decoration::{DecorationMap, StyledSpan};

    use super::{emit_spans, render_lines, style_open};

    // ── style_open ───────────────────────────────────────────────────────────

    // Kills: returning non-empty string when style is default.
    #[test]
    fn style_open_empty_for_default_style() {
        assert_eq!(
            style_open(&Style::default()),
            "",
            "default style must produce no ANSI sequence"
        );
    }

    // Kills: wrong prefix / wrong colour channel order.
    #[test]
    fn style_open_rgb_fg_produces_correct_sequence() {
        let style = Style::default().fg(Color::Rgb(255, 128, 0));
        assert_eq!(
            style_open(&style),
            "\x1b[38;2;255;128;0m",
            "RGB foreground must produce correct 24-bit ANSI sequence"
        );
    }

    // Kills: bold code wrong or missing.
    #[test]
    fn style_open_bold_produces_code_1() {
        let style = Style::default().add_modifier(Modifier::BOLD);
        assert_eq!(style_open(&style), "\x1b[1m");
    }

    // Kills: italic code wrong or missing.
    #[test]
    fn style_open_italic_produces_code_3() {
        let style = Style::default().add_modifier(Modifier::ITALIC);
        assert_eq!(style_open(&style), "\x1b[3m");
    }

    // Kills: modifier + color separator wrong (double semicolon or missing).
    #[test]
    fn style_open_bold_and_rgb_combined() {
        let style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Rgb(100, 200, 50));
        let got = style_open(&style);
        assert_eq!(
            got, "\x1b[1;38;2;100;200;50m",
            "bold + RGB must be joined by semicolons with no leading/trailing junk"
        );
    }

    // Kills: background colour not emitted (48;2 sequence missing).
    #[test]
    fn style_open_emits_background_colour() {
        let style = Style::default().bg(Color::Rgb(30, 30, 30));
        assert_eq!(
            style_open(&style),
            "\x1b[48;2;30;30;30m",
            "background colour must produce a 48;2 ANSI sequence"
        );
    }

    // Kills: fg suppressed when bg is also set, or bg suppressed when fg set.
    #[test]
    fn style_open_emits_both_fg_and_bg() {
        let style = Style::default()
            .fg(Color::Rgb(200, 100, 50))
            .bg(Color::Rgb(30, 30, 30));
        let got = style_open(&style);
        assert_eq!(
            got, "\x1b[38;2;200;100;50;48;2;30;30;30m",
            "both fg and bg must be emitted when both are set"
        );
    }

    // ── emit_spans ───────────────────────────────────────────────────────────

    // Kills: omitting the reset `\x1b[0m` after a styled span.
    #[test]
    fn emit_spans_unstyled_span_has_no_ansi() {
        let spans = vec![Span::raw("hello")];
        let mut buf = Vec::new();
        emit_spans(&spans, &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "hello");
    }

    // Kills: not emitting the reset after a coloured span.
    #[test]
    fn emit_spans_styled_span_has_open_and_reset() {
        let style = Style::default().fg(Color::Rgb(1, 2, 3));
        let spans = vec![Span::styled("word", style)];
        let mut buf = Vec::new();
        emit_spans(&spans, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert!(
            out.starts_with("\x1b[38;2;1;2;3m"),
            "must open with ANSI fg"
        );
        assert!(out.ends_with("\x1b[0m"), "must close with reset");
        assert!(out.contains("word"), "must contain the span text");
    }

    // ── render_lines ─────────────────────────────────────────────────────────

    // Kills: skipping the newline at the end of each line.
    #[test]
    fn render_lines_each_line_ends_with_newline() {
        let lines = vec!["alpha".to_string(), "beta".to_string()];
        let map = DecorationMap::default();
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&lines, &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        // Strip any ANSI sequences to check structure.
        let plain: String = strip_ansi(&out);
        assert_eq!(plain, "alpha\nbeta\n");
    }

    // Kills: rendering wrong line content (e.g. transposing lines).
    #[test]
    fn render_lines_content_matches_input() {
        let lines = vec!["# Heading".to_string(), "body text".to_string()];
        let map = DecorationMap::default();
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&lines, &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let plain = strip_ansi(&out);
        let plain_lines: Vec<&str> = plain.lines().collect();
        assert_eq!(plain_lines[0], "# Heading");
        assert_eq!(plain_lines[1], "body text");
    }

    // Kills: cont_indent not applied, or applied on wrong rows.
    // A list line with cont_indent=2 that soft-wraps must have the first row
    // at column 0 and continuation rows indented by 2 spaces.
    #[test]
    fn render_lines_cont_indent_on_wrapped_list_item() {
        // "- word1 word2 word3 word4" with term_width=12, cont_indent=2.
        // "- word1 wor" fits in 12; "d2 word3 word4" continuation at 12-2=10.
        // Simpler: "- aaa bbb" at term_width=8, cont_indent=2.
        // first_w = 8, cont_w = 6.
        // wrap_line_indented("- aaa bbb", 8, 6) → ["- aaa", "bbb"]
        // (first row: "- aaa" = 5 chars ≤ 8; next word "bbb"=3 would give 5+1+3=9 > 8, wrap)
        // Wait, let me think about what wrap_line_indented actually does here.
        // It calls wrap_line("- aaa bbb", 8) = ["- aaa", "bbb"] since "- aaa bbb" = 9 > 8.
        // Then for the continuation, it re-wraps "bbb" at cont_w=6 → ["bbb"].
        // So visual_rows = ["- aaa", "bbb"], char_ranges = [(0,5),(6,3)].
        let line = "- aaa bbb".to_string();
        let mut map = DecorationMap::default();
        // Bullet span at chars 0..1 carries cont_indent=2.
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 1,
                continuation_indent: 2,
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 8, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        let plain = strip_ansi(&out);
        let rows: Vec<&str> = plain.lines().collect();
        // First row: no indent.
        assert_eq!(rows[0], "- aaa", "first row must not be indented");
        // Continuation row: 2 spaces indent + "bbb".
        assert_eq!(
            rows[1], "  bbb",
            "continuation row must have 2-space indent"
        );
    }

    // Kills: row_indent not applied, or applied on wrong rows.
    // Frontmatter content lines carry row_indent=3 to give a visual offset.
    #[test]
    fn render_lines_row_indent_on_frontmatter_content() {
        let line = "title: Hello".to_string();
        let mut map = DecorationMap::default();
        // Frontmatter span with row_indent=3 and cont_indent=3.
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 12,
                row_indent: 3,
                continuation_indent: 3,
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let plain = strip_ansi(&String::from_utf8(buf).unwrap());
        let rows: Vec<&str> = plain.lines().collect();
        // First (only) row: 3 spaces + content.
        assert!(
            rows[0].starts_with("   "),
            "first row must have 3-space row_indent; got: {:?}",
            rows[0]
        );
        assert!(rows[0].contains("title: Hello"));
    }

    // Kills: full_line_bg 48;2 sequence missing or wrong channel order.
    #[test]
    fn render_lines_full_line_bg_sequence_present() {
        let line = "# Heading".to_string();
        let mut map = DecorationMap::default();
        // Heading span with full_line_bg = RGB(20, 30, 40).
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 9,
                full_line_bg: Some(Color::Rgb(20, 30, 40)),
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        assert!(
            raw.contains("\x1b[48;2;20;30;40m"),
            "full_line_bg must produce a 48;2 ANSI bg sequence; got: {raw:?}"
        );
        // `\x1b[K` must appear to fill trailing columns with the background.
        assert!(
            raw.contains("\x1b[K"),
            "full_line_bg must include \\x1b[K to extend bg to EOL"
        );
    }

    // Kills: border_bottom rule line missing or wrong colour.
    #[test]
    fn render_lines_border_bottom_emits_rule_line() {
        let line = "# H1".to_string();
        let mut map = DecorationMap::default();
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 4,
                border_bottom: Some(Color::Rgb(100, 150, 200)),
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        // term_width=5: heading row + border row of 5 × '─'.
        render_lines(&[line], &map, Style::default(), &theme, 5, &mut buf).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        let plain = strip_ansi(&raw);
        let rows: Vec<&str> = plain.lines().collect();
        // rows[0] = "# H1", rows[1] = "─────" (5 chars).
        assert_eq!(rows.len(), 2, "border_bottom must add a second output line");
        assert_eq!(rows[1], "─────", "border row must be all ─ at term_width");
        // The border colour must appear in the raw output.
        assert!(
            raw.contains("\x1b[38;2;100;150;200m"),
            "border_bottom must use the specified fg colour"
        );
    }

    // Kills: inline bg (48;2) missing from span-level style.
    #[test]
    fn render_lines_inline_bg_emitted_for_span() {
        let line = "hello".to_string();
        let mut map = DecorationMap::default();
        // Span with only a bg colour (inline code style).
        let span_style = Style::default().bg(Color::Rgb(50, 60, 70));
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 5,
                style: span_style,
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        assert!(
            raw.contains("\x1b[48;2;50;60;70m"),
            "span with bg must emit a 48;2 sequence; got: {raw:?}"
        );
    }

    // Kills: blockquote_color not applied as default fg for blockquote lines.
    #[test]
    fn render_lines_blockquote_uses_blockquote_color() {
        // Real blockquote lines have a narrow indicator span covering the ">"
        // marker (char 0 only, with is_blockquote=true).  The plain text after
        // the indicator has no decoration span and falls through to the
        // line_default_style, which is where blockquote_color must appear.
        let line = "> plain text".to_string();
        let mut map = DecorationMap::default();
        // Indicator span: only covers the ">" character (char_start=0, char_end=1).
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 0,
                char_end: 1,
                style: Style::default().fg(Color::Rgb(80, 80, 80)), // muted indicator
                is_blockquote: true,
                ..Default::default()
            }],
        );
        let mut theme = Theme::default_theme();
        // Use a distinctive blockquote colour so we can detect it in the output.
        theme.blockquote_color = Color::Rgb(100, 150, 200);
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 40, &mut buf).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        // The blockquote fg colour must appear as a 38;2 sequence for the gap text.
        assert!(
            raw.contains("\x1b[38;2;100;150;200m"),
            "blockquote line must emit blockquote_color as fg for undecorated gap; got: {raw:?}"
        );
    }

    // ── render_lines span positioning ────────────────────────────────────────

    // Kills: delete field char_start from StyledSpan expression in render_lines.
    // When char_start is omitted, the adjusted struct gets char_start=0 (Default),
    // so a span in a soft-wrapped continuation row appears to start at column 0 of
    // that row instead of at its correct position within the row.
    //
    // "- aaa bbb" at term_width=8 wraps to:
    //   row 0: "- aaa"  char_range (0, 5)
    //   row 1: "bbb"    char_range (6, 3)   ← space at char 5 consumed by wrapping
    //
    // A span at original chars 7–9 covers the last two 'b's ("bb").
    // Correctly adjusted to row 1: [char_start=1, char_end=3] of "bbb".
    // Under delete-char_start: [char_start=0, char_end=3] → all "bbb" gets the color.
    //
    // Test: the first 'b' of row 1 must be unstyled (no ANSI before it).
    #[test]
    fn render_lines_span_char_start_adjusted_for_wrapped_rows() {
        let line = "- aaa bbb".to_string();
        let mut map = DecorationMap::default();
        let span_color = Color::Rgb(200, 100, 50);
        map.insert(
            0,
            vec![StyledSpan {
                char_start: 7,
                char_end: 9,
                style: Style::default().fg(span_color),
                ..Default::default()
            }],
        );
        let theme = Theme::default_theme();
        let mut buf = Vec::new();
        render_lines(&[line], &map, Style::default(), &theme, 8, &mut buf).unwrap();
        let raw = String::from_utf8(buf).unwrap();
        // The second newline-delimited segment is the "bbb" row.
        let row1 = raw.split('\n').nth(1).unwrap_or("");
        // First char must be unstyled: the row must start with 'b', not an ANSI escape.
        // Under delete-char_start, the whole "bbb" gets the span color, starting the row
        // with \x1b[38;2;...m instead of the plain 'b'.
        assert!(
            row1.starts_with('b'),
            "first 'b' of wrapped row must be unstyled (span starts at col 1, not 0); \
             got row1={row1:?}"
        );
        // The span color must still appear somewhere (span is actually applied to "bb").
        assert!(
            row1.contains("\x1b[38;2;200;100;50m"),
            "span color must appear in the wrapped row; got row1={row1:?}"
        );
    }

    // ── parse_columns ─────────────────────────────────────────────────────────

    // Kills: replace > with < or == in parse_columns (COLUMNS never used when
    // condition can't be satisfied for any positive usize).
    #[test]
    fn parse_columns_accepts_positive_value() {
        assert_eq!(
            super::parse_columns("100"),
            Some(100),
            "parse_columns must return Some(n) for a positive integer string"
        );
        assert_eq!(
            super::parse_columns("  80  "),
            Some(80),
            "parse_columns must trim whitespace"
        );
    }

    // Kills: replace > with >= in parse_columns (COLUMNS=0 would be returned as 0).
    #[test]
    fn parse_columns_rejects_zero() {
        assert!(
            super::parse_columns("0").is_none(),
            "parse_columns must return None for 0 — a zero-width terminal is invalid"
        );
    }

    // Sanity: non-numeric input must return None.
    #[test]
    fn parse_columns_rejects_non_numeric() {
        assert!(super::parse_columns("abc").is_none());
        assert!(super::parse_columns("").is_none());
    }

    // ── preview_width ─────────────────────────────────────────────────────────

    // Kills: replace preview_width -> usize with 1 (and other small-constant mutations).
    // In a typical test environment (no attached terminal, no COLUMNS override), the
    // crossterm fallback produces 80.  We assert >= 2 to survive narrow-terminal CI
    // environments while still failing if the whole function is replaced with a constant.
    #[test]
    fn preview_width_is_at_least_two() {
        let w = super::preview_width();
        assert!(w >= 2, "preview_width must be at least 2, got {w}");
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Strip ANSI escape sequences for plain-text comparison in tests.
    fn strip_ansi(s: &str) -> String {
        let mut out = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                // Skip until the command byte (letter after the CSI sequence).
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }
}
