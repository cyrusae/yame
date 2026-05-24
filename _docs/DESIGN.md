# `yame` Terminal Markdown Editor

Build a lightweight terminal Markdown editor in Rust called `yame` (Yet Another Markdown Editor). The goal is a fast-starting, pleasant editor for files like READMEs and CLAUDE.mds — something meaningfully lighter than VS Code while being more capable than nano for Markdown specifically.

---

## Stack

- **Rust** + **Ratatui** + **tui-textarea** + **pulldown-cmark**
- **serde** + **toml** for config
- **arboard** for system clipboard support
- Nerd Fonts / Powerline symbols assumed available in the user's terminal

---

## Invocation

```
yame [path]
```

Single optional file argument. No built-in file browser — file discovery is handled by a shell wrapper (documented in README, see below). The binary itself should:

- Accept an explicit file path and open it
- Accept a new filename that doesn't exist yet and open an empty buffer ready to save to that path
- With no argument: print a short usage error and exit cleanly

---

## Layout

- **Editing column**: centered at ~50% terminal width with equal left/right margins. Soft word wrap within the column. Layout is computed against current pane dimensions at every redraw — this is inherently correct in tmux splits and resized panes without special handling. Column width is `max(50% of pane width, min_cols)` where `min_cols` is a config value with a sensible default (pick a reasonable minimum — something in the 40–60 character range). Below `min_cols`, margins compress to zero and the editor fills the full pane width. Overridable via `[layout] min_cols = N` in config.
- **Mouse support**: enable crossterm mouse capture on startup (`EnableMouseCapture`); disable on exit (`DisableMouseCapture`) — critical, failure to disable leaves the terminal in a broken state. Pass mouse events to tui-textarea for click-to-place-cursor and scroll wheel support.
- **Scrollbar**: Ratatui `Scrollbar` widget on the right edge of the editing column, styled to match the theme (Unicode track + thumb).
- **Floating info line**: one line above the status bar, left-aligned. Background color matches `ui_background` (the main editor background) — this occludes text underneath without presenting as a bar or blocked-off region. No border. Content: `Ln 42, Col 8 · 1,204 words`.
- **Status bar** (bottom line): Powerline-style separators between segments.
    - Left: `[filename · ../parent/filename.md]` — filename only plus up to 2 parent directories of the path, shortened. Dirty flag: Nerd Fonts unsaved-disk icon or `[*]` suffix when buffer is modified.
    - Center: keybinding hints — `^S Save ^X Exit ^Z Undo ^Y Redo`
    - Right: reserved / empty for now

---

## Rendering Architecture

`tui-textarea` is used as the **text state backend only** — it owns the buffer, cursor position, undo/redo stack, and keymap. It does **not** own the visual output.

The rendering layer is implemented as a custom Ratatui widget that:

1. Reads `textarea.lines()` to get the current buffer content as a `&[String]`.
2. Applies the decoration map (see below) to produce a `Vec<ratatui::text::Line>` of styled `Span`s.
3. Draws those lines directly via a `Paragraph` (or equivalent custom `Widget` impl), positioned within the centered editing column.

This means `textarea.widget()` is **never called** for display. tui-textarea's built-in rendering is bypassed entirely. This is the only approach that supports inline per-span styling and blockquote soft-wrap continuation indentation — none of which are achievable through tui-textarea's public styling API.

Cursor and selection state are read from tui-textarea and re-applied in the custom renderer so they remain visually correct.

**Selection rendering**: selection applies a full fg+bg override — the selection colors replace span colors entirely for all covered characters. This keeps selection visually unambiguous regardless of what decoration is underneath. Composing selection with per-span colors would produce too many visual variations and make the selection hard to read.

---

## Inline Markdown Decoration

Run `pulldown-cmark` in **offset-iterator mode** on the full buffer text on every keystroke, debounced at ~50ms. Build a `HashMap<usize, Vec<Span>>` (line index → styled spans) from the parse output. The custom renderer (see above) applies this map when drawing each line.

**Threading model (v1): single-threaded timer loop.** The event loop uses `crossterm::event::poll()` with a short timeout (~16ms). Last-keystroke time is tracked with `std::time::Instant`; when 50 ms has elapsed since the last keystroke the decoration pass runs inline on the main thread before the next render. No background threads or channels. pulldown-cmark is fast enough on typical Markdown files that blocking the event loop for the decoration pass is not a real concern in v1.

**Isolation seam for v1.5:** the decoration pass must be implemented as a free function with a clear signature:

```rust
fn build_decoration_map(text: &str) -> DecorationMap
```

This makes it trivial to move the call to a background `std::thread` + `mpsc` channel in v1.5 (when syntect is added and grammar loading makes the pass expensive) without touching the renderer or event loop.

**Do not use line-by-line regex.** The parser must handle multi-line constructs correctly — fenced code blocks, blockquotes, and nested structures must all be handled via the parser's event stream.

### Delimiter visibility

Decoration is applied to the **full matched span including delimiters**. The raw syntax characters remain visible and are styled along with their content — they are not hidden or replaced. For example, all five characters of `*italic*` receive italic style + emphasis color. This applies to all inline elements.

### Elements to decorate

| Element                | Style                                                                                                                                                           |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `# H1`                 | Bold + accent color (per-level override if configured) + `heading_bg` background tint across full line width                                                    |
| `## H2`                | Bold + accent color + `heading_bg` tint                                                                                                                         |
| `### H3` and below     | Accent color + `heading_bg` tint, no extra weight                                                                                                               |
| `**bold**`             | Bold, including the `**` delimiters                                                                                                                             |
| `*italic*`             | Italic + emphasis color, including the `*` delimiters                                                                                                           |
| `inline code`          | Code color + distinct background tint; backtick delimiters included                                                                                             |
| `fenced blocks`        | Block-level background tint across all lines; no syntax highlighting inside blocks                                                                              |
| `> blockquote`         | Muted color, left-edge indicator `▌` (U+258C LEFT HALF BLOCK). On soft-wrapped lines, indent continuation text to align with the text start after `>` — do not wrap to column zero |
| `[link text](url)`     | Link text: underlined + accent color. URL: italic + muted color. All delimiters (`[`, `]`, `(`, `)`) included in styling                                        |
| `-` / `*` / `1.` lists | Preserve visual indentation; bullet/number in accent color                                                                                                      |
| `- [ ]` todo           | `[` `]` delimiters in accent color; content normal style                                                                                                                    |
| `- [x]` todo           | `[` `x` `]` in muted color; content in strikethrough + muted color                                                                                                         |
| Tables (GFM)           | Table headers: bold + accent color. Pipes (`\|`): muted delimiter style. Cell content: normal text. No column alignment in v1.                                  |

### Cursor line unfurl

The line the cursor is currently on: render raw syntax at normal weight (no decoration applied — the user sees literal `**`, `*`, `#` etc. while editing that line). All other lines: fully decorated. This prevents cursor-offset confusion inside styled text.

### Italic fallback

At startup, detect italic support by checking `$TERM` against a hardcoded known-good list: `xterm-256color`, `tmux-256color`, `screen-256color`, `kitty`, `alacritty`, `rio`, `wezterm`, `foot`, and any value prefixed with `xterm-kitty`. No additional crate is needed. If italics are unsupported:

- Fall back to emphasis color only (no italic style attribute applied anywhere)
- Display a one-time dismissible warning in the status bar: `⚠ Terminal does not support italics — using color fallback [press any key to dismiss]`

If detection is uncertain, assume italics are supported rather than disabling them unnecessarily.

---

## Edit Behavior

- **Undo/redo**: use tui-textarea's built-in linear undo/redo stack. `Ctrl+Z` / `Ctrl+Y`.
- **Save**: `Ctrl+S` saves in place. Show a brief `Saved.` confirmation in the status bar that clears after ~1.5s.
- **Exit flow** (nano pattern): if buffer is dirty, `Ctrl+X` prompts in the status bar: `Save modified buffer? [Y/N/Cancel]`. Y saves and exits, N exits without saving, Escape or C cancels back to editing.
- **Soft wrap + cursor movement**: use tui-textarea's default behavior. Do not override wrap or cursor movement — take whatever the library provides.
- **Copy/paste**:
    - `Ctrl+C`: copy selection (if any) or current line to system clipboard via `arboard`
    - `Ctrl+V`: paste from system clipboard at cursor position
    - **Copy behavior with soft-wrapped text:** When copying selected text, soft-wrap line breaks are not included — the clipboard receives the logical buffer content without renderer-inserted breaks. This preserves Markdown syntax when pasting into other tools.
    - **Clipboard failure handling:** if `arboard` returns an error (e.g. no clipboard provider on a headless system, or Wayland without a compositor), display a dismissible status bar warning: `⚠ Clipboard unavailable: <error>`. Do not panic or silently discard the operation.
- **Word count**: computed during the decoration debounce pass. Strip Markdown syntax by collecting only `Event::Text` payloads from a `pulldown-cmark` pass over the buffer, then split on whitespace and count tokens. This is an eyeball-size convenience feature — exact accuracy is not critical, but Markdown syntax characters should not inflate the count. Display live in the floating info line as `· N words`. Format with thousands separator for large counts.

---

## Theming & Config

Config file location: `~/.config/yame/config.toml` (XDG base dir spec — respect `$XDG_CONFIG_HOME` if set). Config is entirely optional; all values below are hardcoded defaults if the file is absent or a key is missing.

### Config error handling

If the config file is present but contains errors:

- **Invalid TOML syntax**: Log a warning to stderr and load defaults. The user can fix the syntax and reload (v1.5) or restart.
- **Invalid individual values** (e.g., malformed color `"not-a-color"`): Skip that key, load the default for it, and display a dismissible warning banner at the top of the editor: `⚠ Config warning: Invalid color value for theme.accent, using default`. The user can continue editing and fix the config file in another window.
- **Missing keys**: Use hardcoded defaults — no error needed, config is optional.

This approach keeps the editor usable while making problems visible. A user who typos a color can still work while they fix it.

### Architecture: base palette + derived tokens

The theme system has two levels:

1. **Base palette** — six colors the user sets to define the overall feel. Everything else derives from these automatically.
2. **Override tokens** — optional per-element overrides for users who want fine-grained control. Omitting any override means the derived default is used.

A user who only sets the base palette gets a fully coherent theme. Overrides are additive, never required.

### Base palette

```toml
[theme]
text    = "#cdd6f4"   # body text, bold, italic (emphasis is typographic, not chromatic)
accent  = "#cba6f7"   # headings, links, bullets — structural elements
muted   = "#6c7086"   # URLs, blockquotes, completed todos, receded punctuation
code    = "#a6e3a1"   # inline code and fenced blocks
bg      = "#1e1e2e"   # editor background; all other backgrounds derive from this
warning = "#fab387"   # dirty flag, italic-unsupported notice
```

Defaults are **Catppuccin Mocha** values as shown. Accept colors as hex strings (`#rrggbb`). Parse at startup and error clearly if a value is malformed.

### Derived token defaults

Compute all derived tokens at startup from the base palette. The blend function lerps RGB channels:

```rust
fn blend(fg: (u8, u8, u8), bg: (u8, u8, u8), ratio: f32) -> (u8, u8, u8) {
    // ratio: 0.0 = all bg, 1.0 = all fg
    let r = (fg.0 as f32 * ratio + bg.0 as f32 * (1.0 - ratio)) as u8;
    let g = (fg.1 as f32 * ratio + bg.1 as f32 * (1.0 - ratio)) as u8;
    let b = (fg.2 as f32 * ratio + bg.2 as f32 * (1.0 - ratio)) as u8;
    (r, g, b)
}
```

|Derived token|Default derivation|Purpose|
|---|---|---|
|`bold_color`|`text`|bold spans|
|`italic_color`|`text`|italic spans|
|`link_text`|`accent`|`[link text]` portion|
|`link_url`|`muted`|`(url)` portion|
|`blockquote_color`|`muted`|blockquote text|
|`todo_done`|`muted`|completed todo items|
|`heading_bg`|`blend(accent, bg, 0.15)`|full-width heading line tint|
|`code_bg`|`blend(code, bg, 0.22)`|inline code background|
|`fenced_bg`|`blend(code, bg, 0.12)`|fenced block background (subtler, spans many lines)|
|`ui_bg`|`bg`|editor background (floating info line uses this)|
|`ui_bar`|`blend(bg, (0,0,0), 0.15)`|status bar (slightly darker than bg)|
|`ui_text`|`text`|status bar text|

### Heading background span

The `heading_bg` tint applies **across the full logical line width of the centered editing column** (the width computed for `[layout] min_cols`), not the full pane width. This keeps the highlight visually contained to the editing area and maintains visual hierarchy even with wide margins.

### Delimiter colors

Delimiters (`*`, `**`, `#`, ```, `[`, `]`, `(`, `)`) are styled as a blend of their span's color toward `muted`. This makes them visibly recede relative to the content they wrap, while remaining tonally related to it. Computed per-span at decoration time:

```rust
let delimiter_color = blend(span_color, muted, delimiter_blend);
```

`delimiter_blend` defaults to `0.5` (tunable via override — this is the first value most users will want to adjust visually).

### Optional overrides

```toml
# [theme.override]
# bold_color       = "#cdd6f4"
# italic_color     = "#f5c2e7"   # e.g. Catppuccin pink for tonal distinction
# link_text        = "#cba6f7"
# link_url         = "#6c7086"
# blockquote_color = "#6c7086"
# todo_done        = "#6c7086"
# heading_bg       = "#302d45"   # skips blend computation if set
# heading_bg_blend = 0.15        # adjusts heading_bg blend intensity
# code_bg          = "#262637"
# fenced_bg        = "#222233"
# ui_bg            = "#1e1e2e"
# ui_bar           = "#181825"
# ui_text          = "#cdd6f4"
# delimiter_blend  = 0.5         # 0.0 = full muted, 1.0 = full span color

# [theme.headings]               # per-level accent color overrides
# h1 = "#cba6f7"
# h2 = "#89b4fa"
# h3 = "#94e2d5"
# h4 = "#a6e3a1"
# h5 = "#f5c2e7"
# h6 = "#fab387"
```

Implement all override lookups — check override first, fall back to derived default. The config key structure should be live and functional even where defaults produce identical values.

---

## Planned Features (Roadmap)

### v1.5: Config Reload & Syntax Highlighting

**Config reload:** `Ctrl+R` reloads `~/.config/yame/config.toml` and applies changes to theming and layout without closing the editor. If the reload fails, display an error in the status bar and keep the previous config active.

**Syntax highlighting in fenced code blocks:** See the detailed section below.

**Smart pair wrapping:** When text is selected, typing an opening bracket/quote (`[`, `(`, `{`, `"`, `'`, `` ` ``, `*`, `_`) wraps the selection with matching pair. Cursor moves to end of wrapped text. No automatic escaping — user controls nesting.

### v2: Search/Replace, Line Numbers

**Search and replace:** `Ctrl+F` opens a search dialog. Basic regex support. `Ctrl+H` for replace, or as an add-on from the search interaction.

**Line numbers:** Optional display of line numbers in a narrow gutter. Controlled via `[ui] show_line_numbers = false` in config (reserved for v2 implementation).

### v3: Table Rendering

**Table alignment and borders:** v1 detects and colors GFM tables. v3 adds column-width computation, alignment, and pretty borders. Tables may overflow the centered column width for readability. Controlled via `[layout] allow_table_overflow = true` in config.

---

## v1.5 Extension: Syntax Highlighting in Fenced Code Blocks

> Do not implement in v1. This section documents the intended approach so v1 leaves the right seams.

### Library

Use **`syntect`** for syntax highlighting. It supports TextMate grammars, ships with built-in themes, and outputs styled spans that map directly onto Ratatui's `Span` model — the same structure the decoration pass already produces.

### Integration point

The v1 decoration pass already identifies fenced code block line ranges and applies `fenced_bg` as a background tint. That range detection is the seam. In v1, leave a clearly marked comment at that site:

```rust
// TODO(v1.5): pass block content and language tag to syntect here
// language tag is available from pulldown-cmark's CodeBlock(CodeBlockKind::Fenced(lang)) event
// replace fenced_bg-only spans with syntect-highlighted spans that retain fenced_bg as background
```

### Design notes for the implementer

**Grammar loading**: load syntect grammars lazily on first fenced block encountered, not at startup. Grammar loading is the expensive step — deferring it keeps startup fast. Load in a background thread if possible; fall back to `fenced_bg`-only tint until highlighting is ready.

**Grammar scope**: bundle a limited set of common grammars rather than all 300+ to keep binary size manageable. Suggested set: Rust, Python, JavaScript/TypeScript, Shell/Bash, JSON, TOML, YAML, SQL, Markdown. Make the set configurable via `[highlighting] grammars = ["rust", "python", ...]` in config.

**Per-keystroke cost**: do not re-highlight a fenced block on every keystroke. Maintain a cache keyed on block content hash — only re-run syntect when the block content actually changes. The decoration debounce already helps, but the cache is the more important guard.

**Span merging**: syntect-produced spans need `fenced_bg` applied as their background while preserving syntect's foreground colors. Merge carefully — do not let syntect's theme override `fenced_bg`.

**Theme mapping**: syntect has its own theme system. Either map yame's base palette to a syntect theme at startup, or use syntect's built-in themes (e.g. `base16-ocean.dark`) as a starting point. The latter is simpler for v1.5; full palette integration can come later.

**Language tag fallback**: if the fenced block has no language tag, or the tag is unrecognized, fall back silently to `fenced_bg`-only tint. No error, no warning.

---

## README Requirements

The README should include:

### Shell wrapper (required)

Document a shell function to add to `.bashrc` / `.zshrc` that handles smart invocation. The binary stays pure; the shell layer handles file discovery.

```bash
yame() {
  local target
  if [[ -z "$1" ]]; then
    # No argument: fuzzy find markdown files in current directory
    target=$(fd --type f --extension md | fzf --select-1 --exit-0 --preview 'head -20 {}')
  elif [[ "$1" == */* || "$1" == *.* ]]; then
    # Looks like an explicit path: pass through directly
    target="$1"
  else
    # Treat as fuzzy search term
    target=$(fd --type f "$1" | fzf --select-1 --exit-0 --preview 'head -20 {}')
  fi
  [[ -n "$target" ]] && command yame "$target"
}
```

Note that this requires `fd` and `fzf`. Include a fallback note for users without `fd`:

```bash
# Fallback: replace fd with find if fd is not installed
target=$(find . -name "*.md" | fzf --select-1 --exit-0 --preview 'head -20 {}')
```

### Install instructions

- `cargo install` from source
- Where to put the shell function
- Config file location (`~/.config/yame/config.toml`) and theming instructions
- Nerd Fonts requirement note
- `fd` and `fzf` as optional dependencies for the shell wrapper

### Keybinding reference

Document all keybindings in a quick reference section:

```
Editing:
  Ctrl+Z, Ctrl+Y        Undo, Redo
  Ctrl+C, Ctrl+V        Copy, Paste
  Arrow keys            Move cursor
  Shift + Arrow         Select text (if tui-textarea supports)

File:
  Ctrl+S                Save
  Ctrl+X                Exit (prompts if dirty)
```

---

## What This Is Not

To keep scope clear for implementation:

- No file browser or directory navigation in the binary
- No syntax highlighting inside fenced code blocks (v1.5)
- No table column alignment or pretty borders (v3)
- No split-pane preview
- No plugin system
- No collaborative features
- No git integration
- No search/replace (v2)
- No line numbers (v2)