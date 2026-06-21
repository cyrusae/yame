//! GFM pipe-table formatter.
//!
//! `handle_format_table` (bound to `Alt+T`) detects the table under the cursor,
//! reflowing it to uniform column widths while preserving alignment markers.
//!
//! ## Design notes
//!
//! Column widths are measured in *source characters*, not rendered display
//! columns.  This matches the behaviour of tools like Prettier: a cell
//! containing `**bold**` counts 8 source characters even though it renders 4.
//! The alternative — a full render pass to count visual widths — would be
//! circular (decoration depends on the source, which depends on decoration).
//!
//! Every column is guaranteed a minimum content width of 3 so that the
//! separator `---` / `:--` / `--:` / `:-:` always has room for its markers.

use std::time::Duration;

use tui_textarea::CursorMove;

use crate::app::App;

// ---------------------------------------------------------------------------
// Column alignment
// ---------------------------------------------------------------------------

/// Column alignment parsed from the GFM separator row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColAlign {
    /// No colon: `---`
    None,
    /// Leading colon only: `:---`
    Left,
    /// Trailing colon only: `---:`
    Right,
    /// Both colons: `:---:`
    Center,
}

// ---------------------------------------------------------------------------
// Table-bounds detection
// ---------------------------------------------------------------------------

/// Return `(start_row, end_row)` of the table that contains `cursor_row`, or
/// `None` if the cursor is not on a table line.
///
/// A table line is any non-empty line whose trimmed form starts with `|`.
/// A valid table requires at least two rows (header + separator).
pub fn find_table_bounds(lines: &[String], cursor_row: usize) -> Option<(usize, usize)> {
    if cursor_row >= lines.len() {
        return None;
    }
    if !is_table_line(&lines[cursor_row]) {
        return None;
    }

    // Extend upward.
    let mut start = cursor_row;
    while start > 0 && is_table_line(&lines[start - 1]) {
        start -= 1;
    }

    // Extend downward.
    let mut end = cursor_row;
    while end + 1 < lines.len() && is_table_line(&lines[end + 1]) {
        end += 1;
    }

    // Need at least header + separator rows.
    if end <= start {
        return None;
    }

    Some((start, end))
}

fn is_table_line(line: &str) -> bool {
    let t = line.trim();
    !t.is_empty() && t.starts_with('|')
}

// ---------------------------------------------------------------------------
// Row parsing
// ---------------------------------------------------------------------------

/// Split a table source line into trimmed cell strings.
///
/// Leading and trailing `|` are stripped before splitting, so
/// `"| A | B |"` → `["A", "B"]`.
pub fn parse_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let inner = t.strip_prefix('|').unwrap_or(t);
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// Return `true` if every cell in `cells` looks like a separator cell
/// (`^:?-+:?$`) and there is at least one cell.
fn is_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|c| {
            let t = c.trim();
            !t.is_empty() && t.contains('-') && t.chars().all(|ch| matches!(ch, '-' | ':'))
        })
}

/// Parse the alignment marker from a separator cell string.
fn parse_alignment(cell: &str) -> ColAlign {
    let t = cell.trim();
    let left = t.starts_with(':');
    let right = t.ends_with(':');
    match (left, right) {
        (true, true) => ColAlign::Center,
        (true, false) => ColAlign::Left,
        (false, true) => ColAlign::Right,
        (false, false) => ColAlign::None,
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Build the separator cell string for `align` that is exactly `width`
/// characters wide (not counting the surrounding spaces the row emitter adds).
///
/// `width` must be ≥ 3 (the minimum enforced by `format_table`).
fn format_sep_cell(align: ColAlign, width: usize) -> String {
    let w = width.max(3);
    match align {
        ColAlign::None => "-".repeat(w),
        ColAlign::Left => format!(":{}", "-".repeat(w - 1)),
        ColAlign::Right => format!("{}:", "-".repeat(w - 1)),
        ColAlign::Center => {
            // `:` + (w-2) dashes + `:`
            let dashes = w.saturating_sub(2);
            format!(":{}:", "-".repeat(dashes))
        }
    }
}

/// Left-pad `cell` with trailing spaces so its char-count equals `width`.
fn pad_cell(cell: &str, width: usize) -> String {
    let len = cell.chars().count();
    if len >= width {
        cell.to_string()
    } else {
        format!("{}{}", cell, " ".repeat(width - len))
    }
}

/// Rebuild a single table row as `| cell1 | cell2 | … |`.
///
/// `is_sep` — if `true`, cells are formatted as separator dashes+colons;
/// otherwise as padded content.
fn format_row(cells: &[String], widths: &[usize], alignments: &[ColAlign], is_sep: bool) -> String {
    let mut out = String::from("|");
    for (ci, &w) in widths.iter().enumerate() {
        let cell = cells.get(ci).map(String::as_str).unwrap_or("");
        let align = alignments.get(ci).copied().unwrap_or(ColAlign::None);
        out.push(' ');
        if is_sep {
            out.push_str(&format_sep_cell(align, w));
        } else {
            out.push_str(&pad_cell(cell, w));
        }
        out.push(' ');
        out.push('|');
    }
    out
}

// ---------------------------------------------------------------------------
// Public: format a slice of table lines
// ---------------------------------------------------------------------------

/// Reformat `lines` (which must be the rows of a single GFM table) to have
/// uniform column widths.
///
/// - Column widths are the max source-char width across all non-separator rows
///   (minimum 3 to accommodate the separator markers).
/// - The separator row is rebuilt to preserve alignment colons (`:---`, `---:`,
///   `:---:`, or `---`) at the computed column width.
/// - All other rows have cells left-padded with spaces.
///
/// Returns the reformatted lines (same count as input).
pub fn format_table(lines: &[String]) -> Vec<String> {
    let rows: Vec<Vec<String>> = lines.iter().map(|l| parse_row(l)).collect();

    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return lines.to_vec();
    }

    // The separator row is conventionally the second row (index 1).
    let sep_row_idx = 1;
    let has_sep = rows.len() > sep_row_idx && is_separator_row(&rows[sep_row_idx]);

    let alignments: Vec<ColAlign> = if has_sep {
        let mut a: Vec<ColAlign> = rows[sep_row_idx]
            .iter()
            .map(|c| parse_alignment(c))
            .collect();
        // Pad to ncols if the separator has fewer cells than the header.
        a.resize(ncols, ColAlign::None);
        a
    } else {
        vec![ColAlign::None; ncols]
    };

    // Compute column widths from non-separator rows; minimum 3.
    let mut widths = vec![3usize; ncols];
    for (ri, row) in rows.iter().enumerate() {
        if ri == sep_row_idx && has_sep {
            continue;
        }
        for (ci, cell) in row.iter().enumerate() {
            if ci < ncols {
                widths[ci] = widths[ci].max(cell.chars().count());
            }
        }
    }

    // Rebuild each row.
    rows.iter()
        .enumerate()
        .map(|(ri, row)| {
            let is_sep = ri == sep_row_idx && has_sep;
            format_row(row, &widths, &alignments, is_sep)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public: compact (strip padding)
// ---------------------------------------------------------------------------

/// Rebuild each table row with no cross-row alignment: content cells keep only
/// their own content (no trailing spaces), separator cells use the minimum
/// width of 3 with alignment markers preserved.
///
/// The result is a valid GFM table where rows may have different total widths.
/// This is the inverse of `format_table`.
pub fn compact_table(lines: &[String]) -> Vec<String> {
    let rows: Vec<Vec<String>> = lines.iter().map(|l| parse_row(l)).collect();
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return lines.to_vec();
    }

    let sep_row_idx = 1;
    let has_sep = rows.len() > sep_row_idx && is_separator_row(&rows[sep_row_idx]);
    let alignments: Vec<ColAlign> = if has_sep {
        let mut a: Vec<ColAlign> = rows[sep_row_idx]
            .iter()
            .map(|c| parse_alignment(c))
            .collect();
        a.resize(ncols, ColAlign::None);
        a
    } else {
        vec![ColAlign::None; ncols]
    };

    rows.iter()
        .enumerate()
        .map(|(ri, row)| {
            let is_sep = ri == sep_row_idx && has_sep;
            // Per-cell widths: separator gets minimum 3; content gets own length.
            let widths: Vec<usize> = (0..ncols)
                .map(|ci| {
                    if is_sep {
                        3
                    } else {
                        row.get(ci).map(|c| c.chars().count()).unwrap_or(0).max(1)
                    }
                })
                .collect();
            format_row(row, &widths, &alignments, is_sep)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public: editor command
// ---------------------------------------------------------------------------

/// Replace the table under the cursor with a uniformly-padded version, or
/// strip it back to compact form if it is already uniform.
///
/// - First press (non-uniform table): pad every column to its maximum width.
/// - Second press (already-uniform table): strip padding back to compact form.
///
/// Bound to `Alt+T` in normal editing mode.  Does nothing (shows a status
/// hint) if the cursor is not inside a GFM pipe table.
#[mutants::skip] // Requires a live App+TextArea; covered by manual testing only.
pub fn handle_format_table(app: &mut App) {
    let (cursor_row, _) = app.textarea.cursor();
    let lines: Vec<String> = app.textarea.lines().iter().map(|s| s.to_string()).collect();

    let Some((start, end)) = find_table_bounds(&lines, cursor_row) else {
        app.status
            .set_dismissible("Alt+T: cursor is not inside a table.".to_string());
        return;
    };

    let table_lines = lines[start..=end].to_vec();
    let formatted = format_table(&table_lines);
    let already_uniform = formatted == table_lines;

    let result = if already_uniform {
        compact_table(&table_lines)
    } else {
        formatted
    };

    // Replace each line in place (top-to-bottom; line indices stay stable
    // because we never add or remove rows, only rewrite their content).
    for (i, new_line) in result.iter().enumerate() {
        let row = start + i;
        app.textarea.move_cursor(CursorMove::Jump(row as u16, 0));
        app.textarea.delete_line_by_end();
        app.textarea.insert_str(new_line);
    }

    // Return cursor to the original row, column 0.
    app.textarea
        .move_cursor(CursorMove::Jump(cursor_row as u16, 0));

    app.force_redecorate = true;
    app.mark_keystroke();
    app.recompute_dirty();

    let msg = if already_uniform {
        "Table compacted."
    } else {
        "Table formatted."
    };
    app.status
        .set_timed(msg.to_string(), Duration::from_millis(1500));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // ── find_table_bounds ────────────────────────────────────────────────────

    #[test]
    fn bounds_cursor_on_header_row() {
        let lines = s(&["| A | B |", "| - | - |", "| 1 | 2 |"]);
        assert_eq!(find_table_bounds(&lines, 0), Some((0, 2)));
    }

    #[test]
    fn bounds_cursor_on_body_row() {
        let lines = s(&["| A | B |", "| - | - |", "| 1 | 2 |"]);
        assert_eq!(find_table_bounds(&lines, 2), Some((0, 2)));
    }

    #[test]
    fn bounds_table_surrounded_by_other_content() {
        let lines = s(&[
            "Some text",
            "| A | B |",
            "| - | - |",
            "| 1 | 2 |",
            "",
            "More text",
        ]);
        assert_eq!(find_table_bounds(&lines, 2), Some((1, 3)));
    }

    #[test]
    fn bounds_cursor_not_in_table_returns_none() {
        let lines = s(&["Some text", "| A | B |", "| - | - |"]);
        assert_eq!(find_table_bounds(&lines, 0), None);
    }

    #[test]
    fn bounds_single_row_table_returns_none() {
        // A single line is not a valid table (needs at least header + separator).
        let lines = s(&["| A | B |"]);
        assert_eq!(find_table_bounds(&lines, 0), None);
    }

    #[test]
    fn bounds_empty_line_between_tables_splits_them() {
        let lines = s(&["| A |", "| - |", "", "| B |", "| - |"]);
        // Cursor on first table.
        assert_eq!(find_table_bounds(&lines, 0), Some((0, 1)));
        // Cursor on second table.
        assert_eq!(find_table_bounds(&lines, 3), Some((3, 4)));
    }

    // ── parse_row ────────────────────────────────────────────────────────────

    #[test]
    fn parse_row_strips_outer_pipes_and_trims() {
        let row = parse_row("| Hello | World |");
        assert_eq!(row, vec!["Hello", "World"]);
    }

    #[test]
    fn parse_row_separator_line() {
        let row = parse_row("|:---|:---:|---:|");
        assert_eq!(row, vec![":---", ":---:", "---:"]);
    }

    #[test]
    fn parse_row_no_trailing_pipe() {
        let row = parse_row("| A | B");
        assert_eq!(row, vec!["A", "B"]);
    }

    // ── is_separator_row ─────────────────────────────────────────────────────

    #[test]
    fn separator_detected_plain_dashes() {
        assert!(is_separator_row(&s(&["---", "---"])));
    }

    #[test]
    fn separator_detected_with_colons() {
        assert!(is_separator_row(&s(&[":---", ":---:", "---:"])));
    }

    #[test]
    fn separator_not_detected_for_content_row() {
        assert!(!is_separator_row(&s(&["Hello", "World"])));
    }

    // ── parse_alignment ──────────────────────────────────────────────────────

    #[test]
    fn alignment_none() {
        assert_eq!(parse_alignment("---"), ColAlign::None);
    }

    #[test]
    fn alignment_left() {
        assert_eq!(parse_alignment(":---"), ColAlign::Left);
    }

    #[test]
    fn alignment_right() {
        assert_eq!(parse_alignment("---:"), ColAlign::Right);
    }

    #[test]
    fn alignment_center() {
        assert_eq!(parse_alignment(":---:"), ColAlign::Center);
    }

    // ── format_sep_cell ──────────────────────────────────────────────────────

    #[test]
    fn sep_cell_none_width_3() {
        assert_eq!(format_sep_cell(ColAlign::None, 3), "---");
    }

    #[test]
    fn sep_cell_left_width_5() {
        assert_eq!(format_sep_cell(ColAlign::Left, 5), ":----");
    }

    #[test]
    fn sep_cell_right_width_5() {
        assert_eq!(format_sep_cell(ColAlign::Right, 5), "----:");
    }

    #[test]
    fn sep_cell_center_width_5() {
        assert_eq!(format_sep_cell(ColAlign::Center, 5), ":---:");
    }

    #[test]
    fn sep_cell_center_width_3() {
        assert_eq!(format_sep_cell(ColAlign::Center, 3), ":-:");
    }

    // ── format_table ─────────────────────────────────────────────────────────

    #[test]
    fn format_table_pads_short_cells_to_max_width() {
        let lines = s(&["| A | Longer header |", "| - | ------------ |", "| x | y |"]);
        let result = format_table(&lines);
        // "A" / "x" are 1 char each but minimum column width is 3, so they
        // pad to 3.  "Longer header" / "y" → max 13 chars.
        assert_eq!(result[0], "| A   | Longer header |");
        assert_eq!(result[2], "| x   | y             |");
    }

    #[test]
    fn format_table_preserves_alignment_markers() {
        let lines = s(&[
            "| Left | Center | Right |",
            "|:-----|:------:|------:|",
            "| a    | b      | c     |",
        ]);
        let result = format_table(&lines);
        // Separator row must use `:` correctly.
        let sep = &result[1];
        assert!(sep.contains(":---"), "left marker: {sep}");
        assert!(
            sep.contains(":---:") || sep.contains("---:"),
            "other markers: {sep}"
        );
    }

    #[test]
    fn format_table_separator_width_matches_column_width() {
        let lines = s(&[
            "| Short | A very long header |",
            "| ----- | ------------------ |",
            "| x     | y                  |",
        ]);
        let result = format_table(&lines);
        let sep = &result[1];
        // Separator columns should be exactly as wide as header columns + surrounding spaces.
        assert_eq!(
            result[0].len(),
            result[1].len(),
            "all rows same total width"
        );
        assert_eq!(
            result[0].len(),
            result[2].len(),
            "all rows same total width"
        );
        // No width regression: separator must not be shorter than its column header.
        let sep_cells = parse_row(sep);
        assert!(
            sep_cells[1].chars().count() >= "A very long header".len(),
            "sep col width ≥ header width: {:?}",
            sep_cells
        );
    }

    #[test]
    fn format_table_minimum_column_width_is_3() {
        // Even single-char headers get a minimum of 3-char-wide separator.
        let lines = s(&["| A |", "| - |", "| x |"]);
        let result = format_table(&lines);
        let sep_cell = parse_row(&result[1]).into_iter().next().unwrap_or_default();
        assert!(
            sep_cell.chars().count() >= 3,
            "separator cell must be ≥ 3 chars wide: {:?}",
            sep_cell
        );
    }

    #[test]
    fn format_table_roundtrips_already_formatted_table() {
        // Calling format_table on an already-formatted table is idempotent.
        let lines = s(&[
            "| Left | Center | Right |",
            "| ---- | ------ | ----- |",
            "| a    | b      | c     |",
        ]);
        let once = format_table(&lines);
        let twice = format_table(&once);
        assert_eq!(once, twice, "format_table must be idempotent");
    }

    #[test]
    fn format_table_all_rows_same_total_width() {
        let lines = s(&[
            "| Short | A longer one |",
            "| ----- | ------------ |",
            "| x     | y            |",
            "| abc   | def          |",
        ]);
        let result = format_table(&lines);
        let widths: Vec<usize> = result.iter().map(|l| l.chars().count()).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "all rows must have equal total width: {:?}",
            widths
        );
    }

    #[test]
    fn format_table_inline_markup_counted_as_source_chars() {
        // **bold** counts 8 source chars, not 4 rendered chars.
        let lines = s(&[
            "| **bold** | plain |",
            "| -------- | ----- |",
            "| x        | y     |",
        ]);
        let result = format_table(&lines);
        let cells = parse_row(&result[0]);
        assert_eq!(
            cells[0], "**bold**",
            "bold cell must not be truncated: {:?}",
            cells
        );
    }

    // ---- T-21: is_separator_row — body rows must not be detected as separators ----

    #[test]
    fn separator_not_detected_for_row_with_letters_and_dash() {
        // Kills: line 103:46 `replace && with || in is_separator_row`
        // With `||`, any non-empty cell is a separator because `contains('-')` or
        // `contains(':')` is often true.  "a-b" contains '-' but is not a separator.
        assert!(
            !is_separator_row(&s(&["a-b"])),
            "cell 'a-b' must not be a separator"
        );
        assert!(
            !is_separator_row(&s(&["a:b"])),
            "cell 'a:b' must not be a separator"
        );
        assert!(
            !is_separator_row(&s(&["hello"])),
            "cell 'hello' must not be a separator"
        );
        // True separators still work.
        assert!(is_separator_row(&s(&["---"])), "'---' must be a separator");
        assert!(
            is_separator_row(&s(&[":--:"])),
            "':--:' must be a separator"
        );
    }

    // ── compact_table ────────────────────────────────────────────────────────

    #[test]
    fn compact_table_removes_trailing_padding() {
        // A uniformly-padded table should become compact (no trailing spaces in cells).
        let lines = s(&["| Name   | Age |", "| ------ | --- |", "| Alice  | 30  |"]);
        let result = compact_table(&lines);
        assert_eq!(result[0], "| Name | Age |");
        assert_eq!(result[2], "| Alice | 30 |");
    }

    #[test]
    fn compact_table_separator_uses_minimum_width() {
        let lines = s(&[
            "| Long header column |",
            "| ------------------ |",
            "| x                  |",
        ]);
        let result = compact_table(&lines);
        let sep_cell = parse_row(&result[1]).into_iter().next().unwrap_or_default();
        assert_eq!(sep_cell, "---", "compact sep cell must be minimum '---'");
    }

    #[test]
    fn compact_table_preserves_alignment_markers() {
        let lines = s(&[
            "| Left   | Center  | Right  |",
            "| :----- | :-----: | -----: |",
            "| a      | b       | c      |",
        ]);
        let result = compact_table(&lines);
        let sep_cells = parse_row(&result[1]);
        assert_eq!(parse_alignment(&sep_cells[0]), ColAlign::Left);
        assert_eq!(parse_alignment(&sep_cells[1]), ColAlign::Center);
        assert_eq!(parse_alignment(&sep_cells[2]), ColAlign::Right);
    }

    #[test]
    fn compact_table_is_idempotent() {
        let lines = s(&["| A | B |", "| - | - |", "| x | y |"]);
        let once = compact_table(&lines);
        let twice = compact_table(&once);
        assert_eq!(once, twice, "compact_table must be idempotent");
    }

    #[test]
    fn format_then_compact_returns_to_compact_form() {
        // Round-trip: compact → format → compact must restore the compact form.
        // Use sep cells that are already at min width (3) so the separator is
        // unchanged by the round-trip; only content padding is stripped.
        let compact = s(&["| Name | Age |", "| --- | --- |", "| Alice | 30 |"]);
        let formatted = format_table(&compact);
        assert_ne!(formatted, compact, "format must change a compact table");
        let back = compact_table(&formatted);
        assert_eq!(back, compact, "compact after format must restore original");
    }

    #[test]
    fn already_uniform_table_detected_correctly() {
        // format_table on a uniformly-padded table must be identity.
        let uniform = s(&["| Name  | Age |", "| ----- | --- |", "| Alice | 30  |"]);
        let again = format_table(&uniform);
        assert_eq!(
            again, uniform,
            "uniform table must be fixed point of format_table"
        );
    }

    // ---- T-22: format_table with single-row input (no separator row) ----

    #[test]
    fn format_table_single_row_does_not_panic() {
        // Kills: lines 197:30 (`>→>=`) and 197:44 (`&&→||`).
        // With `>→>=`: `rows.len() >= 1` → tries rows[sep_row_idx=1] on a 1-row input → OOB.
        // With `&&→||`: same OOB path triggered when len==1.
        let lines = s(&["| A | B |"]);
        let result = format_table(&lines);
        assert_eq!(
            result.len(),
            1,
            "single-row input must produce single-row output"
        );
        assert!(
            result[0].contains('|'),
            "output must still be a pipe-table row: {:?}",
            result
        );
    }

    #[test]
    fn compact_table_single_row_does_not_panic() {
        // Kills compact_table 252:30 (`>→>=`) and 252:44 (`&&→||`).
        // With `>→>=`: `1 >= 1` is true → rows[1] OOB panic.
        // With `&&→||`: `false || rows[1]` → same OOB panic.
        let lines = s(&["| A | B |"]);
        let result = compact_table(&lines);
        assert_eq!(result.len(), 1, "single-row compact must produce one row");
        assert!(result[0].contains('|'));
    }

    #[test]
    fn compact_table_two_rows_no_separator_preserved_as_content() {
        // Kills compact_table 252:44 (`&&→||`) without relying on OOB.
        // With `&&→||`: `rows.len() > 1 || ...` short-circuits to true, so row 1 is
        // treated as a separator → formatted as `| --- | --- |` instead of content.
        let lines = s(&["| A | B |", "| C | D |"]);
        let result = compact_table(&lines);
        assert_eq!(result.len(), 2);
        assert!(!result[1].contains("---"), "row 1 must not be separator-formatted: {:?}", result[1]);
        assert!(result[1].contains('C'), "row 1 content must survive: {:?}", result[1]);
    }
}
