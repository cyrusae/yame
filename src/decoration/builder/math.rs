use ratatui::style::Style;

use crate::config::blend_colors;
use crate::decoration::spans::{byte_to_line_char, line_char_len, make_span, push_span};
use crate::decoration::types::{DecorationMap, StyledSpan};

use super::BuildState;

// ---------------------------------------------------------------------------
// ---- n. LaTeX math ($…$ and $$…$$) ----
// ---------------------------------------------------------------------------

/// `$…$` — pulldown-cmark `Event::InlineMath`.  `range` covers the delimiters.
pub(super) fn on_inline_math(s: &mut BuildState, range: std::ops::Range<usize>) {
    emit_math(s, range, 1);
}

/// `$$…$$` — pulldown-cmark `Event::DisplayMath`.  `range` covers the delimiters.
pub(super) fn on_display_math(s: &mut BuildState, range: std::ops::Range<usize>) {
    emit_math(s, range, 2);
}

/// True when `text[line_start..byte]` is whitespace only — i.e. `byte` is the
/// first non-blank position on its line.
fn only_ws_before(text: &str, line_start: usize, byte: usize) -> bool {
    text[line_start..byte].chars().all(char::is_whitespace)
}

/// True when everything from `byte` to the end of its line is whitespace.
fn only_ws_after(text: &str, byte: usize) -> bool {
    text[byte..]
        .chars()
        .take_while(|&c| c != '\n')
        .all(char::is_whitespace)
}

/// Token class inside a math region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MathTok {
    /// Variables, digits, operators — drawn in `math_color`.
    Body,
    /// `\command` (or `\` + one symbol, e.g. `\\`, `\,`) — drawn in `accent`,
    /// matching what ```` ```latex ```` / ```` ```math ```` fences produce.
    Command,
    /// Grouping / script syntax `{ } ^ _ &` — recedes like a delimiter.
    Structure,
}

/// Split math source into runs of `(char_start, char_end, kind)`.
/// Adjacent runs of the same kind are merged.
fn tokenize_math(chars: &[char]) -> Vec<(usize, usize, MathTok)> {
    let mut out: Vec<(usize, usize, MathTok)> = Vec::new();
    let mut rest = chars;
    let mut pos = 0;
    while let Some((&c, tail)) = rest.split_first() {
        let (len, kind) = match c {
            // `\alpha` → whole word; `\\`, `\,`, `\{` → one symbol;
            // a trailing lone `\` → just itself.
            '\\' => {
                let letters = tail.iter().take_while(|c| c.is_ascii_alphabetic()).count();
                (1 + letters.max(1).min(tail.len()), MathTok::Command)
            }
            '{' | '}' | '^' | '_' | '&' => (1, MathTok::Structure),
            _ => (1, MathTok::Body),
        };
        // Always advance at least one char so the loop terminates even if the
        // length arithmetic above is ever wrong.
        let len = len.clamp(1, rest.len());
        let end = pos + len;
        match out.last_mut() {
            Some(last) if last.2 == kind => last.1 = end,
            _ => out.push((pos, end, kind)),
        }
        rest = &rest[len..];
        pos = end;
    }
    out
}

/// Number of leading container-prefix chars on a continuation line of a
/// multi-line math region: indentation, plus `>` markers on blockquote lines.
/// These belong to the enclosing block, not to the math.
fn container_prefix_len(chars: &[char], in_blockquote: bool) -> usize {
    chars
        .iter()
        .take_while(|&&c| c == ' ' || c == '\t' || (in_blockquote && c == '>'))
        .count()
}

/// Emit delimiter + content spans for one math region.
///
/// * Delimiters (`$` / `$$`) and structure (`{ } ^ _ &`) blend `math_color`
///   toward `muted`, like every other Markdown delimiter.
/// * Body is drawn in `math_color` with **no** `code_bg`, so `$x$` and
///   `` `x` `` stay visually distinct; `\commands` use `accent`.
/// * Display math that stands alone on its lines (`$$` opens the first line
///   and closes the last, nothing else on them) becomes a block panel on
///   `fenced_bg`, matching fenced code blocks.  Its lines are added to
///   `code_block_lines` so the `==highlight==` pass leaves them alone.
/// * Any other math region records its char ranges in `math_ranges` so the
///   `==highlight==` pass skips matches that overlap it (`$a==b$`).  On its
///   continuation lines the container prefix (`> `, indentation) is skipped.
fn emit_math(s: &mut BuildState, range: std::ops::Range<usize>, delim_len: usize) {
    let (start_line, start_char) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, end_char) = byte_to_line_char(&s.line_starts, s.text, range.end);

    let is_block = delim_len == 2
        && only_ws_before(s.text, s.line_starts[start_line], range.start)
        && only_ws_after(s.text, range.end);

    let bg = if is_block {
        Some(s.theme.fenced_bg)
    } else {
        s.in_heading_bg
    };
    let with_bg = |style: Style| match bg {
        Some(c) => style.bg(c),
        None => style,
    };
    let body_style = with_bg(Style::default().fg(s.theme.math_color));
    let command_style = with_bg(Style::default().fg(s.theme.accent));
    let delim_style = with_bg(Style::default().fg(blend_colors(
        s.theme.math_color,
        s.theme.muted,
        s.theme.delimiter_blend,
    )));

    for line in start_line..=end_line {
        let line_len = line_char_len(&s.line_starts, s.text, line);
        let chars: Vec<char> = s.text[s.line_starts[line]..]
            .chars()
            .take(line_len)
            .collect();
        let in_blockquote = s
            .map
            .get(&line)
            .is_some_and(|v| v.iter().any(|sp| sp.is_blockquote));

        let seg_start = if line == start_line {
            start_char
        } else if is_block {
            0
        } else {
            container_prefix_len(&chars, in_blockquote)
        };
        let seg_end = if line == end_line { end_char } else { line_len };

        // Delimiter boundaries on this line (empty when not on the edge line).
        let content_start = if line == start_line {
            (seg_start + delim_len).min(seg_end)
        } else {
            seg_start
        };
        let content_end = if line == end_line {
            seg_end.saturating_sub(delim_len).max(content_start)
        } else {
            seg_end
        };

        if is_block {
            s.code_block_lines.insert(line);
        } else {
            s.math_ranges
                .entry(line)
                .or_default()
                .push((seg_start, seg_end));
        }

        // The first span on a block line carries full_line_bg so the panel
        // fills the column.  (A blank line always ends the paragraph, so
        // display math never contains an empty interior line.)
        let mut full_bg = if is_block { bg } else { None };
        let mut push = |map: &mut DecorationMap, cs: usize, ce: usize, style: Style| {
            push_span(
                map,
                line,
                StyledSpan {
                    full_line_bg: full_bg.take(),
                    ..make_span(cs, ce, style)
                },
            );
        };

        if content_start > seg_start {
            push(&mut s.map, seg_start, content_start, delim_style);
        }
        for (cs, ce, kind) in tokenize_math(&chars[content_start..content_end]) {
            let style = match kind {
                MathTok::Body => body_style,
                MathTok::Command => command_style,
                MathTok::Structure => delim_style,
            };
            push(&mut s.map, content_start + cs, content_start + ce, style);
        }
        if seg_end > content_end {
            push(&mut s.map, content_end, seg_end, delim_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Style;

    use crate::config::{Theme, blend_colors};
    use crate::decoration::build_decoration_map;
    use crate::decoration::types::{DecorationMap, StyledSpan};

    fn build(text: &str) -> (DecorationMap, Theme) {
        let theme = Theme::default_theme();
        let map = build_decoration_map(text, &theme, true, None).0;
        (map, theme)
    }

    fn span_at(map: &DecorationMap, line: usize, cs: usize, ce: usize) -> Option<StyledSpan> {
        map.get(&line)?
            .iter()
            .find(|s| s.char_start == cs && s.char_end == ce)
            .cloned()
    }

    fn delim_fg(theme: &Theme) -> ratatui::style::Color {
        blend_colors(theme.math_color, theme.muted, theme.delimiter_blend)
    }

    #[test]
    fn inline_math_content_uses_math_color_without_code_bg() {
        // "a $x+2$ b": `$` at 2, content 3..6, `$` at 6.
        let (map, theme) = build("a $x+2$ b");
        let content = span_at(&map, 0, 3, 6).expect("content span 3..6");
        assert_eq!(content.style, Style::default().fg(theme.math_color));
        assert_eq!(content.full_line_bg, None);
        let open = span_at(&map, 0, 2, 3).expect("opening $ span");
        let close = span_at(&map, 0, 6, 7).expect("closing $ span");
        assert_eq!(open.style.fg, Some(delim_fg(&theme)));
        assert_eq!(close.style.fg, Some(delim_fg(&theme)));
        assert_eq!(open.style.bg, None);
    }

    #[test]
    fn inline_math_and_inline_code_are_distinguishable() {
        let (map, theme) = build("`x` $x$");
        let code = span_at(&map, 0, 1, 2).expect("code content");
        let math = span_at(&map, 0, 5, 6).expect("math content");
        assert_eq!(code.style.bg, Some(theme.code_bg));
        assert_eq!(math.style.bg, None);
        assert_ne!(code.style, math.style);
    }

    #[test]
    fn display_math_inline_on_line_uses_two_char_delimiters() {
        // "see $$x$$ here": `$$` 4..6, content 6..7, `$$` 7..9.
        let (map, theme) = build("see $$x$$ here");
        assert!(span_at(&map, 0, 4, 6).is_some(), "opening $$");
        let content = span_at(&map, 0, 6, 7).expect("content");
        assert_eq!(content.style.fg, Some(theme.math_color));
        assert_eq!(
            content.full_line_bg, None,
            "mid-paragraph $$ is not a panel"
        );
        assert!(span_at(&map, 0, 7, 9).is_some(), "closing $$");
    }

    #[test]
    fn display_math_block_gets_fenced_panel_on_every_line() {
        let (map, theme) = build("$$\n\\frac{a}{b}\n$$\n");
        let open = span_at(&map, 0, 0, 2).expect("opening $$ line");
        assert_eq!(open.full_line_bg, Some(theme.fenced_bg));
        assert_eq!(open.style.bg, Some(theme.fenced_bg));
        // `\frac` 0..5 is the first span on the body line → carries the panel.
        let cmd = span_at(&map, 1, 0, 5).expect("\\frac command");
        assert_eq!(cmd.style.fg, Some(theme.accent));
        assert_eq!(cmd.style.bg, Some(theme.fenced_bg));
        assert_eq!(cmd.full_line_bg, Some(theme.fenced_bg));
        let a = span_at(&map, 1, 6, 7).expect("body `a`");
        assert_eq!(a.style.fg, Some(theme.math_color));
        assert_eq!(
            a.full_line_bg, None,
            "only the first span carries full_line_bg"
        );
        let close = span_at(&map, 2, 0, 2).expect("closing $$ line");
        assert_eq!(close.full_line_bg, Some(theme.fenced_bg));
        assert_eq!(close.style.fg, Some(delim_fg(&theme)));
    }

    #[test]
    fn dollar_amounts_are_not_math() {
        let (map, theme) = build("costs $5 and $10 total");
        let any_math = map
            .get(&0)
            .is_some_and(|v| v.iter().any(|s| s.style.fg == Some(theme.math_color)));
        assert!(!any_math, "currency must not be parsed as math");
    }

    #[test]
    fn highlight_pass_skips_equals_inside_math() {
        let (map, theme) = build("$a==b==c$");
        let any_hl = map
            .get(&0)
            .is_some_and(|v| v.iter().any(|s| s.style.bg == Some(theme.highlight_bg)));
        assert!(!any_hl, "== inside math must not become a highlight");
    }

    #[test]
    fn highlight_outside_math_still_works() {
        let (map, theme) = build("$x$ and ==hi==");
        let any_hl = map
            .get(&0)
            .is_some_and(|v| v.iter().any(|s| s.style.bg == Some(theme.highlight_bg)));
        assert!(any_hl, "==hi== outside math must still highlight");
    }

    #[test]
    fn math_in_heading_keeps_heading_bg() {
        let (map, theme) = build("# H $e$");
        let content = span_at(&map, 0, 5, 6).expect("math content in heading");
        assert_eq!(content.style.fg, Some(theme.math_color));
        assert_eq!(content.style.bg, Some(theme.heading_bg));
    }

    #[test]
    fn math_inside_bold_is_not_overwritten() {
        let (map, theme) = build("**$x$**");
        let content = span_at(&map, 0, 3, 4).expect("math content inside bold");
        assert_eq!(content.style.fg, Some(theme.math_color));
    }

    #[test]
    fn display_math_is_panel_only_when_alone_on_its_lines() {
        // Text before → not a panel (kills only_ws_before → true).
        let (map, _) = build("see $$x$$");
        assert_eq!(span_at(&map, 0, 4, 6).unwrap().full_line_bg, None);
        assert_eq!(span_at(&map, 0, 6, 7).unwrap().style.bg, None);
        // Text after → not a panel (kills only_ws_after → true / `!=` → `==`).
        let (map, _) = build("$$x$$ here");
        assert_eq!(span_at(&map, 0, 0, 2).unwrap().full_line_bg, None);
        assert_eq!(span_at(&map, 0, 2, 3).unwrap().style.bg, None);
        // Alone on the line → panel.
        let (map, theme) = build("$$x$$");
        assert_eq!(
            span_at(&map, 0, 0, 2).unwrap().full_line_bg,
            Some(theme.fenced_bg)
        );
    }

    #[test]
    fn inline_math_alone_on_line_is_not_a_panel() {
        // Kills `delim_len == 2 && …` → `||`: single-$ math is never a panel.
        let (map, _) = build("$x$");
        let content = span_at(&map, 0, 1, 2).expect("content");
        assert_eq!(content.full_line_bg, None);
        assert_eq!(content.style.bg, None);
    }

    #[test]
    fn multi_line_math_emits_no_zero_width_spans() {
        let (map, _) = build("$$\nx\n$$\n");
        for (line, spans) in map.iter() {
            for s in spans {
                assert!(
                    s.char_start < s.char_end,
                    "zero-width span on line {line}: {s:?}"
                );
            }
        }
    }

    #[test]
    fn highlight_adjacent_to_math_is_not_skipped() {
        // Ranges are half-open: a match that merely touches math still highlights.
        for text in ["$x$==hi==", "==hi==$x$"] {
            let (map, theme) = build(text);
            let any_hl = map
                .get(&0)
                .is_some_and(|v| v.iter().any(|s| s.style.bg == Some(theme.highlight_bg)));
            assert!(any_hl, "{text:?}: highlight touching math must still apply");
        }
    }

    // ---- tokenizer ----

    use super::{MathTok, container_prefix_len, tokenize_math};

    fn tok(src: &str) -> Vec<(usize, usize, MathTok)> {
        tokenize_math(&src.chars().collect::<Vec<_>>())
    }

    #[test]
    fn tokenize_commands_structure_and_body() {
        use MathTok::*;
        assert_eq!(
            tok("\\frac{a}{b}+x^2"),
            vec![
                (0, 5, Command),
                (5, 6, Structure),
                (6, 7, Body),
                (7, 9, Structure),
                (9, 10, Body),
                (10, 11, Structure),
                (11, 13, Body),
                (13, 14, Structure),
                (14, 15, Body),
            ]
        );
    }

    #[test]
    fn tokenize_symbol_commands_and_trailing_backslash() {
        use MathTok::*;
        // `\\` and `\,` are one-symbol commands; adjacent commands merge.
        assert_eq!(
            tok("a\\\\b"),
            vec![(0, 1, Body), (1, 3, Command), (3, 4, Body)]
        );
        assert_eq!(tok("\\,\\alpha"), vec![(0, 8, Command)]);
        assert_eq!(tok("x\\"), vec![(0, 1, Body), (1, 2, Command)]);
        // Non-ASCII letters are not part of a command name.
        assert_eq!(tok("\\aé"), vec![(0, 2, Command), (2, 3, Body)]);
        assert_eq!(tok(""), vec![]);
    }

    #[test]
    fn tokenize_ampersand_is_structure() {
        assert_eq!(tok("&"), vec![(0, 1, MathTok::Structure)]);
    }

    #[test]
    fn container_prefix_skips_quote_markers_only_in_blockquotes() {
        let c: Vec<char> = " > > x".chars().collect();
        assert_eq!(container_prefix_len(&c, true), 5);
        assert_eq!(container_prefix_len(&c, false), 1);
        let t: Vec<char> = "\tx".chars().collect();
        assert_eq!(container_prefix_len(&t, false), 1);
    }

    // ---- integration ----

    #[test]
    fn command_inside_inline_math_uses_accent() {
        let (map, theme) = build("$\\alpha$");
        let cmd = span_at(&map, 0, 1, 7).expect("\\\\alpha command");
        assert_eq!(cmd.style.fg, Some(theme.accent));
        let close = span_at(&map, 0, 7, 8).expect("closing $");
        assert_eq!(close.style.fg, Some(delim_fg(&theme)));
    }

    #[test]
    fn structure_chars_recede_like_delimiters() {
        let (map, theme) = build("$x^2$");
        let caret = span_at(&map, 0, 2, 3).expect("^");
        assert_eq!(caret.style.fg, Some(delim_fg(&theme)));
    }

    #[test]
    fn display_math_in_blockquote_leaves_quote_prefix_alone() {
        // `> $$` / `> x` / `> $$`: the `> ` on lines 1–2 is the blockquote's.
        let (map, theme) = build("> $$\n> x\n> $$\n");
        for line in 1..=2 {
            let spans = map.get(&line).expect("line has spans");
            assert!(
                spans.iter().all(|s| s.is_blockquote
                    || s.char_start >= 2
                    || (s.style.fg != Some(theme.math_color)
                        && s.style.fg != Some(delim_fg(&theme)))),
                "line {line}: `> ` prefix must not take math colors: {spans:?}"
            );
        }
        let x = span_at(&map, 1, 2, 3).expect("`x` body");
        assert_eq!(x.style.fg, Some(theme.math_color));
        let close = span_at(&map, 2, 2, 4).expect("closing $$");
        assert_eq!(close.style.fg, Some(delim_fg(&theme)));
    }

    #[test]
    fn multi_line_math_in_list_skips_indent_only() {
        // In a list item, continuation indentation is skipped but nothing else.
        let (map, theme) = build("- $$\n  x\n  $$\n");
        let x = span_at(&map, 1, 2, 3).expect("`x` body after indent");
        assert_eq!(x.style.fg, Some(theme.math_color));
        assert!(
            span_at(&map, 1, 0, 2).is_none(),
            "indent must not be painted"
        );
    }
}
