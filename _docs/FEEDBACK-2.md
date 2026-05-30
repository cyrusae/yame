# Adversarial Codebase Review (FEEDBACK-2)

This document contains findings from a second adversarial pass over the `yame` ("yet another markdown editor") codebase as part of prep work for the `v0.1.0` release. It highlights hidden bugs, visual glitches, performance bottlenecks, and architectural shortcuts that could be sharpened, polished, or otherwise improved.

---

## 1. Functional & Visual Defects

### 1.1. Table Header Decoration Swallows Inline Formatting
* **Location:** [src/decoration/mod.rs:L964-980](file:///Users/watcher/githere/yame/src/decoration/mod.rs#L964-L980) in [build_decoration_map](file:///Users/watcher/githere/yame/src/decoration/mod.rs#L218)
* **Human Reader Explanation:**
  Markdown table headers often contain formatted text like `**bold text**`, `*italics*`, `[links](url)`, or `` `inline code` ``. Currently, if any inline styling is used inside a table header, the editor completely ignores it and renders it as plain text. This happens because the table header decorator matches the `Tag::TableHead` event and applies a single wide style span covering the entire header line. When the rendering logic processes the spans, this wide span collides with and clips (swallows) all nested inline spans, violating standard Markdown rendering expectations.
* **AI Coding Agent Fix Note:**
  Do not call `add_byte_range_span` with a single wide span for `Tag::TableHead`. Instead, style table header text gaps by using `emit_content_around_existing` (like bold/italic do) or by setting the default style of the row in the renderer, similar to how blockquote text styling was refactored (applying the header style only to undecorated parts of the table head).

### 1.2. Left-Heavy Centering on Dismissible Messages
* **Location:** [src/renderer/status.rs:L65-77](file:///Users/watcher/githere/yame/src/renderer/status.rs#L65-L77) in [render_status_bar](file:///Users/watcher/githere/yame/src/renderer/status.rs#L48)
* **Human Reader Explanation:**
  When a warning or error banner is shown in the status bar (e.g. config error alerts or italics fallback warnings), the text is intended to be centered. However, because it only inserts space padding on the *left* of the message, the text appears visually off-center. Worse, because the text span has a different background color (`hints_bg`) from the rest of the status bar (`canvas_bg`), the highlighted warning block cuts off abruptly immediately after the message instead of extending symmetrically on the right or spanning the entire width, producing a jarring visual layout.
* **AI Coding Agent Fix Note:**
  Modify the format string in [src/renderer/status.rs:L72](file:///Users/watcher/githere/yame/src/renderer/status.rs#L72) to pad both the left and right sides of the message:
  ```rust
  let padded = format!("{:pad$}{msg}{:pad$}", "", msg, "", pad = pad as usize);
  ```
  Alternatively, configure the widget style so the entire status bar row draws with `hints_bg` background whenever a dismissible warning is active.

### 1.3. Raw Tab Input Breaks Visual Grid Alignment
* **Location:** [src/input.rs:L288-312](file:///Users/watcher/githere/yame/src/input.rs#L288-L312) in [handle_key_event](file:///Users/watcher/githere/yame/src/input.rs#L199)
* **Human Reader Explanation:**
  Although `yame` correctly expands tab characters (`\t`) to spaces when loading files from disk, it does not intercept tab keypresses during active editing. Pressing the `Tab` key inserts a raw `\t` character into the text buffer. Because the visual layout engine assumes control characters have a visual width of 1 column, but terminals display tabs as 4 or 8 columns, the cursor positioning, selection highlighting, and soft-wrap calculations immediately drift out of sync, leading to severe visual corruption.
* **AI Coding Agent Fix Note:**
  In the editing fallback of [handle_key_event](file:///Users/watcher/githere/yame/src/input.rs#L199), intercept `KeyCode::Tab` (when `modifiers` matches `KeyModifiers::NONE` or `KeyModifiers::SHIFT`) and insert the appropriate number of spaces to align the cursor to the next tab stop (using `tab_width`) rather than passing the raw key to `app.textarea.input(k)`.

### 1.4. Exit Prompt Intercepts Control Key Shortcuts
* **Location:** [src/input.rs:L205-220](file:///Users/watcher/githere/yame/src/input.rs#L205-L220) in [handle_key_event](file:///Users/watcher/githere/yame/src/input.rs#L199)
* **Human Reader Explanation:**
  When the editor prompts the user to save changes before exiting ("Save modified buffer? [Y]es [N]o [C]ancel"), it accepts `y`/`Y` and `n`/`N` inputs. However, it does not check if any modifier keys are held. If a user presses `Ctrl+Y` (intending to Redo) or `Ctrl+N` while this prompt is open, the prompt intercepts the keystroke as a bare `y` or `n` and triggers a destructive action (saving and exiting, or exiting and discarding changes) without confirming the user's intent.
* **AI Coding Agent Fix Note:**
  Refactor the exit prompt match block to enforce that modifiers are empty or only contain shift:
  ```rust
  if matches!(app.status.mode, StatusMode::ExitPrompt) {
      if k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) {
          return KeyOutcome::Continue;
      }
      return match k.code { ... }
  }
  ```

---

## 2. Performance & Resource Bottlenecks

### 2.1. Constant Heap Allocations in Render Hot Path (`shorten_path`)
* **Location:** [src/renderer/status.rs:L34](file:///Users/watcher/githere/yame/src/renderer/status.rs#L34) in [pill1_parts](file:///Users/watcher/githere/yame/src/renderer/status.rs#L26)
* **Human Reader Explanation:**
  The status bar updates on every single frame draw (roughly 60 times per second). Part of this rendering calls `shorten_path` to display the file name and its parent directories. `shorten_path` splits the file path and allocates a fresh vector of strings on the heap every single time it runs. This creates continuous memory churn and garbage collection pressure in a hot rendering loop, even though the file path remains completely static during editing.
* **AI Coding Agent Fix Note:**
  Cache the shortened file path as a pre-computed `String` field (e.g. `app.shortened_path`) on the [App](file:///Users/watcher/githere/yame/src/app.rs#L13) struct. Compute this value only inside `App::new` and update it if the file path is modified (such as on a save or save-as action), and reference this cached string directly in `pill1_parts`.

### 2.2. Redundant Dirty Buffer Comparisons on Every Navigation Key
* **Locations:** [src/input.rs:L305](file:///Users/watcher/githere/yame/src/input.rs#L305) and [src/input.rs:L433](file:///Users/watcher/githere/yame/src/input.rs#L433) in [handle_key_event](file:///Users/watcher/githere/yame/src/input.rs#L199)
* **Human Reader Explanation:**
  Whenever the cursor moves or the viewport scrolls (using arrow keys, PageUp/Down, Home, End, or mouse dragging), the editor recomputes whether the document is dirty. To do this, it compares the active buffer line-by-line against a saved snapshot. For files with thousands of lines, moving the cursor triggers thousands of string comparisons on every single keystroke. Because navigation cannot mutate the document, this full comparison is completely redundant and causes high CPU usage and noticeable input lag on larger files.
* **AI Coding Agent Fix Note:**
  Remove `app.recompute_dirty()` from the navigation-only event handling flows (such as visual-cursor movements and arrow key keypresses). Only recompute the dirty flag when a mutating edit occurs (typing, pasting, undoing, redoing, or selection wrapping).

### 2.3. Uncapped Syntax Highlighting Cache Memory Leak
* **Location:** [src/highlighting.rs:L199](file:///Users/watcher/githere/yame/src/highlighting.rs#L199) in [HighlightCache](file:///Users/watcher/githere/yame/src/highlighting.rs#L180)
* **Human Reader Explanation:**
  To speed up syntax highlighting of fenced code blocks, `yame` caches highlighted blocks in a hash map. However, this cache is uncapped and grows indefinitely during the editor session. If a user edits a large document or keeps the editor open for a long time, the cache holds on to highlighted spans of deleted or modified code blocks forever, creating a slow memory leak.
* **AI Coding Agent Fix Note:**
  Replace the raw `HashMap` in `HighlightCache` with a bounded cache (such as an LRU cache or a map with a maximum size limit) to evict stale highlights, or clear the cache when reloading configurations.

> **TODO:** Would like a second opinion on this but it sounds legit?

### 2.4. Infinite Clipboard Reconnection Retries on Headless Systems
* **Location:** [src/clipboard.rs:L47-51](file:///Users/watcher/githere/yame/src/clipboard.rs#L47-L51) in [ensure_clipboard](file:///Users/watcher/githere/yame/src/clipboard.rs#L47)
* **Human Reader Explanation:**
  If `yame` is run on a system without a running display server or clipboard daemon (like a headless Linux server or a remote Docker container), `arboard` cannot connect to a system clipboard and returns an error. Currently, the editor retries connecting on *every* copy or paste attempt. Because establishing a connection to a display server clipboard is a blocking, slow network operation, the editor will briefly freeze or stutter every time the user uses clipboard shortcuts in these environments.
* **AI Coding Agent Fix Note:**
  Replace `app.clipboard: Option<Clipboard>` with a three-state enum representation:
  ```rust
  pub enum ClipboardState {
      Uninitialized,
      Ready(arboard::Clipboard),
      Unavailable,
  }
  ```
  If a connection attempt fails once, set the state to `Unavailable` so subsequent copy/paste operations bypass the initialization and fail immediately without blocking the event loop.

---

## 3. Code Quality & Architectural Debt

### 3.1. Duplicated Text Selection Logic in `input.rs` and `clipboard.rs`
* **Locations:** [src/input.rs:L133-159](file:///Users/watcher/githere/yame/src/input.rs#L133-L159) ([get_selection_text](file:///Users/watcher/githere/yame/src/input.rs#L133)) and [src/clipboard.rs:L54-87](file:///Users/watcher/githere/yame/src/clipboard.rs#L54-L87) ([get_copy_text](file:///Users/watcher/githere/yame/src/clipboard.rs#L54))
* **Human Reader Explanation:**
  The logic for extracting highlighted text from the document (which maps single-line vs multi-line selections, collects character indices, and maps margins) is duplicated almost identically in two separate places. This violates the DRY (Don't Repeat Yourself) principle, increasing code maintenance costs and introducing potential bugs where changes to selection handling in one file are not propagated to the other.
* **AI Coding Agent Fix Note:**
  Refactor `get_copy_text` to reuse `get_selection_text` directly (returning the selected text if present, and falling back to the current line if not), and relocate this logic to a single shared module or method on `App` or `TextArea`.

### 3.2. Code Highlight Settings Ignored on Configuration Reload
* **Location:** [src/input.rs:L584-597](file:///Users/watcher/githere/yame/src/input.rs#L584-L597) in [handle_key_event](file:///Users/watcher/githere/yame/src/input.rs#L199)
* **Human Reader Explanation:**
  Pressing `Ctrl+R` reloads settings from the configuration file. However, this command fails to recreate the syntax highlighting cache or apply changes to highlighting preferences (like switching theme settings, enabling/disabling highlighting, or shifting palette options). Code blocks also continue to render using the old theme colors because the cache is not cleared.
* **AI Coding Agent Fix Note:**
  In `ReloadConfig`, reconstruct `app.highlight_cache` with the newly loaded settings (reflecting any toggled flags or syntax theme updates) and clear the existing memoized cache so that all code blocks are re-styled correctly.

### 3.3. POSIX Compliance: Inconsistent Empty File Saving
* **Location:** [src/commands.rs:L11-37](file:///Users/watcher/githere/yame/src/commands.rs#L11-L37) in [handle_save](file:///Users/watcher/githere/yame/src/commands.rs#L11)
* **Human Reader Explanation:**
  To prevent empty buffers from writing a single trailing newline `\n` to disk, `yame` tracks whether the file was empty on load via `initial_file_empty`. However, this tracking is never cleared. If a user opens an empty file, types text, and saves it, it is saved properly. But if they delete all text and save again, the file is saved as 0 bytes. Conversely, if they open a non-empty file, clear it completely, and save, it writes `\n` (1 byte). This inconsistent behavior violates the expectation that clearing an editor buffer always yields a standard 0-byte file.
* **AI Coding Agent Fix Note:**
  Simplify `handle_save` in [src/commands.rs](file:///Users/watcher/githere/yame/src/commands.rs): if the active buffer is completely empty (`lines == [""]`), always write `String::new()` (0 bytes) to disk, removing the need for the convoluted `initial_file_empty` state variable.

### 3.4. Over-broad `#[mutants::skip]` on Clipboard and I/O Functions
* **Location:** [src/clipboard.rs:L54-87](file:///Users/watcher/githere/yame/src/clipboard.rs#L54-L87) ([get_copy_text](file:///Users/watcher/githere/yame/src/clipboard.rs#L54)) and [src/commands.rs:L11-37](file:///Users/watcher/githere/yame/src/commands.rs#L11-L37) ([handle_save](file:///Users/watcher/githere/yame/src/commands.rs#L11))
* **Human Reader Explanation:**
  Several functions are skipped in mutation tests using `#[mutants::skip]` under the assumption that they cannot be tested due to I/O dependencies. For example, `get_copy_text` is skipped because it "Accesses live textarea state... not unit-testable in isolation". In reality, the `App` state is completely testable in memory, and this function's selection-extraction code is identical to `get_selection_text` in `src/input.rs` which is fully tested. This skip creates a blind spot in the codebase's test coverage.
* **AI Coding Agent Fix Note:**
  Remove the `#[mutants::skip]` annotations from pure functions (or those that only depend on internal data structures like `App` or `TextArea`), and write direct unit tests to cover them.

> **TODO:** Review all skipped mutants to check against this feedback.
