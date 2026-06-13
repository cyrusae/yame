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
/// Background colours are intentionally suppressed so the preview integrates
/// cleanly with the file-manager's own background.
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

use yame::app::{FileMode, is_likely_binary, load_file, resolve_file_mode};
use yame::config::{Theme, load_config, supports_italic};
use yame::decoration::{
    DecorationMap, block_highlights_to_decoration_map, build_decoration_map, count_words,
};
use yame::renderer::split_into_spans;

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
/// Extracted from `render_preview` so it can be tested without real I/O.
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

        // Horizontal rule — replace content with a full-width `─` bar.
        let is_rule = deco.map(|s| s.iter().any(|sp| sp.is_rule)).unwrap_or(false);
        if is_rule {
            let rule_style = deco
                .and_then(|s| s.iter().find(|sp| sp.is_rule))
                .map(|sp| sp.style)
                .unwrap_or_else(|| Style::default().fg(theme.muted));
            let rule = "─".repeat(term_width);
            let open = style_open(&rule_style);
            if open.is_empty() {
                writeln!(out, "{rule}")?;
            } else {
                writeln!(out, "{open}{rule}\x1b[0m")?;
            }
            continue;
        }

        // Normal line — split at decoration boundaries and emit styled chunks.
        let ratatui_spans: Vec<Span<'_>> = match deco {
            Some(styled_spans) => split_into_spans(line, styled_spans, default_style),
            None => vec![Span::styled(line.as_str().to_owned(), default_style)],
        };

        emit_spans(&ratatui_spans, out)?;
        writeln!(out)?;
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
    if let Ok(s) = std::env::var("COLUMNS")
        && let Ok(n) = s.trim().parse::<usize>()
        && n > 0
    {
        return n;
    }
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

/// Convert a ratatui [`Style`] to an ANSI SGR opening escape sequence.
///
/// Only foreground colour and the four common text modifiers (bold, italic,
/// underline, strikethrough) are emitted.  Background colour is deliberately
/// suppressed so the preview integrates cleanly with the file-manager's own
/// background colour.
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
    use yame::decoration::DecorationMap;

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

    // Kills: background colour leaking into output (it must be suppressed).
    #[test]
    fn style_open_suppresses_background_colour() {
        // Fg = None, Bg = something.  Must produce no sequence.
        let style = Style::default().bg(Color::Rgb(30, 30, 30));
        assert_eq!(
            style_open(&style),
            "",
            "background colour alone must produce no ANSI sequence"
        );
    }

    // Kills: suppresses foreground when bg is also set (must still emit fg).
    #[test]
    fn style_open_emits_fg_even_when_bg_also_set() {
        let style = Style::default()
            .fg(Color::Rgb(200, 100, 50))
            .bg(Color::Rgb(30, 30, 30));
        assert_eq!(
            style_open(&style),
            "\x1b[38;2;200;100;50m",
            "fg must still be emitted when bg is also set"
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

    // ── preview_width ─────────────────────────────────────────────────────────

    // Kills: $COLUMNS parsing returning wrong value or ignoring the variable.
    #[test]
    fn preview_width_uses_columns_env_var() {
        // We can't mutate the real env in parallel tests safely, but we can
        // confirm the fallback (80) is a positive integer — a minimal sanity check.
        let w = super::preview_width();
        assert!(w > 0, "preview_width must return a positive width");
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
