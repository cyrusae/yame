# `yame` Roadmap & Next Steps

_Generated 2026-05-24. Decisions recorded same day._
_Baseline: 93 tests green, clippy clean, Phases 0–11 complete._

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
| D7 | Parent directory creation on save: **fix now** |

---

## Current State (v0.1 — Phase 12 pending)

All planned v1 phases are implemented except the README (Phase 12, #13). The three
critical bugs from the adversarial review have been fixed (POSIX newline, mouse
coordinate offset, dirty-flag on navigation). Four live-testing UI issues fixed (initial
decoration pass, gutter, todo muted-only, narrow info line). Scrollbar removed.

---

## Remaining v1 Work (#35 + #13)

### #13 · README & Distribution (Phase 12)

Full spec in PLAN.md § Phase 12:
- Install: `cargo install --path .`
- Shell wrapper function (fd/fzf/find fallback)
- Config reference: path, full palette defaults, override table, heading overrides
- Keybinding reference table
- Nerd Fonts note
- `Cargo.toml` publishing metadata (mostly already present)

### #35 · v1 polish: spec gaps + quick wins

All small items, one commit:

| Item | Detail |
|---|---|
| Italic startup warning | In `run()`, after `App::new()`: if `!italic_support`, `app.status.set_dismissible("⚠ Terminal does not support italics — using color fallback [any key to dismiss]")` |
| `delimiter_blend` config override | Add `delimiter_blend: Option<f32>` to `ThemeOverrides`; read in `Theme::from_config` with default `0.4` |
| `todo_done` theme token | Add `todo_done: Color` to `Theme`; default = `theme.muted`; used in todo-checked handler |
| `ui_text` theme token | Add `ui_text: Color` to `Theme`; default = `theme.text`; used in status bar text |
| Parent dir creation on save | `if let Some(parent) = app.file_path.parent() { fs::create_dir_all(parent)?; }` before `fs::write` |

**Tech debt cleanup (same commit):**
- Remove `current_item_line` / `let _ = current_item_line` from `decoration.rs` — was stored for TaskListMarker correlation but pulldown-cmark provides the marker's own range directly; it's dead code
- Remove `italic_support` field from `MarkdownView` — detected in `decoration.rs` not the renderer; field is stored but never read; remove or actually use it

---

## v1.5 — Sprint Plan

### Sprint 1: Quick wins & correctness (#36, #37, #38)

**#36 · Merge `count_words` into `build_decoration_map`**

Two full pulldown-cmark passes per debounce tick → one. Accumulate a `word_count: usize`
local during the decoration pass, return `(DecorationMap, usize)`. Update both call
sites (initial pass + debounce loop). Update tests to destructure.

**#37 · Fix O(N²) char counting and allocation hot-paths**

Two fixes in one issue:

_a) Allocation hot-paths:_ `split_into_spans` allocates `Vec<(usize, char)>` before the
empty-spans fast-path check — move it after. `wrap_line` allocates `char_indices` before
the `total_chars <= width` early-return — move it after. Together cuts thousands of
allocations per second during typing.

_b) O(N²) char counting:_ In `MarkdownView::render` and `apply_selection_overlay`, the
`char_start` for each visual row is computed by `line[..byte_off].chars().count()` —
O(line_length) per visual row. Fix: track `char_start` incrementally as wrapped chunks
are iterated (derive byte-offset deltas instead of rescanning from zero).

**#38 · Cache `arboard::Clipboard` in App**

`arboard::Clipboard::new()` opens an OS connection on every copy/paste. On Wayland this
can visibly stall. Fix: add `clipboard: Option<arboard::Clipboard>` to `App`; lazy-init
on first use; reuse for the session.

```rust
// App field:
pub clipboard: Option<arboard::Clipboard>,
```

In `clipboard.rs`, change `copy_to_clipboard(app, text)` / `paste_from_clipboard(app)`
to take `&mut App` and use `app.clipboard.get_or_insert_with(...)`.

### Sprint 2: Spec debt (#39, #40)

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

### Sprint 3: Wide character correctness (#41)

**#41 · CJK / wide character support**

Add `unicode-width` to `[dependencies]`. Three coordinated changes:

_a) `wrap_line`:_ Replace char-count width accumulation with display-column accumulation
using `UnicodeWidthChar::width(c).unwrap_or(1)`. The function signature stays the same;
the internal loop changes.

_b) `MarkdownView::render`:_ Replace the `buf[(x, y)].set_char(ch); x += 1` loop with
`buf.set_string(x, y, row_str, style)` which handles wide chars natively. Ratatui's
`Buffer::set_string` advances x by the actual display width of each character. The
per-span approach needs a helper that applies style spans to buffer positions accounting
for wide character widths.

_c) Cursor and selection:_ Both `cursor_buf_pos` calculation and `apply_selection_overlay`
must count display columns (not char counts) when computing x offsets.

_Effort estimate: 3–4 hours. Regression risk: medium — test with CJK fixture._

Add test fixture: `tests/fixtures/cjk_sample.md` with Japanese/Chinese/Korean text and
a test asserting word count is nonzero and decoration passes without panic.

### Sprint 4: User-facing features (#42, #43)

**#42 · `Ctrl+R` config reload**

The `// TODO(v1.5): Ctrl+R reloads config` comment is in the event loop in `main.rs`.

```rust
(KeyModifiers::CONTROL, KeyCode::Char('r')) => {
    let (new_config, new_warnings) = load_config();
    let mut warnings = new_warnings;
    app.theme = Theme::from_config(&new_config.palette, &new_config.theme, &new_config.headings, &mut warnings);
    app.config_warnings = warnings;
    app.status.set_timed("Config reloaded.", Duration::from_millis(1500));
    app.last_keystroke = Some(std::time::Instant::now()); // trigger decoration rebuild
}
```

Add `Ctrl+R` to status bar hint string. Test: no-op when config file absent.

**#43 · Smart pair wrapping**

When `textarea.selection_range().is_some()` and user types an opening bracket/quote,
wrap the selection instead of inserting the character.

Opening chars and their pairs:
```
'[' → ']'    '(' → ')'    '{' → '}'
'"' → '"'    '\'' → '\''  '`' → '`'
'*' → '*'    '_' → '_'
```

Implementation:
1. In the catch-all key handler, before `textarea.input(k)`, check if there's a
   selection and the key is one of the above openers
2. Extract selected text: `textarea.lines()` sliced by `selection_range()`
3. Call `textarea.delete_selection()` (tui-textarea API)
4. `textarea.insert_str(&format!("{opener}{selected}{closer}"))`
5. Position cursor at end of inserted text (tui-textarea's cursor will be there
   automatically after insert_str)

Edge cases:
- Multiline selection: insert opener at start line, closer at end line (respects newlines)
- Undo: a single Ctrl+Z undoes the entire wrap operation (tui-textarea handles this if
  the three operations are done as a single edit — check if tui-textarea has a
  `start_undo_group` API; if not, live with it being three undo steps)

Test: select "hello", type `[` → buffer becomes `[hello]`, cursor after `]`.

### Sprint 5: Syntax highlighting (#44, #45)

**#44 · Syntect fenced code block highlighting**

See DESIGN.md § "v1.5 Extension" for full design. Key points:

- Add `syntect` to `[dependencies]`
- Lazy grammar loading on first fenced block encountered (not startup)
- Cache keyed on block content hash — only re-highlight when block changes
- Limited grammar set: Rust, Python, JS/TS, Shell, JSON, TOML, YAML, SQL, Markdown
- Configurable via `[highlighting] grammars = ["rust", "python", ...]`
- Syntect spans merged with `fenced_bg` as background (syntect fg, yame's fenced_bg bg)
- Background thread for grammar loading; fall back to fenced_bg-only until ready
- Language tag fallback: unrecognized tag → silent fenced_bg-only

Integration point: `// TODO(v1.5): pass block content and language tag to syntect here`
in `decoration.rs` `Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))` handler.

**#45 · Move decoration pass to background thread**

_Do after #44 (syntect) since that's when it actually matters._

The seam is already correct: `build_decoration_map` is a pure free function.

```rust
// In App, add:
pub decoration_tx: std::sync::mpsc::Sender<String>,
pub decoration_rx: std::sync::mpsc::Receiver<(DecorationMap, usize)>,

// Debounce loop change:
// Was: app.decoration_map = build_decoration_map(...)
// Now: decoration_tx.send(text)?;

// Draw loop:
if let Ok((map, wc)) = app.decoration_rx.try_recv() {
    app.decoration_map = map;
    app.word_count = wc;
}
```

---

## v2 — Search, Line Numbers (#46, #47)

### #46 · `Ctrl+F` search with regex

Modal search bar at the bottom of the editor column (above the info line). 

- `Ctrl+F` opens search bar; typing updates matches live
- `Enter` / `n` / `N` for next/prev match
- `Escape` closes, returns focus to editor
- Matches highlighted using a temporary decoration layer that overlays the base map
- Regex via `regex` crate (add to `[dependencies]`)
- Case-insensitive by default; `Ctrl+I` (or similar) toggles

State to add to `App`:
```rust
pub search: Option<SearchState>,
// where:
pub struct SearchState {
    pub query: String,
    pub matches: Vec<(usize, usize, usize)>, // (line, char_start, char_end)
    pub current: usize,
}
```

The render path checks `app.search` and overlays match highlights after the decoration
pass but before selection — same pattern as selection overlay.

### #47 · Line numbers

`[ui] show_line_numbers = false` in config. The `// TODO(v2): line numbers gutter`
comment is in `layout.rs`.

- When enabled: gutter width = `digits(total_lines) + 1` columns
- `compute_layout` adjusts `column.x` by gutter width
- Renderer draws number for the first visual row of each logical line (only), blank for
  continuation rows — per the D6 decision
- Style: `theme.muted` fg, `theme.bg` bg

---

## Technical Debt Log

| Item | Location | Issue | Notes |
|---|---|---|---|
| State mutation inside `terminal.draw` | `main.rs` | — | Scroll clamping in draw closure; architecturally should be in event handler. No correctness impact. Low priority. |
| `#[mutants::skip]` on clipboard handlers | `clipboard.rs` | — | `handle_copy`/`handle_paste` skip mutation testing. Could be tested with mock. |

---

## Issue Index

| Issue | Title | Sprint |
|---|---|---|
| #13 | Phase 12: README & Distribution | v1 |
| #35 | v1 polish: italic warning, delimiter_blend, parent-dir, theme tokens + cleanup | v1 |
| #36 | v1.5: merge count_words into build_decoration_map | 1.5-S1 |
| #37 | v1.5: O(N²) char counting + allocation hot-paths | 1.5-S1 |
| #38 | v1.5: cache arboard::Clipboard in App | 1.5-S1 |
| #39 | v1.5: blockquote continuation indent | 1.5-S2 |
| #40 | v1.5: tab character expansion on load | 1.5-S2 |
| #41 | v1.5: CJK / wide character support | 1.5-S3 |
| #42 | v1.5: Ctrl+R config reload | 1.5-S4 |
| #43 | v1.5: smart pair wrapping | 1.5-S4 |
| #44 | v1.5: syntect fenced code highlighting | 1.5-S5 |
| #45 | v1.5: background decoration thread | 1.5-S5 |
| #46 | v2: Ctrl+F search with regex | v2 |
| #47 | v2: line numbers | v2 |
