# `yame` Roadmap & Next Steps

_Last updated 2026-05-29._
_Baseline: 341 tests green, clippy clean. #41 (CJK) and #44 (syntect) complete._

---

## Decisions (locked)

| # | Decision |
|---|---|
| D1 | Italic startup warning: **implement as spec** |
| D2 | CJK wide characters: **fix in v1.5** (sooner is fine) |
| D3 | Tab characters: **expand to spaces on load** (simple path) |
| D4 | Smart pair wrapping: **v1.5, high priority** |
| D5 | Search: **regex** in v2 (`regex` crate) |
| D6 | Line numbers soft-wrap: **first visual row only** |
| D7 | Parent directory creation on save: **done** |

---

## Current State

v1, v1.5 Sprint 2–5 complete (#41 CJK, #44 syntect). Open medium-priority work:
Ctrl+F search (#46), plain-mode non-markdown files (#129). Windows support (#125)
is implemented and ready to push.

---

## ✅ v1.5 Sprint 3: Wide character correctness (#41, #71) — DONE

**#41 · CJK / wide character support** — complete.

`unicode-width` added. `display_cols_for_chars`, `chars_for_display_cols`, `cursor_vcol`
helpers added to `renderer/mod.rs`. Five sites fixed: `cursor_buf_pos`, `apply_selection_overlay`,
`char_col_at_visual`, `handle_visual_move` sticky column, `screen_to_doc` click column.
16 new tests. 322 → 341 tests green.

**#71 · Wide char (CJK) scroll redraw artifact** — resolved by #41 fix.

---

## ✅ v1.5 Sprint 5: Syntax highlighting (#44) — DONE

**#44 · Syntect fenced code block highlighting** — complete.

- `syntect` added with `default-themes`, `default-syntaxes`, `regex-fancy` features
- `src/highlighting.rs`: `HighlightCache` wraps `SyntaxSet` + `ThemeSet` with
  `RefCell<HashMap>` memoisation keyed on `(lang_lower, content_hash)`
- `App` gains `highlight_cache: Option<HighlightCache>`; initialised at startup
  from `[highlighting] enabled` + `syntect_theme` config fields
- `build_decoration_map` gains `highlight_cache: Option<&HighlightCache>` parameter;
  fenced block handler emits syntect fg spans (with `fenced_bg` background) on hit,
  falls back to `fenced_bg`-only on miss (unknown lang / disabled / no cache)
- Config: `[highlighting] enabled = true`, `syntect_theme = "base16-ocean.dark"`
  written to `DEFAULT_CONFIG_TEMPLATE`
- 14 unit tests in `highlighting.rs`, 5 integration tests in `integration.rs`
- **Note:** TOML is not in syntect's default bundled syntaxes; use YAML for config
  file examples instead. Consider `two-face` crate if TOML highlighting is needed.

**#45 · Move decoration pass to background thread**

_Do after #44 (syntect) since that's when it actually matters._

The seam is already correct: `build_decoration_map` is a pure free function taking
`Option<&HighlightCache>` (shared reference — `RefCell` inside makes it safe).

```rust
// In App, add:
pub decoration_tx: std::sync::mpsc::Sender<String>,
pub decoration_rx: std::sync::mpsc::Receiver<(DecorationMap, usize)>,
```

---

## v2 — Search, Line Numbers (#46, #47)

### #46 · `Ctrl+F` search with regex

Modal search bar at the bottom of the editor column (above the info line).

- `Ctrl+F` opens search bar; typing updates matches live
- `Enter` / `n` / `N` for next/prev match
- `Escape` closes, returns focus to editor
- Matches highlighted using a temporary decoration layer overlaying the base map
- Regex via `regex` crate
- Case-insensitive by default; `Ctrl+I` (or similar) toggles

State to add to `App`:
```rust
pub search: Option<SearchState>,
pub struct SearchState {
    pub query: String,
    pub matches: Vec<(usize, usize, usize)>, // (line, char_start, char_end)
    pub current: usize,
}
```

### #47 · Line numbers

`[ui] show_line_numbers = false` in config. Comment in `layout.rs`.

- When enabled: gutter width = `digits(total_lines) + 1` columns
- `compute_layout` adjusts `column.x` by gutter width
- Renderer draws number for first visual row of each logical line only (D6)
- Style: `theme.muted` fg, `theme.bg` bg

---

## Open Issues

| Issue | Title | Priority | Sprint |
|---|---|---|---|
| #45 | Background decoration thread | low | 1.5-S5 |
| #46 | Ctrl+F search with regex | medium | v2 |
| #47 | Line numbers | low | v2 |
| #77 | In-app settings modal | low | v2 |
| #89 | Integration test planning | low | — |
| #125 | Windows support | low | — (ready to push) |
| #129 | Plain-mode: syntect whole-file for non-markdown | medium | after #44 (unblocked) |
