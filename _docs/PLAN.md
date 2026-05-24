# PLAN.md — `yame` v1 Implementation Plan

## Overview

This plan covers the full v1 build sequence for `yame`. Each phase ends with something runnable or meaningfully testable. Dependencies between phases are explicit. Roadmap seams (v1.5/v2/v3) are marked where they occur.

No Rust source exists yet. The project root has only docs, a README stub, and `.gitignore`. Every step below starts from scratch.

---

## Phase 0 — Project Scaffold

**Goal:** A compiling, running Rust binary that opens, prints a placeholder frame, and exits cleanly.

### Step 0.1 — `cargo init`

Run `cargo init --name yame` in the project root. This creates `src/main.rs` and `Cargo.toml`.

**Acceptance:** `cargo run` prints "Hello, world!" and exits with code 0.

### Step 0.2 — Add all v1 dependencies to `Cargo.toml`

```toml
[dependencies]
ratatui = "0.29"
tui-textarea = { version = "0.7", features = ["crossterm"] }
crossterm = "0.28"
pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
arboard = "3"

[profile.release]
strip = true
opt-level = 3
```

Pin minor versions. Do not use `*`.

**Acceptance:** `cargo build` succeeds with no errors. Warnings about unused deps are acceptable at this stage.

### Step 0.3 — Module skeleton

Create the source module tree:

```
src/
  main.rs          — arg parsing, terminal setup/teardown, top-level event loop
  app.rs           — App struct (all mutable state)
  config.rs        — Config/Theme loading, blend(), derived tokens
  decoration.rs    — build_decoration_map() and DecorationMap type
  renderer.rs      — custom Ratatui widget
  layout.rs        — column width computation, area splitting
  clipboard.rs     — arboard wrapper with error handling
  status.rs        — StatusMessage (timed messages, exit prompt state)
```

Declare all modules in `main.rs` with `mod foo;`. Each file can be empty `// stub` for now.

**Acceptance:** `cargo build` succeeds. No logic yet.

---

## Phase 1 — Terminal Lifecycle

**Goal:** A black terminal screen that responds to `Ctrl+C`/`q` and restores the terminal on exit. This is the foundation everything else sits on.

### Step 1.1 — Terminal setup and teardown in `main.rs`

Implement the standard Ratatui crossterm setup:

1. `enable_raw_mode()`
2. `execute!(stdout, EnterAlternateScreen, EnableMouseCapture)`
3. Create `Terminal::new(CrosstermBackend::new(stdout))`
4. On exit (normal or panic): `disable_raw_mode()`, `execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)`

Use a `setup_panic_hook()` function that runs teardown inside `std::panic::set_hook`. This is critical — a panicking process that skips `DisableMouseCapture` leaves the terminal broken for the user.

**Warning:** `DisableMouseCapture` must be called on every exit path including panics. Test this by inserting a deliberate `panic!()` and verifying the terminal is clean afterward.

### Step 1.2 — Minimal event loop in `main.rs`

```rust
loop {
    terminal.draw(|f| { /* blank frame */ })?;
    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(k) if k.code == KeyCode::Char('q') => break,
            _ => {}
        }
    }
}
```

**Acceptance:** `cargo run -- somefile.md` shows a blank terminal, `q` exits cleanly, terminal is restored.

### Step 1.3 — CLI argument parsing in `main.rs`

Parse `std::env::args()` manually (no clap needed for a single positional arg):

- 0 args: print `"Usage: yame <file>"` to stderr, exit with code 1
- 1 arg: store the path; if the file does not exist that is fine (new buffer)
- 2+ args: same usage error

**Acceptance:** `yame` with no args prints usage and exits 1. `yame somefile.md` opens (blank screen) and exits on `q`.

---

## Phase 2 — Config & Theming

**Goal:** Config is loaded at startup (with defaults when absent), and the `Theme` struct with all derived tokens is available throughout the app.

### Step 2.1 — `Config` and `Theme` structs in `config.rs`

Define structs with `#[derive(Deserialize, Default)]`:

```rust
pub struct Palette { pub text, accent, muted, code, bg, warning: String }
pub struct ThemeOverrides { /* all override fields as Option<String> */ }
pub struct HeadingColors { pub h1, h2, h3, h4, h5, h6: Option<String> }
pub struct Config { pub theme: PaletteConfig, pub layout: LayoutConfig }
```

Separate the raw TOML-deserialized `Config` from the computed `Theme` (resolved `Color` values). `Theme` is never serialized.

### Step 2.2 — `blend()` utility

Implement exactly as specified in the design doc. Place in `config.rs`. Unit test it:

- `blend((255,0,0), (0,0,0), 0.5)` → `(127, 0, 0)`
- `blend(fg, bg, 0.0)` → `bg`
- `blend(fg, bg, 1.0)` → `fg`

**Acceptance:** Unit tests pass.

### Step 2.3 — Color parsing

Parse `#rrggbb` strings to `(u8, u8, u8)`. Return a `Result` with a clear error message including the field name. Convert the tuple to `ratatui::style::Color::Rgb(r, g, b)`.

No external color-parsing crate needed — it is 6 hex digits.

### Step 2.4 — Derived token computation

Implement `Theme::from_palette(palette: &Palette, overrides: &ThemeOverrides, headings: &HeadingColors) -> Theme`. For each derived token:

1. Check if an override exists in `ThemeOverrides`; if so, parse and use it
2. Otherwise compute from the blend formulas in the design doc

Add `selection_bg` and `selection_fg` to `Theme` here: derive as `blend(accent, bg, 0.6)` for `selection_bg` and `bg` for `selection_fg`.

This function is called once at startup and produces the `Theme` used everywhere else.

### Step 2.5 — Config file loading

XDG path resolution:

```rust
fn config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME").map(PathBuf::from)
                .unwrap_or_default()
                .join(".config")
        });
    base.join("yame").join("config.toml")
}
```

Load sequence:
1. If path does not exist → return `Config::default()` (Catppuccin Mocha)
2. If file exists but TOML parse fails → `eprintln!` warning, return `Config::default()`
3. If individual color fields are invalid → skip them (use field default), queue a `StatusMessage` warning for display

**Warning:** Config errors must not crash the editor. The user must be able to continue editing.

**Acceptance:** `cargo test` covers blend, color parsing, and the XDG path fallback.

### Step 2.6 — Italic support detection

In `main.rs` or a `terminal.rs` helper:

```rust
fn supports_italic() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    matches!(term.as_str(),
        "xterm-256color" | "tmux-256color" | "screen-256color" |
        "kitty" | "alacritty" | "rio" | "wezterm" | "foot"
    ) || term.starts_with("xterm-kitty")
}
```

Store the result in `App`. When false, the renderer never applies `Modifier::ITALIC` — it applies the `italic_color` only. When false also queue the dismissible italic warning message.

**Acceptance:** Compiles; test with `TERM=dumb cargo run -- file.md` and verify the flag is false.

---

## Phase 3 — App State & tui-textarea Integration

**Goal:** A `TextArea` is initialized with file content, and the event loop routes keyboard and mouse events into it. The buffer is editable.

### Step 3.1 — `App` struct in `app.rs`

```rust
pub struct App {
    pub textarea: TextArea<'static>,
    pub file_path: PathBuf,
    pub is_dirty: bool,
    pub saved_content: Option<Vec<String>>,  // for dirty-flag comparison after undo
    pub theme: Theme,
    pub italic_support: bool,
    pub last_keystroke: Option<Instant>,
    pub decoration_map: DecorationMap,
    pub word_count: usize,
    pub status: StatusLine,
    pub config_warnings: Vec<String>,
}
```

### Step 3.2 — File loading into `TextArea`

```rust
fn load_file(path: &Path) -> io::Result<TextArea<'static>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Ok(TextArea::new(lines))
    } else {
        Ok(TextArea::default())  // new empty buffer
    }
}
```

**Note:** `TextArea::new()` accepts `Vec<String>`. Do not call `textarea.widget()` anywhere — it is never used for display.

### Step 3.3 — Event routing

In the event loop:

```rust
match event::read()? {
    Event::Key(key) => {
        // intercept app-level keys before passing to textarea
        match (key.modifiers, key.code) {
            (CONTROL, KeyCode::Char('s')) => handle_save(&mut app)?,
            (CONTROL, KeyCode::Char('x')) => handle_exit(&mut app)?,
            (CONTROL, KeyCode::Char('c')) => handle_copy(&mut app),
            (CONTROL, KeyCode::Char('v')) => handle_paste(&mut app),
            _ => {
                app.textarea.input(key);
                app.is_dirty = true;
                app.last_keystroke = Some(Instant::now());
            }
        }
    }
    Event::Mouse(mouse) => { app.textarea.input(Event::Mouse(mouse)); }
    Event::Resize(_, _) => { /* next draw() picks up new dimensions automatically */ }
    _ => {}
}
```

**Warning:** Do not pass `Ctrl+S`, `Ctrl+X`, `Ctrl+C`, `Ctrl+V` to `textarea.input()`. tui-textarea will consume them as internal keymap events.

**Acceptance:** Launch with a real file. Type text, see cursor move (even with a placeholder renderer). The buffer is live.

### Step 3.4 — Dirty flag and undo/redo

`is_dirty` is set true on any textarea mutation. It is cleared after save. `Ctrl+Z` / `Ctrl+Y` pass directly to `textarea.input()` as-is (tui-textarea handles undo/redo internally). After undo/redo, recompute `is_dirty` by comparing `textarea.lines()` to `saved_content`.

---

## Phase 4 — Layout Engine

**Goal:** The centered editing column is computed correctly and all layout rectangles are available to the renderer.

### Step 4.1 — Column width computation in `layout.rs`

```rust
pub struct EditorLayout {
    pub full: Rect,
    pub column: Rect,
    pub scrollbar: Rect,   // 1 column wide, right of editing column
    pub info_line: Rect,   // second-to-last row
    pub status_bar: Rect,  // last row
}

pub fn compute_layout(area: Rect, min_cols: u16) -> EditorLayout {
    let col_width = (area.width / 2).max(min_cols).min(area.width);
    let margin = (area.width.saturating_sub(col_width)) / 2;
    // editing area: area.height - 2 rows (info_line + status_bar)
    // column starts at x = margin, scrollbar at x = margin + col_width
    // ...
}
```

The scrollbar occupies the 1 column immediately to the right of the editing column (within the right margin).

**Acceptance:** Unit test `compute_layout` for: narrow terminal (width < min_cols → zero margins), wide terminal, and exact min_cols boundary.

### Step 4.2 — Scroll offset tracking

Maintain `scroll_top: usize` in `App`. Update after every event that may move the cursor:

```rust
let (cursor_row, _) = app.textarea.cursor();
let visible_rows = layout.column.height as usize;
if cursor_row < app.scroll_top {
    app.scroll_top = cursor_row;
}
if cursor_row >= app.scroll_top + visible_rows {
    app.scroll_top = cursor_row - visible_rows + 1;
}
```

`visible_rows` comes from `EditorLayout::column.height`. Compute layout before this check each frame.

---

## Phase 5 — Status Bar & Info Line

**Goal:** Status bar and info line render with real data. These use simple Ratatui widgets and can be built before the main editor renderer.

### Step 5.1 — `StatusLine` state in `status.rs`

```rust
pub enum StatusMode {
    Normal,
    TimedMessage { text: String, expires_at: Instant },
    DismissibleMessage(String),
    ExitPrompt,
}

pub struct StatusLine {
    pub mode: StatusMode,
}

impl StatusLine {
    pub fn set_timed(&mut self, text: impl Into<String>, duration: Duration) { ... }
    pub fn set_dismissible(&mut self, text: impl Into<String>) { ... }
    pub fn tick(&mut self) { /* clear TimedMessage if expired */ }
    pub fn dismiss(&mut self) { /* clear DismissibleMessage */ }
}
```

### Step 5.2 — Status bar widget

Implement as `fn render_status_bar(f: &mut Frame, area: Rect, app: &App)`.

**Left segment:** filename. Take `file_path.components()`, join the last 3 (up to 2 parent dirs + filename). Dirty flag: append ` [*]` when `app.is_dirty` (use `[*]` for v1; Nerd Fonts icon can be added as a config option later).

**Center segment:** `^S Save  ^X Exit  ^Z Undo  ^Y Redo`

**Right segment:** empty in v1.

Powerline separators: `\u{e0b0}` (right-pointing filled arrow) between segments, with color transitions between `ui_bar` and segment backgrounds.

In `ExitPrompt` mode: render full width as `Save modified buffer? [Y/N/Cancel]` in `warning` color.

In `TimedMessage`/`DismissibleMessage` mode: render the message text across the center of the bar.

**Acceptance:** Status bar renders with filename and hints. Dirty flag appears when buffer is modified.

### Step 5.3 — Floating info line

Render at `EditorLayout::info_line`, left-aligned, with `ui_bg` background:

```
Ln 42, Col 8 · 1,204 words
```

Cursor position from `textarea.cursor()` (0-indexed; display as 1-indexed). Word count from `app.word_count`. Thousands separator: manual helper — no locale crate needed.

**Acceptance:** Compiles and renders. Word count shows 0 until Phase 6 wires it in.

### Step 5.4 — Scrollbar widget

Use `ratatui::widgets::Scrollbar` in vertical orientation in `EditorLayout::scrollbar`. Parameters: `content_length = total_lines`, `position = scroll_top`. Style: track as `ui_bar`, thumb as `accent`.

---

## Phase 6 — Decoration Engine

**Goal:** `build_decoration_map()` is implemented and produces correct styled spans for all v1 Markdown elements.

### Step 6.1 — Types in `decoration.rs`

```rust
pub type DecorationMap = HashMap<usize, Vec<StyledSpan>>;

pub struct StyledSpan {
    pub char_start: usize,   // char index within the line (not byte index)
    pub char_end: usize,
    pub style: Style,
    pub is_blockquote: bool, // flag for renderer to handle continuation indent
    pub full_line_bg: Option<Color>, // for heading_bg — renderer expands to column width
}
```

### Step 6.2 — Byte-to-line/char mapping

Before iterating events, build a lookup structure from the full buffer text:

```rust
// Returns (line_index, char_offset_within_line) for a given byte offset
fn byte_to_line_char(line_starts: &[usize], text: &str, byte: usize) -> (usize, usize) {
    let line = line_starts.partition_point(|&s| s <= byte).saturating_sub(1);
    let line_start_byte = line_starts[line];
    let char_col = text[line_start_byte..byte].chars().count();
    (line, char_col)
}
```

Precompute `line_starts: Vec<usize>` by scanning `text` for `\n` byte positions (one pass, O(n)).

### Step 6.3 — `build_decoration_map` signature

```rust
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
    cursor_line: usize,
) -> DecorationMap
```

This signature is the v1.5 migration seam. Do not change it. When v1.5 moves this to a background thread, only the call site changes.

Use pulldown-cmark's offset iterator:

```rust
let parser = Parser::new_with_broken_link_callback(
    text,
    Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    None,
).into_offset_iter();
```

### Step 6.4 — Implement each element

Work through elements in this order (simpler to more complex):

**a. Headings (H1–H6)**

`Event::Start(Tag::Heading { level, .. })` with range covering the full line. Apply:
- H1/H2: `bold + accent_color` (or per-level override from `theme.headings`)
- H3+: `accent_color` only, no bold
- All headings: set `full_line_bg = Some(theme.heading_bg)` on every span in the line

**b. Bold**

`Event::Start(Tag::Strong)` range includes `**` delimiters. Apply `bold` to the whole range. For the delimiter characters (first 2 and last 2 chars of range): `blend(text_color, muted, delimiter_blend)`.

**c. Italic**

`Event::Start(Tag::Emphasis)` range includes `*` delimiters. Apply `italic_color`; apply `Modifier::ITALIC` only if `italic_support` is true. Delimiter characters: `blend(italic_color, muted, delimiter_blend)`.

**d. Inline code**

`Event::Code(_)` range includes backtick delimiters. Apply `code_color` fg + `code_bg` background to the whole range.

**e. Fenced code blocks**

`Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))` gives start; `Event::End(...)` gives end. Apply `fenced_bg` background to every line in the range.

```rust
// TODO(v1.5): pass block content and language tag to syntect here
// language tag is available from pulldown-cmark's CodeBlock(CodeBlockKind::Fenced(lang)) event
// replace fenced_bg-only spans with syntect-highlighted spans that retain fenced_bg as background
```

**f. Blockquotes**

`Event::Start(Tag::BlockQuote)` range spans all lines. For each line in range:
- Prepend a `▌` (U+258C) character in `muted` color at char_start = 0, char_end = 1
- Apply `blockquote_color` to the remaining line content
- Set `is_blockquote = true` on all spans in the line (renderer uses this for continuation indent)

**g. Links**

`Event::Start(Tag::Link { dest_url, .. })` range covers `[text](url)`. Parse the range text to find the `](` boundary:
- `[` … `]` portion: `link_text` color + underline; `[` and `]` characters use delimiter color
- `(` … `)` portion: `link_url` color + italic (if supported); `(` and `)` use delimiter color

**h. Lists and bullets**

`Event::Start(Tag::Item)` gives each item's range. Identify the bullet/number character at the start of the line (char 0 or after indentation) and apply `accent_color` to it. List content remains normal style.

**i. Todo items**

`Event::TaskListMarker(checked)`:
- `false` (unchecked `- [ ]`): `[` and `]` in `accent_color`, space between in normal style
- `true` (checked `- [x]`): entire item line in `muted` + `Modifier::CROSSED_OUT`

**j. Tables (GFM)**

`Event::Start(Tag::TableHead)` → header cells: `bold + accent_color`. Pipe characters `|` throughout the table: `muted` style. Cell content: normal. No column alignment in v1.

```rust
// TODO(v3): table column alignment, pretty borders, allow_table_overflow
```

### Step 6.5 — Cursor line exclusion

After building the full map, remove the cursor line's entries:

```rust
map.remove(&cursor_line);
```

This ensures raw Markdown syntax is visible while editing that line.

### Step 6.6 — Word count

Run a second pass (reuse the same debounce window — do not add a separate timer):

```rust
pub fn count_words(text: &str) -> usize {
    Parser::new(text)
        .filter_map(|e| match e {
            Event::Text(s) | Event::Code(s) => Some(s.split_whitespace().count()),
            _ => None,
        })
        .sum()
}
```

**Acceptance:** Unit tests assert correct span presence for headings, bold, italic, inline code, and that the cursor line has no entries.

---

## Phase 7 — Custom Renderer

**Goal:** The editor area renders with full Markdown decoration, cursor, and selection highlighting.

### Step 7.1 — `MarkdownView` widget in `renderer.rs`

```rust
pub struct MarkdownView<'a> {
    pub lines: &'a [String],
    pub decoration_map: &'a DecorationMap,
    pub scroll_top: usize,
    pub cursor: (usize, usize),
    pub selection: Option<((usize, usize), (usize, usize))>,
    pub theme: &'a Theme,
    pub italic_support: bool,
    pub column_width: u16,
}

impl Widget for MarkdownView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) { ... }
}
```

### Step 7.2 — Line rendering pipeline

For each visible logical line `i` (from `scroll_top` to `scroll_top + visible_height`):

1. Get `lines[i]` as the raw string
2. Soft-wrap it into visual rows (see Step 7.5)
3. For each visual row, look up decoration spans intersecting that row's char range
4. Split the row string at span boundaries into `ratatui::text::Span` objects
5. Expand `full_line_bg` spans to fill `column_width`
6. Apply selection overlay (see Step 7.4)
7. Write the resulting `Line` into `buf` at the correct row offset

### Step 7.3 — Span boundary splitting

Given a raw line string and sorted `Vec<StyledSpan>`:

```
chars [0..span1.char_start]        → default style
chars [span1.char_start..char_end] → span1.style
chars [span1.char_end..span2.char_start] → default style
...
```

Use `str::char_indices()` to convert char ranges to byte slices for safe `&str` slicing. Spans must not overlap; if they do (shouldn't in v1), clip the later span's start to the prior span's end rather than panicking.

### Step 7.4 — Selection overlay

Get selection from `textarea.selection_range()`. For each character in the selection range (across all visual rows of the selection), **replace** the cell's style entirely with `Style::default().fg(theme.selection_fg).bg(theme.selection_bg)`. This is a full fg+bg override — span colors are discarded within the selection.

Apply selection after all other styling, as a final pass over the affected rows in `buf`.

### Step 7.5 — Soft-wrap

```rust
fn wrap_line<'a>(s: &'a str, width: usize) -> Vec<&'a str> {
    // break at last space before width; hard-break if no space
}
```

Blockquote continuation: when `is_blockquote` is true on any span in the line, indent continuation visual rows by 2 chars (to align with text after `▌ `).

`scroll_top` is in logical-line space. The renderer converts to visual rows on each draw. This means scrolling can jump by >1 visual row when a long wrapped line scrolls off the top — acceptable for v1.

### Step 7.6 — Cursor rendering

After writing all spans for a line, set the cell at the cursor position:

```rust
let cell = buf.get_mut(area.x + cursor_col as u16, area.y + cursor_visual_row as u16);
cell.set_style(Style::default().fg(theme.bg).bg(theme.accent));
```

### Step 7.7 — Wire into event loop

```rust
terminal.draw(|f| {
    let layout = compute_layout(f.area(), app.config.layout.min_cols);
    let view = MarkdownView {
        lines: app.textarea.lines(),
        decoration_map: &app.decoration_map,
        scroll_top: app.scroll_top,
        cursor: app.textarea.cursor(),
        selection: get_selection(&app.textarea),
        theme: &app.theme,
        italic_support: app.italic_support,
        column_width: layout.column.width,
    };
    f.render_widget(view, layout.column);
    render_status_bar(f, layout.status_bar, &app);
    render_info_line(f, layout.info_line, &app);
    f.render_stateful_widget(scrollbar, layout.scrollbar, &mut scrollbar_state);
})?;
```

**Acceptance:** A real Markdown file opens and renders with decoration. Headings have tinted backgrounds. Bold and italic are styled. Cursor is visible. Selection highlights correctly.

---

## Phase 8 — Debounce Loop & Decoration Trigger

**Goal:** The decoration pass runs debounced at 50ms, single-threaded, triggered by keystroke timing.

### Step 8.1 — Debounce in the event loop

```rust
const DEBOUNCE: Duration = Duration::from_millis(50);
const POLL_TIMEOUT: Duration = Duration::from_millis(16);

loop {
    // Fire decoration pass if debounce has elapsed
    if let Some(t) = app.last_keystroke {
        if t.elapsed() >= DEBOUNCE {
            let text = app.textarea.lines().join("\n");
            let cursor_line = app.textarea.cursor().0;
            app.decoration_map = build_decoration_map(&text, &app.theme, app.italic_support, cursor_line);
            app.word_count = count_words(&text);
            app.last_keystroke = None;
        }
    }
    app.status.tick();  // clear expired timed messages

    terminal.draw(|f| { ... })?;

    if event::poll(POLL_TIMEOUT)? {
        // process event, set app.last_keystroke = Some(Instant::now()) on keystroke
    }
}
```

**Seam note:** `build_decoration_map` and `count_words` are pure functions. When v1.5 moves them to a background thread, the call site becomes `tx.send(text)` + `rx.try_recv()`. No other changes are needed.

**Acceptance:** Type rapidly in a long file. Decoration updates ~50ms after typing stops. The app stays responsive during fast typing.

---

## Phase 9 — File Operations & Edit Behaviors

**Goal:** Save, exit flow, copy/paste, and dirty tracking all work correctly.

### Step 9.1 — Save (`Ctrl+S`) in `main.rs`

```rust
fn handle_save(app: &mut App) -> io::Result<()> {
    let content = app.textarea.lines().join("\n");
    fs::write(&app.file_path, &content)?;
    app.saved_content = Some(app.textarea.lines().to_vec());
    app.is_dirty = false;
    app.status.set_timed("Saved.", Duration::from_millis(1500));
    Ok(())
}
```

On write error: `app.status.set_dismissible(format!("⚠ Save failed: {err}"))`. Do not crash.

### Step 9.2 — Exit flow (`Ctrl+X`)

```rust
fn handle_exit(app: &mut App) -> ExitAction {
    if app.is_dirty {
        app.status.mode = StatusMode::ExitPrompt;
        ExitAction::Continue
    } else {
        ExitAction::Quit
    }
}
```

When `ExitPrompt` is active, intercept key events before textarea:
- `Y`/`y`: save, then quit
- `N`/`n`: quit without saving
- `Escape`, `C`, or `c`: `app.status.mode = StatusMode::Normal`, resume editing

**Acceptance:** Edit a file, press `Ctrl+X`, confirm the prompt appears, press Y, file is saved, editor exits.

### Step 9.3 — Clipboard (`clipboard.rs`)

```rust
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .map_err(|e| e.to_string())
}

pub fn paste_from_clipboard() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .map_err(|e| e.to_string())
}
```

**Copy:** if textarea has a selection, copy selected text (read from `textarea.lines()` sliced by selection range — no soft-wrap breaks). Otherwise copy the current line.

**Paste:** `textarea.insert_str(&text)` with the clipboard content.

**On error:** `app.status.set_dismissible(format!("⚠ Clipboard unavailable: {err}"))`.

**Acceptance:** Copy a line, paste into another terminal window. Works on macOS, Linux X11/Wayland.

---

## Phase 10 — Polish & Edge Cases

**Goal:** All warnings work, edge cases are handled, and the editor is solid on narrow and wide terminals.

### Step 10.1 — Dismissible warnings

On any key event, if `status.mode` is `DismissibleMessage`, call `status.dismiss()` before processing the key normally. Cover all design doc cases:
- Italic unsupported warning (queued at startup)
- Clipboard error
- Config parse warning

### Step 10.2 — Config warning banner

For `app.config_warnings: Vec<String>` (populated during config loading), render at the top of the editing column (above first content line) with `warning` fg on `ui_bar` bg:

```
⚠ Config warning: Invalid color value for theme.accent, using default  [any key to dismiss]
```

Dismiss on any keypress (same mechanism as status bar dismissible messages, but clears `config_warnings` instead).

### Step 10.3 — Narrow terminal handling

`compute_layout` must never return negative widths or zero-height areas. Clamp:

```rust
let col_width = col_width.min(area.width);
let content_height = area.height.saturating_sub(2);
```

When `area.width < min_cols`: margins are 0, editing column fills full pane width.

### Step 10.4 — Resize handling

`Event::Resize` requires no special handling — the next `terminal.draw()` call receives the new dimensions from `f.area()`. Verify that `scroll_top` clamping runs against the new dimensions each frame, not cached old ones.

### Step 10.5 — Multi-byte character safety

All `&str` slicing by index must go through char-boundary-safe paths. The `byte_to_line_char` helper (Phase 6.2) handles the decoration map. In the renderer, use `str::char_indices()` or `chars().nth()` rather than direct byte indexing. Add a helper if needed:

```rust
fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
}
```

---

## Phase 11 — Testing & Hardening

### Step 11.1 — Unit tests

| Module | Tests |
|---|---|
| `config.rs` | `blend()`, color parse success/failure, XDG path, derived token computation |
| `decoration.rs` | Each element type, cursor line exclusion, multi-byte chars in headings/bold |
| `layout.rs` | `compute_layout` at: width < min_cols, width = min_cols, wide terminal |
| `status.rs` | Timed message expiry, dismissible message clearing on keypress |
| `renderer.rs` | Span boundary splitting, selection overlay, wrap_line |

### Step 11.2 — Integration smoke test

`tests/integration.rs`:
1. Construct a Markdown string with headings, bold, links, fenced blocks
2. Call `build_decoration_map` with a synthetic theme
3. Assert heading lines have `full_line_bg` set
4. Assert bold ranges include delimiter spans
5. Assert `cursor_line` has no entries in the map

Runs in CI without a terminal.

### Step 11.3 — CI setup

`.github/workflows/ci.yml`: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`. Use `ubuntu-latest`.

---

## Phase 12 — README & Distribution

### Step 12.1 — README content

Per the design doc's README requirements:

1. **Install:** `cargo install --path .` from source (crates.io publish is a separate step)
2. **Shell wrapper:** full `yame()` function for `.bashrc`/`.zshrc`, `fd`/`fzf` note, `find` fallback
3. **Config:** path, full base palette with Catppuccin Mocha defaults, override table, heading overrides
4. **Keybinding reference:** the table from the design doc
5. **Nerd Fonts note:** Powerline separators in the status bar require a patched font
6. **Optional deps:** `fd`, `fzf` for the shell wrapper

### Step 12.2 — `Cargo.toml` publishing metadata

```toml
[package]
name = "yame"
version = "0.1.0"
edition = "2021"
description = "A lightweight terminal Markdown editor"
license = "MIT"
exclude = [".chainlink", "_docs"]
keywords = ["markdown", "editor", "terminal", "tui"]
categories = ["command-line-utilities", "text-editors"]
```

---

## Seams for Future Phases (Do Not Implement in v1)

Mark with comments in code; do not build the feature:

**v1.5 — Background decoration thread**
`build_decoration_map` is already isolated as a free function. Change the call site to `std::thread::spawn` + `mpsc::channel`. No other changes needed. Leave a `// TODO(v1.5): move to background thread` comment at the call site in the event loop.

**v1.5 — syntect fenced block highlighting**
The `// TODO(v1.5):` comment in the fenced block handler (Phase 6.4e) is the exact integration point. The `lang` tag from `CodeBlockKind::Fenced(lang)` is already captured there.

**v1.5 — Config reload (`Ctrl+R`)**
Leave a `// TODO(v1.5): Ctrl+R reloads config` comment at the key-intercept point in the event loop. Config loading is already factored into `config.rs`.

**v2 — Line numbers**
Parse and store `[ui] show_line_numbers = false` (as `false`). Leave a `// TODO(v2): line numbers gutter` comment in `compute_layout`.

**v3 — Table alignment**
Parse and store `[layout] allow_table_overflow = true`. Leave a `// TODO(v3): table column alignment` comment in the table handling block of `build_decoration_map`.

---

## Build Order Summary

| Phase | Deliverable | Depends On |
|---|---|---|
| 0 | Compiling scaffold | — |
| 1 | Terminal opens/closes cleanly | 0 |
| 2 | Theme computes correctly | 0 |
| 3 | Buffer is live-editable | 1, 2 |
| 4 | Layout rectangles correct | 1 |
| 5 | Status bar & info line visible | 2, 4 |
| 6 | Decoration map built correctly | 2 |
| 7 | Full decorated render | 3, 4, 5, 6 |
| 8 | Debounce loop wired | 6, 7 |
| 9 | Save/exit/clipboard work | 3, 5 |
| 10 | Edge cases handled | 7, 9 |
| 11 | Tests & CI | all |
| 12 | README & publishing prep | all |

## Critical Files

- `src/decoration.rs` — `build_decoration_map` and `DecorationMap`; the most complex business logic, drives all Markdown decoration, contains the v1.5 syntect seam
- `src/renderer.rs` — the custom `MarkdownView` widget; span-splitting, selection overlay, and soft-wrap all live here
- `src/main.rs` — terminal lifecycle (critical for `DisableMouseCapture` on every exit path), event loop with debounce, and key-intercept ordering
- `src/config.rs` — `blend()`, color parsing, derived token computation, XDG-aware config loading with graceful error handling
- `src/layout.rs` — `compute_layout` and `EditorLayout`; all rendering depends on correct rectangle computation across terminal sizes
