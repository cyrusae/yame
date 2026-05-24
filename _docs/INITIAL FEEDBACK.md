# Adversarial Codebase Review & Initial Feedback

This document contains a strict, adversarial review of the `yame` codebase. It identifies functional defects, deviations from the design specification, ignored edge cases, performance bottlenecks, and architectural shortcuts.

---

## 1. Executive Summary

While the codebase compiles and has a solid base of unit tests, it contains several critical architectural shortcuts, missing design features, and edge-case bugs that would degrade usability or silently corrupt data. 

Specifically:
- **Data Integrity**: Saving files destroys standard POSIX trailing newlines and strips empty trailing lines.
- **Broken Input & Navigation**: Arrow-key navigation marks files as dirty. Undo/redo dirty checking is broken upon file load. Mouse clicks place the cursor at incorrect columns due to screen-space vs. widget-space coordinate misalignment.
- **Unimplemented Specs**: Blockquote soft-wrap indentation is completely absent in the renderer despite being part of the design and defined in the decoration types.
- **Edge Cases**: CJK characters, emojis, and tab characters will corrupt the visual layout and text wrapping.
- **Performance**: The editor parses files twice on every debounce keystroke and executes excessive heap allocations on hot rendering paths (every visual row on every frame).

---

## 2. Critical Bugs & Functional Defects

### 2.1. Silent File Formatting Corruption (POSIX Newline Stripping)
In [src/app.rs](file:///Users/watcher/githere/yame/src/app.rs#L154-L162), `load_file` parses the file content using `str::lines()`:
```rust
let content = std::fs::read_to_string(path)?;
let lines: Vec<String> = content.lines().map(String::from).collect();
```
`str::lines` implicitly strips trailing line-ending characters (`\r\n` or `\n`). If a file ends with a trailing newline (the POSIX standard), or contains multiple empty trailing lines, they are permanently lost upon load. 

Then, in [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L228-L230), the file is saved by joining the lines:
```rust
let content = app.textarea.lines().join("\n");
std::fs::write(&app.file_path, &content)
```
Because `join("\n")` only places newlines *between* lines, the resulting file will never end with a trailing newline. Opening and saving a file in `yame` will silently corrupt its format, causing git diff pollution and violating POSIX standards.

### 2.2. Screen-Space vs. Widget-Space Mouse Coordinate Misalignment
In [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L212-L214), mouse events are piped raw to `tui-textarea`:
```rust
Event::Mouse(mouse) => {
    app.textarea.input(Event::Mouse(mouse));
}
```
However, the editor column is rendered with a dynamic horizontal margin:
```rust
let margin = area.width.saturating_sub(col_width) / 2;
```
`tui-textarea` expects mouse coordinates relative to its own `(0,0)` origin. Because the raw, screen-absolute coordinates are passed, **click-to-place-cursor is completely broken**. 
- If a terminal is 100 columns wide, the editing margin is 20. Clicking at the very start of the text (column 20) sends `column = 20` to `tui-textarea`, which places the cursor 20 characters into the line.
- The same vertical misalignment occurs if any config warning banners shift the vertical offset of the editor.

### 2.3. Cursor Navigation Keypresses Mark the Buffer as Dirty
In [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L202-L208), the event loop catches all key inputs that do not match shortcuts:
```rust
} else {
    // Dismiss any dismissible message on any keypress
    app.status.dismiss();
    app.config_warnings.clear();
    app.textarea.input(k);
    app.mark_keystroke();
}
```
This catch-all executes `app.mark_keystroke()` for navigation keys (arrows, PgUp/PgDn, Home, End).
```rust
pub fn mark_keystroke(&mut self) {
    self.last_keystroke = Some(Instant::now());
    self.is_dirty = true;
}
```
As a result, simply moving the cursor around an unmodified file sets `is_dirty = true`, triggering the exit prompt when the user attempts to close the editor.

### 2.4. Broken Undo/Redo Dirty Checking
`App::saved_content` is initialized to `None` in `App::new`. It is only set to `Some` during a save operation.
In [src/app.rs](file:///Users/watcher/githere/yame/src/app.rs#L62-L67), `recompute_dirty` resolves dirty status via:
```rust
pub fn recompute_dirty(&mut self) {
    self.is_dirty = match &self.saved_content {
        Some(saved) => self.textarea.lines() != saved.as_slice(),
        None => !self.textarea.lines().is_empty(),
    };
}
```
If a user opens an existing file, makes a modification (which marks the buffer dirty), and then presses `Ctrl+Z` to undo the change, the editor compares the content to `None` and falls back to checking if the lines are non-empty. Since the file has text, `is_dirty` remains `true`. The editor remains permanently "dirty" after undoing back to the original loaded state.

---

## 3. Missing Design Specifications

### 3.1. Blockquote Soft-Wrap Continuation Indentation
The design document states:
> *"On soft-wrapped lines, indent continuation text to align with the text start after `>` — do not wrap to column zero"*

In [src/decoration.rs](file:///Users/watcher/githere/yame/src/decoration.rs#L21), `StyledSpan` defines an `is_blockquote: bool` flag which is set correctly on parse. However, in [src/renderer.rs](file:///Users/watcher/githere/yame/src/renderer.rs#L217-L325), `MarkdownView::render` ignores the `is_blockquote` flag entirely. 
Soft-wrapped blockquote text wraps directly to the left margin (column 0), making multi-line blockquotes visually indistinguishable from normal paragraphs.

---

## 4. Ignored Edge Cases & Potential Layout Failures

### 4.1. CJK / Emoji / Wide-Character Wrapping and Cell Corruption
In [src/renderer.rs](file:///Users/watcher/githere/yame/src/renderer.rs#L136-L191), `wrap_line` calculates text wrapping by counting Unicode scalar values (characters):
```rust
let char_indices: Vec<(usize, char)> = s.char_indices().collect();
let total_chars = char_indices.len();
```
CJK characters, emojis, and combining marks occupy 2 visual terminal columns (or 0 columns), meaning the actual screen width of a line containing wide characters will exceed the wrapped `column_width`. 

Furthermore, `MarkdownView::render` draws characters cell-by-cell in a loop:
```rust
for span in &segments {
    for ch in span.content.chars() {
        if (x.saturating_sub(area.x)) as usize >= width {
            break;
        }
        buf[(x, y)].set_char(ch).set_style(span.style);
        x += 1;
    }
}
```
This character iteration assumes every character is exactly `1` column wide.
- Writing a 2-column CJK character or emoji using `buf[(x, y)].set_char(ch)` will result in visual corruption or overlap because the next character is drawn at `x + 1`, overwriting the right half of the double-width character.
- The editor should use `ratatui::buffer::Buffer::set_string` or `set_span` which natively respect visual cell width, and leverage the `unicode-width` crate for wrapping calculations.

### 4.2. Tab Character (`\t`) Column Offsets
The parser and layout engine count `\t` as a single character with width `1`. However, terminals render tabs as 4 or 8 columns. This mismatch will cause cursor alignment offsets, broken selection overlays, and incorrect soft wrapping.

---

## 5. Performance & Resource Bottlenecks

### 5.1. Double Parsing of Markdown Content
On every keystroke debounce tick, the editor joins all buffer lines into a single string and parses it twice:
1. `build_decoration_map` runs `pulldown_cmark::Parser::new_ext`
2. `count_words` runs `pulldown_cmark::Parser::new`

This is completely redundant. The word count could easily be computed during the single decoration pass, avoiding double-parsing overhead.

### 5.2. Hot-Path Heap Allocations on Every Frame
In [src/renderer.rs](file:///Users/watcher/githere/yame/src/renderer.rs#L54-L65), `split_into_spans` allocates a new `Vec` to collect character indices:
```rust
pub fn split_into_spans(
    line: &str,
    spans: &[StyledSpan],
    default_style: Style,
) -> Vec<Span<'static>> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let char_count = chars.len();

    // Fast path — no decoration spans
    if spans.is_empty() {
        return vec![Span::styled(line.to_owned(), default_style)];
    }
```
Crucially, the heap allocation of the `chars` vector occurs **before** the check for empty spans. Because the vast majority of lines in a Markdown file are undecorated (especially when editing, since the active cursor line has all decorations removed), this causes thousands of useless heap allocations per second during scrolls or typing.

Similarly, `wrap_line` performs heap allocations:
```rust
let char_indices: Vec<(usize, char)> = s.char_indices().collect();
```
This is run on every frame for every visible row, even if the text fits within the line width.

### 5.3. Inefficient Substring Counting
In `MarkdownView::render`, visual rows calculate their starting character offset using:
```rust
let byte_off = (row_str.as_ptr() as usize).wrapping_sub(line.as_ptr() as usize);
let char_start = line[..byte_off].chars().count();
```
This is an $O(N)$ operation. When rendering multiple wrapped lines, this counts characters from the start of the logical line repeatedly, yielding $O(N^2)$ behavior relative to line length. It should instead track the character index incrementally as it processes wrapped chunks.

### 5.4. Excessive Clipboard Connection Overhead
In [src/clipboard.rs](file:///Users/watcher/githere/yame/src/clipboard.rs#L69-L73), every single copy or paste action instantiates a new clipboard instance:
```rust
arboard::Clipboard::new()
```
Creating a connection to the OS pasteboard or display server (X11/Wayland/macOS Pasteboard) has significant overhead and can block execution. A single connection should be cached or instantiated lazily and reused.

---

## 6. Code Quality & Technical Debt

### 6.1. Overuse of `#[mutants::skip]` to Bypass Unit Testing
The macro `#[mutants::skip]` is used extensively across the code. While some skip attributes are justified (e.g. panic hooks, CLI entry points), others appear to have been used to bypass testing logic:
- `App::new` in `app.rs` is skipped because it calls `load_file`.
- `handle_copy` and `handle_paste` are skipped completely, preventing unit-testing of clipboard orchestration.
- `render_status_bar` and `render_info_line` are skipped entirely. The rendering logic of the status bar could be tested by passing a mock buffer and inspecting the output cells.

### 6.2. Mutating State inside `terminal.draw`
In `main.rs`, state mutation (clamping scroll positions) is performed inside the rendering closure:
```rust
terminal.draw(|f| {
    ...
    if cursor_row < app.scroll_top {
        app.scroll_top = cursor_row;
    }
    ...
})
```
Modifying application state during a draw call violates clean UI architectures. Scroll tracking and layout limits should be calculated during event handling, keeping the render step pure.

---

## 7. Future Considerations & Enhancements

- **Parent Directory Creation**: If saving a new file in a non-existent directory (e.g. `yame docs/new.md`), the save operation will fail because parent directories are not automatically created.
- **Large Files**: Since the editor loads the entire file into memory and joins it into a single string for parsing on every debounced keystroke, files larger than a few megabytes will trigger major input lag or out-of-memory issues.
- **Visual Wrapped Indicators**: Without line gutters or wrap indicators (such as `↵`), it is difficult for a user to distinguish between a soft-wrapped line and two separate lines.
- **Search Support**: There is currently no searching (`Ctrl+F`) mechanism.
