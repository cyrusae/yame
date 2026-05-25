# Adversarial Codebase Review (FEEDBACK-1)

This document contains the findings of an adversarial review of the `yame` ("yet another markdown editor") codebase. It covers functional bugs, match priority defects, design spec deviations, performance bottlenecks, visual rendering edge cases, and code quality issues.

---

## 1. Critical Bugs & Functional Defects

### 1.1. match Priority Bug in Exit Prompt State

In [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L310-L382), the key event handler matches specific control sequences at the outer level before checking for modal states like the `ExitPrompt` in the default/catch-all arm:

```rust
Event::Key(k) => {
    match (k.modifiers, k.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => { ... }
        (KeyModifiers::CONTROL, KeyCode::Char('x'))
        | (KeyModifiers::NONE, KeyCode::Esc) => {
            if handle_exit(app) { break; }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => { ... }
        _ => {
            // Handle exit prompt key intercepts
            if matches!(app.status.mode, StatusMode::ExitPrompt) {
                match k.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => { ... }
                    KeyCode::Char('n') | KeyCode::Char('N') => { ... }
                    KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') => {
                        app.status.mode = StatusMode::Normal;
                    }
                    _ => {}
                }
            }
        }
    }
}
```

**Impact:**

* **`Esc` is shadowed:** Pressing `Esc` matches the outer arm `(KeyModifiers::NONE, KeyCode::Esc)` and calls `handle_exit(app)`. Since the editor is dirty, `handle_exit` returns `false` and re-applies `StatusMode::ExitPrompt`. The user cannot escape/cancel the exit prompt using `Esc`.
* **`Ctrl+C` is shadowed:** Pressing `Ctrl+C` is intercepted by the outer copy handler, executing clipboard copy logic rather than canceling the prompt.

> **FIX:** How do we correctly address this?

### 1.2. Coordinate Translation Boundary Flaw (Mouse Clicks)

In [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L97-L127), the `screen_to_doc` helper translates screen clicks to logical document positions:

```rust
fn screen_to_doc(
    screen_row: u16,
    screen_col: u16,
    editor_area: &Rect,
    scroll_top: usize,
    lines: &[String],
) -> Option<(u16, u16)> {
    if screen_row < editor_area.y || screen_col < editor_area.x {
        return None;
    }
    // Missing upper-bound checks for width and height!
    ...
```

**Impact:**

* There are no boundary checks for `screen_col >= editor_area.x + editor_area.width` or `screen_row >= editor_area.y + editor_area.height`.
* Clicking below the editor area (on the info line or the status bar) satisfies the check, passes through the translation, falls through the loop, and triggers:

  ```rust
  // Click landed below all content — go to last line, column 0.
  Some((lines.len().saturating_sub(1) as u16, 0))
  ```

  This causes the cursor to jump to the last line, column 0, whenever the status bar or info line is clicked.

> **FIX:** This seems like a good catch.

---

## 2. Deviations from Design Specification

### 2.1. Completely Missing Scrollbar Widget

The design specification in [_docs/DESIGN.md](file:///Users/watcher/githere/yame/_docs/DESIGN.md#L34) explicitly states:
> *"Scrollbar: Ratatui Scrollbar widget on the right edge of the editing column, styled to match the theme (Unicode track + thumb)."*

There is no scrollbar widget implemented, imported, or rendered anywhere in the codebase.

> **TODO:** This is intended--the scrollbar was removed--edit DESIGN.md to reflect that instead of reinserting it.

### 2.2. Missing Blockquote Soft-Wrap Continuation Indentation

The design specification in [_docs/DESIGN.md](file:///Users/watcher/githere/yame/_docs/DESIGN.md#L92) states:
> *"On soft-wrapped lines, indent continuation text to align with the text start after `>` — do not wrap to column zero"*

Although the parser flags `is_blockquote` on `StyledSpan` in [src/decoration.rs](file:///Users/watcher/githere/yame/src/decoration.rs#L21), the renderer in [src/renderer.rs](file:///Users/watcher/githere/yame/src/renderer.rs#L246-L354) ignores this flag entirely. As a result, soft-wrapped blockquote rows wrap to column zero, breaking the alignment and visual hierarchy.

> **TODO:** This is already on the list to be addressed.

### 2.3. Unimplemented `yame init` command & No-Args Help

The design specification in [_docs/SHELL INTENTIONS.md](file:///Users/watcher/githere/yame/_docs/SHELL%20INTENTIONS.md) outlines implementing `yame init` to auto-generate the fuzzy shell wrapper.

* `parse_args()` in [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L39-L48) only accepts exactly one argument: a file path. It does not support `yame init` or output the shell configuration script.
* Running `yame` without arguments prints a terse usage error and exits with code 1 instead of showing an intro page or suggesting `yame init`.
* If `yame init` is run, it treats `"init"` as a file path and attempts to create or open a file named `init`.

> **TODO:** This is known (not implemented yet), but the "it is impossible to create a file named 'init'" issue is worth noting and considering. Probably fine (why would someone want to do that???) but should be documented at minimum? We can also implement no-args help right now.

---

## 3. Performance & Resource Bottlenecks

### 3.1. Excessive Redecorating on Every Cursor Move

In [src/main.rs](file:///Users/watcher/githere/yame/src/main.rs#L354-L382), the key event handler processes navigation keys (arrow keys, Home, End, PgUp, PgDn) in the default block and calls `app.mark_keystroke()`.
```rust
pub fn mark_keystroke(&mut self) {
    self.last_keystroke = Some(Instant::now());
    self.recompute_dirty();
}
```

**Impact:**

* Moving the cursor sets the 50ms re-decoration timer.
* After 50ms, the event loop joins the entire text buffer and runs the full Markdown parser via `build_decoration_map` and `count_words`, even though no text content changed.
* For larger files, moving the cursor triggers continuous, redundant parsing of the entire document, causing high CPU usage and rendering lag.

> **TODO:** Investigate this--is it a consequence of something necessary or a thing we can improve?

### 3.2. Redundant Double-Parsing of Markdown Content

During every decoration pass, the document is parsed twice:

1. `build_decoration_map` runs `pulldown_cmark::Parser::new_ext` to style the spans.
2. `count_words` runs `pulldown_cmark::Parser::new` to count words.
Both passes run sequentially on the main thread, parsing the exact same text. They should be unified into a single parsing pass.

> **TODO:** Good suggestion, unify this.

### 3.3. Hot-Path Heap Allocations on Every Frame

* **In `split_into_spans`:** The character collection `line.char_indices().collect::<Vec<_>>()` is executed on every line *before* checking if `spans.is_empty()`. Since most lines are undecorated, this results in thousands of unnecessary heap allocations per second during scrolls or typing.
* **In `wrap_line`:** `char_indices` vector allocation happens unconditionally on every frame for every visible row, even if the line fits perfectly within the viewport.

> **TODO:** Can this be addressed? How and what are the tradeoffs? Discuss.

### 3.4. Excessive Clipboard Connection Overhead

In [src/clipboard.rs](file:///Users/watcher/githere/yame/src/clipboard.rs#L69-L80), `arboard::Clipboard::new()` is called on every single copy or paste operation. Establishing a connection to the system clipboard / display server (X11/Wayland/macOS) is expensive and should be initialized lazily and cached.

> **TODO:** Best practices in establishing clipboard connections, test clipboard further.

---

## 4. Visual Rendering & Layout Edge Cases

### 4.1. CJK / Emoji Display and Wrapping Corruption

* **Wrapping:** `wrap_line` in [src/renderer.rs](file:///Users/watcher/githere/yame/src/renderer.rs#L136-L191) wraps text using character count (`char_indices.len()`). Wide CJK characters and emojis occupy 2 visual columns but count as 1 character, which causes wrapped text to exceed the editor column boundary.
* **Overwriting:** `MarkdownView::render` draws cells one char at a time, incrementing `x` by 1. For double-width characters, the next character is drawn at `x + 1`, overwriting the right half of the character. The editor should use `unicode-width` and skip cells accordingly.

### 4.2. Tab Character (`\t`) Column Offsets

The layout engine treats `\t` as width 1, but terminals render tabs as 4 or 8 columns. Writing a raw tab using `set_char` does not expand it to spaces in Ratatui's buffer, causing cursor alignment offsets and selection layout corruption.

### 4.3. POSIX Compliance: Empty File Save Growth

If `yame` opens a 0-byte file, `load_file` parses it as `[""]`. Upon saving, `lines.join("\n") + "\n"` is written, yielding `"\n"`. This converts a 0-byte empty file into a 1-byte file containing a single newline.

---

## 5. Code Quality & Technical Debt

### 5.1. Bypassing Tests with `#[mutants::skip]`

Many testable portions of the codebase are skipped using `#[mutants::skip]` to avoid cargo-mutants coverage instead of being unit-tested using mocks or custom inputs:

* `App::new` in `app.rs` is skipped because it handles I/O.
* Status bar and info line rendering are skipped instead of feeding a mock buffer and asserting output cells.
* Clipboard operations are skipped entirely, leaving clipboard coordination untested.

> **TODO:** I'm pretty sure most of these are required/not mutant-able? Address one by one whether any are improperly skipped.

### 5.2. State Mutation inside `terminal.draw`

In `main.rs`, state mutation (clamping scroll positions and updating `app.scroll_top`) is executed inside the rendering closure:

```rust
terminal.draw(|f| {
    ...
    if cursor_row < app.scroll_top {
        app.scroll_top = cursor_row;
    }
    ...
})
```

This violates clean separation of concerns and renders the drawing function impure. Layout and scroll boundaries should be computed during event handling.

> **TODO:** Discuss this--is it a side effect of intended behavior or something that can be improved? Explain it to me.

---

## 6. Suggested Integration Tests

The following integration tests should be added to the test suite to guard against regressions:

1. **Modal Exit Prompt Cancellation Test**
   * **Scenario:** Mock a dirty buffer, trigger `Ctrl+X` to enter the exit prompt, send `Esc` or `C`, and assert that the status mode returns to `Normal` and the editor remains open.
2. **Cursor Navigation Inertness Test**
   * **Scenario:** Mock navigation inputs (`Up`, `Down`, `Left`, `Right`, `Home`, `End`) on a clean buffer. Assert that `app.is_dirty` remains `false` and `app.last_keystroke` is `None` (preventing redundant decoration passes).
3. **Empty File Integrity Test**
   * **Scenario:** Open a non-existent file path (representing a 0-byte buffer), save it, and verify that the saved file size is exactly 0 bytes (or does not grow to 1 byte on blank buffer save).
4. **Mouse Translation Boundary Test**
   * **Scenario:** Trigger a click event outside the editor column bounds (e.g., column width + margins, or y-level of status bar) and assert that `screen_to_doc` returns `None`.
5. **Wrapped Blockquote Indentation Test**
   * **Scenario:** Construct a long blockquote line that soft-wraps across 3 lines. Render it and assert that visual lines 2 and 3 are indented to match the first line text offset.
6. **Wide Character/Emoji Wrapping Test**
   * **Scenario:** Provide a line composed of Japanese characters/emojis. Verify that the line wrapping splits the line at the correct column boundary, and that rendering does not overwrite characters.
7. **Tab character Expansion Test**
   * **Scenario:** Render a line containing tabs (`\t`) and verify that the layout and cursor jump positions are correctly offset by the expanded visual width in columns.

> **TODO:** Open an issue for integration test planning and make a deliberate plan, keeping these in mind. What's our integration test framework landscape look like?
