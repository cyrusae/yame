# `yame` Roadmap & Next Steps

_Last updated 2026-05-28._
_Baseline: 306 tests green, clippy clean. All v1 + v1.5 Sprint 2–4 work complete._

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

All v1 and v1.5 Sprint 2–4 phases complete. Open medium-priority work: CJK support
(#41), syntect highlighting (#44), Ctrl+F search (#46). Windows support (#125) is
implemented and ready to push.

---

## v1.5 Sprint 3: Wide character correctness (#41, #71)

**#41 · CJK / wide character support**

Add `unicode-width` to `[dependencies]`. Three coordinated changes:

_a) `wrap_line`:_ Replace char-count width accumulation with display-column accumulation
using `UnicodeWidthChar::width(c).unwrap_or(1)`.

_b) `MarkdownView::render`:_ Replace the per-char `buf[(x, y)].set_char(ch)` loop with
`buf.set_string(x, y, row_str, style)` which handles wide chars natively.

_c) Cursor and selection:_ Both `cursor_buf_pos` and `apply_selection_overlay` must count
display columns (not char counts) when computing x offsets.

_Effort estimate: 3–4 hours. Regression risk: medium — test with CJK fixture._

Add test fixture: `tests/fixtures/cjk_sample.md` with Japanese/Chinese/Korean text and
a test asserting word count is nonzero and decoration passes without panic.

**#71 · Wide char (CJK) scroll redraw artifact** — related bug, fix alongside #41.

---

## v1.5 Sprint 5: Syntax highlighting (#44, #45)

**#44 · Syntect fenced code block highlighting**

See DESIGN.md § "v1.5 Extension" for full design. Key points:

- Add `syntect` to `[dependencies]`
- Lazy grammar loading on first fenced block encountered (not startup)
- Cache keyed on block content hash — only re-highlight when block changes
- Limited grammar set: Rust, Python, JS/TS, Shell, JSON, TOML, YAML, SQL, Markdown
- Configurable via `[highlighting] grammars = ["rust", "python", ...]`
- Syntect spans merged with `fenced_bg` as background
- Background thread for grammar loading; fall back to fenced_bg-only until ready
- Language tag fallback: unrecognized tag → silent fenced_bg-only

Integration point: `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))` handler
in `decoration/mod.rs`.

**#45 · Move decoration pass to background thread**

_Do after #44 (syntect) since that's when it actually matters._

The seam is already correct: `build_decoration_map` is a pure free function.

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
| #41 | CJK / wide character support | medium | 1.5-S3 |
| #44 | Syntect fenced code highlighting | medium | 1.5-S5 |
| #45 | Background decoration thread | low | 1.5-S5 |
| #46 | Ctrl+F search with regex | medium | v2 |
| #47 | Line numbers | low | v2 |
| #71 | Wide char (CJK) scroll redraw artifact | low | 1.5-S3 |
| #77 | In-app settings modal | low | v2 |
| #89 | Integration test planning | low | — |
| #125 | Windows support | low | — (ready to push) |
