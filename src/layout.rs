use ratatui::layout::Rect;

/// All layout rectangles computed from the terminal area.
#[derive(Debug, Clone, Copy)]
pub struct EditorLayout {
    /// Full terminal area.
    pub full: Rect,
    /// Centered editing column.
    pub column: Rect,
    /// Second-to-last row (cursor position / word count info).
    pub info_line: Rect,
    /// Last row (status bar / hint line).
    pub status_bar: Rect,
}

/// Default minimum column width when not configured.
pub const DEFAULT_MIN_COLS: u16 = 60;

/// Compute all layout rectangles for the given terminal area.
///
/// # Rules
/// - `column.width = max(area.width / 2, min_cols).min(area.width)`
/// - If `max_cols` is set the column is capped at `max(max_cols, min_cols)` so
///   the content never exceeds the configured prose width on wide terminals.
/// - Column is horizontally centered with equal margins.
/// - `status_bar` = last row, `info_line` = second-to-last row.
/// - `column.height` = total height - 2 (info_line + status_bar).
pub fn compute_layout(area: Rect, min_cols: u16, max_cols: Option<u16>) -> EditorLayout {
    let col_width = (area.width / 2).max(min_cols).min(area.width);
    // Apply the prose-width cap, but never let it push below min_cols.
    let col_width = max_cols
        .map(|m| col_width.min(m.max(min_cols)))
        .unwrap_or(col_width)
        .min(area.width);
    let margin = area.width.saturating_sub(col_width) / 2;
    let content_height = area.height.saturating_sub(2);

    let column = Rect {
        x: area.x + margin,
        y: area.y,
        width: col_width,
        height: content_height,
    };

    let info_line = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 1,
    };

    let status_bar = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    EditorLayout {
        full: area,
        column,
        info_line,
        status_bar,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_wide_terminal_has_margins() {
        let area = Rect::new(0, 0, 200, 50);
        let layout = compute_layout(area, 60, None);
        assert!(layout.column.x > 0, "expected left margin");
        assert_eq!(layout.column.width, 100); // 50% of 200
    }

    #[test]
    fn layout_narrow_terminal_fills_width() {
        let area = Rect::new(0, 0, 40, 50);
        let layout = compute_layout(area, 60, None); // min_cols > width
        assert_eq!(layout.column.x, 0);
        assert_eq!(layout.column.width, 40);
    }

    #[test]
    fn layout_always_has_status_and_info_rows() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = compute_layout(area, 40, None);
        assert_eq!(layout.status_bar.y, 23);
        assert_eq!(layout.info_line.y, 22);
    }

    #[test]
    fn layout_min_cols_respected_on_medium_terminal() {
        // 100-wide, min_cols=60: 100/2=50 < 60, so col_width=60
        let area = Rect::new(0, 0, 100, 30);
        let layout = compute_layout(area, 60, None);
        assert_eq!(layout.column.width, 60);
        // Column should be centered: margin = (100-60)/2 = 20
        assert_eq!(layout.column.x, 20);
    }

    #[test]
    fn layout_content_height_is_total_minus_two() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = compute_layout(area, 40, None);
        assert_eq!(layout.column.height, 22);
    }

    #[test]
    fn layout_tiny_terminal_does_not_panic() {
        // 2x2 terminal — must not panic or produce nonsensical rects
        let area = Rect::new(0, 0, 2, 2);
        let layout = compute_layout(area, 60, None);
        assert_eq!(layout.column.width, 2); // clamped to area.width
        assert_eq!(layout.column.height, 0); // 2 - 2 = 0, saturating
    }

    #[test]
    fn layout_status_bar_full_width() {
        let area = Rect::new(0, 0, 80, 24);
        let layout = compute_layout(area, 40, None);
        assert_eq!(layout.status_bar.width, 80);
        assert_eq!(layout.info_line.width, 80);
    }

    // ── max_cols ─────────────────────────────────────────────────────────────

    #[test]
    fn max_cols_caps_column_on_wide_terminal() {
        // 200-wide terminal: without cap, col = 100 (50 %).
        // With max_cols = 80, col must be 80.
        let area = Rect::new(0, 0, 200, 50);
        let layout = compute_layout(area, 60, Some(80));
        assert_eq!(layout.column.width, 80, "max_cols=80 must cap column at 80");
    }

    #[test]
    fn max_cols_has_no_effect_when_column_already_narrower() {
        // 100-wide terminal: col = max(50, 60) = 60.  max_cols=80 > 60, so no effect.
        let area = Rect::new(0, 0, 100, 30);
        let layout = compute_layout(area, 60, Some(80));
        assert_eq!(
            layout.column.width, 60,
            "max_cols=80 must not shrink a 60-col column"
        );
    }

    #[test]
    fn max_cols_none_preserves_existing_behaviour() {
        // None → same result as before: col = 50 % of width.
        let area = Rect::new(0, 0, 200, 50);
        let layout = compute_layout(area, 60, None);
        assert_eq!(layout.column.width, 100);
    }

    #[test]
    fn max_cols_never_goes_below_min_cols() {
        // Conflicting config: max_cols=40 < min_cols=60.  min_cols wins.
        let area = Rect::new(0, 0, 200, 50);
        let layout = compute_layout(area, 60, Some(40));
        assert!(
            layout.column.width >= 60,
            "column must be >= min_cols even when max_cols < min_cols"
        );
    }

    #[test]
    fn max_cols_column_remains_centered() {
        // 200-wide terminal, max_cols=80: column should be centered with equal margins.
        let area = Rect::new(0, 0, 200, 50);
        let layout = compute_layout(area, 60, Some(80));
        assert_eq!(layout.column.width, 80);
        // margin = (200 - 80) / 2 = 60
        assert_eq!(layout.column.x, 60, "capped column must still be centered");
    }
}
