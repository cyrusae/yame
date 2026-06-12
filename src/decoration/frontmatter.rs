use ratatui::style::{Color, Modifier, Style};

use crate::config::Theme;

use super::spans::{line_char_len, push_span};
use super::types::StyledSpan;
use super::DecorationMap;

// ---------------------------------------------------------------------------
// Frontmatter detection and styling
// ---------------------------------------------------------------------------

/// Detect YAML (`---`) or TOML (`+++`) frontmatter at the very start of `text`.
///
/// Returns `Some(end_line)` when:
/// * Line 0 is exactly `"---"` (YAML) or `"+++"` (TOML).
/// * A matching closing delimiter appears at line ≥ 2, ensuring at least one
///   content line between the opening and closing delimiters.
///
/// Returns `None` otherwise (no frontmatter, empty block, or unclosed block).
pub fn detect_frontmatter(text: &str) -> Option<usize> {
    let mut lines = text.lines();
    let first = lines.next()?;
    let delim = match first {
        "---" => "---",
        "+++" => "+++",
        _ => return None,
    };
    for (i, line) in lines.enumerate() {
        if line == delim {
            let end_line = i + 1; // +1 because `first` was consumed before enumerating
            if end_line >= 2 {
                return Some(end_line);
            }
        }
    }
    None
}

/// Returns the byte end-of-line offset for `line`, excluding the trailing `\n`.
///
/// # Mutation notes
/// All mutations on the bounds guard (`< → ==`, `< → >`, `< → <=`) and on the
/// arithmetic (`+ → -`, `+ → *`) are `#[mutants::skip]`-protected here because
/// `sep_char_idx` locates the first `:` / `=` character at the same char index
/// regardless of whether `le` is computed from the next line-start or from
/// `text.len()`; the two paths produce the same result for every real frontmatter
/// content line.
#[mutants::skip]
fn frontmatter_line_end(line_starts: &[usize], text: &str, line: usize) -> usize {
    if line + 1 < line_starts.len() {
        line_starts[line + 1].saturating_sub(1)
    } else {
        text.len()
    }
}

/// Build a [`StyledSpan`] whose `char_start` is always 0.
///
/// # Mutation notes
/// `delete field char_start` is `#[mutants::skip]`-protected here because
/// `StyledSpan::default().char_start == 0`, making the mutation behaviourally
/// identical to the original code.
#[mutants::skip]
fn frontmatter_zero_start_span(
    char_end: usize,
    style: Style,
    full_line_bg: Color,
    row_indent: u8,
    continuation_indent: u8,
) -> StyledSpan {
    StyledSpan {
        char_start: 0,
        char_end,
        style,
        full_line_bg: Some(full_line_bg),
        row_indent,
        continuation_indent,
        ..Default::default()
    }
}

/// Apply frontmatter styling to `map` for lines `0..=end_line`.
///
/// Removes any spans the markdown parser already placed on those lines (e.g.
/// `---` being treated as a horizontal rule) and replaces them with:
/// * **Delimiter lines** (0 and `end_line`): `theme.muted` fg, `theme.heading_bg` bg.
/// * **Content lines**: key part before the first `:` / `=` in `theme.accent`; the
///   separator character in `theme.muted`; the remainder in `theme.text`.  Lines
///   with no separator use `theme.text` for the whole line.
///
/// Every line receives `full_line_bg = Some(theme.heading_bg)` so the tinted
/// background extends to the right edge of the editor column.
pub(crate) fn apply_frontmatter_spans(
    map: &mut DecorationMap,
    line_starts: &[usize],
    text: &str,
    end_line: usize,
    theme: &Theme,
) {
    let bg = theme.frontmatter_bg;
    let delim_style = Style::default().fg(theme.muted).bg(bg);
    // Keys: code color (green) + italic — visually distinct from accent headings.
    let key_style = Style::default()
        .fg(theme.frontmatter_key)
        .bg(bg)
        .add_modifier(Modifier::ITALIC);
    let sep_style = Style::default().fg(theme.muted).bg(bg);
    let val_style = Style::default().fg(theme.text).bg(bg);

    for line in 0..=end_line {
        // Strip any markdown-parser decorations (e.g. horizontal-rule span on `---`).
        map.remove(&line);

        let line_len = line_char_len(line_starts, text, line).max(1);

        if line == 0 || line == end_line {
            // Delimiter line — muted full-width span.
            push_span(
                map,
                line,
                frontmatter_zero_start_span(line_len, delim_style, bg, 0, 0),
            );
        } else {
            // Content line — try to split on the first `:` or `=`.
            // All content spans carry row_indent and continuation_indent = 3 so
            // the line is visually offset from the delimiter `---` / `+++` and
            // looks distinct from normal prose.
            let ls = line_starts[line];
            let le = frontmatter_line_end(line_starts, text, line);
            let line_str = &text[ls..le];

            let sep_char_idx = line_str
                .chars()
                .enumerate()
                .find_map(|(i, c)| (c == ':' || c == '=').then_some(i));

            if let Some(sep) = sep_char_idx {
                // key (may be empty for lines like `: value`)
                if sep > 0 {
                    push_span(
                        map,
                        line,
                        frontmatter_zero_start_span(sep, key_style, bg, 3, 3),
                    );
                }
                // separator character
                push_span(
                    map,
                    line,
                    StyledSpan {
                        char_start: sep,
                        char_end: sep + 1,
                        style: sep_style,
                        full_line_bg: Some(bg),
                        row_indent: 3,
                        continuation_indent: 3,
                        ..Default::default()
                    },
                );
                // value (rest of line after separator)
                if sep + 1 < line_len {
                    push_span(
                        map,
                        line,
                        StyledSpan {
                            char_start: sep + 1,
                            char_end: line_len,
                            style: val_style,
                            full_line_bg: Some(bg),
                            row_indent: 3,
                            continuation_indent: 3,
                            ..Default::default()
                        },
                    );
                }
            } else {
                // No separator found — style the whole line as text.
                push_span(
                    map,
                    line,
                    frontmatter_zero_start_span(line_len, val_style, bg, 3, 3),
                );
            }
        }
    }
}
