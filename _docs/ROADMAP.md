# `yame` Roadmap & Next Steps

_Last updated 2026-05-25._
_Baseline: 135 tests green, clippy clean, Phases 0–11 complete. Module split done (#74)._

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

## Current State (v0.1 — Phase 12 pending)

All planned v1 phases implemented except README (Phase 12, #13). Module split
complete: `decoration.rs` → `decoration/{mod,spans,words}.rs`; `renderer.rs` →
`renderer/{mod,status,utils}.rs`; `main.rs` → `main.rs + commands.rs + input.rs`.
135 tests passing.

---

## Remaining v1 Work

### #13 · README & Distribution (Phase 12)

Full spec in PLAN.md § Phase 12:
- Install: `cargo install --path .`
- Shell wrapper function (fd/fzf/find fallback)
- Config reference: path, full palette defaults, override table, heading overrides
- Keybinding reference table
- Nerd Fonts note
- `Cargo.toml` publishing metadata (mostly already present)

---

## v1.5 — Sprint Plan

### Sprint 2: Spec debt (#39, #40, #59)

**#39 · Blockquote continuation indent**

DESIGN.md: _"On soft-wrapped lines, indent continuation text to align with text start
after `>` — do not wrap to column zero."_ The `is_blockquote: bool` flag already exists
and propagates from decoration → renderer. The render loop has a `wrap_idx` variable.

```rust
// In MarkdownView::render, inside the wrap loop:
let is_continuation = wrap_idx > 0 && row_spans.iter().any(|s| s.is_blockquote);
let indent: u16 = if is_continuation { 2 } else { 0 };
let mut x = area.x + GUTTER + indent;
let effective_width = content_width.saturating_sub(indent as usize);
```

Also update `apply_selection_overlay` to apply the same indent for blockquote
continuation rows.

Test: multi-line blockquote where wrapped line should indent 2 columns.

**#40 · Tab character expansion on load**

In `load_file` (or as a post-process step), expand `\t` → 4 spaces before passing
to `TextArea::new(lines)`. Save writes back the expanded form (intentionally lossy —
tabs in Markdown are almost always indentation).

Add a configurable tab width via `[layout] tab_width = 4` (default 4). Store in
`LayoutConfig`.

Test: file containing `\t` loads with spaces; word count unaffected; wrapping correct.

**#59 · Soft-wrap list items with continuation indent**

Similar to #39 but for list items. Continuation lines should align to text start
(after bullet + space), not column zero.

### Sprint 3: Wide character correctness (#41, #71)

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

### Sprint 4 (formerly): User-facing features — DONE

- ~~#42 · Ctrl+R config reload~~ — done
- ~~#43 · Smart pair wrapping~~ — done

### Sprint 5: Syntax highlighting (#44, #45)

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

## Open Issues (non-sprint)

| Issue | Title | Notes |
|---|---|---|
| #50 | Fix nested bold+italic rendering (***) | v1-bug |
| #54 | Replace Powerline glyph with universal fallback | v1-polish |
| #56 | Decouple scroll from cursor | v1.5 |
| #76 | Rework status message display | polish |
| #77 | In-app settings modal | v2 |
| #89 | Integration test planning | testing |
| #91 | Heading `#` delimiters not bold to match heading style | v1-polish |

---

## Issue Index

| Issue | Title | Sprint | Status |
|---|---|---|---|
| #13 | Phase 12: README & Distribution | v1 | open |
| #39 | v1.5: blockquote continuation indent | 1.5-S2 | open |
| #40 | v1.5: tab character expansion on load | 1.5-S2 | open |
| #41 | v1.5: CJK / wide character support | 1.5-S3 | open |
| #44 | v1.5: syntect fenced code highlighting | 1.5-S5 | open |
| #45 | v1.5: background decoration thread | 1.5-S5 | open |
| #46 | v2: Ctrl+F search with regex | v2 | open |
| #47 | v2: line numbers | v2 | open |
| #50 | Fix nested bold+italic rendering (***) | v1-bug | open |
| #54 | Replace Powerline glyph with universal fallback | v1-polish | open |
| #56 | Decouple scroll from cursor | v1.5 | open |
| #59 | Soft-wrap list items with continuation indent | 1.5-S2 | open |
| #71 | Wide char (CJK) scroll redraw artifact | bug | open |
| #74 | Split decoration.rs/renderer.rs/main.rs into submodules | refactor | **done** |
| #76 | Rework status message display | polish | open |
| #77 | In-app settings modal | v2 | open |
| #89 | Integration test planning | testing | open |
| #91 | Heading `#` delimiters not bold to match heading style | v1-polish | open |
