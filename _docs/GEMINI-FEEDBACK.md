# Yame Codebase Feedback Report

This document provides a comprehensive review of the **Yame** terminal Markdown editor codebase, focused on **Performance Improvements**, **Code Quality / Best Practices**, and **Architectural Recommendations**.

---

## Executive Summary

The **Yame** codebase is an exceptionally clean, well-structured, and idiomatic Rust project. It shows a high degree of maturity, characterized by:
- **Strong separation of concerns**: Clear boundaries between the state engine (`App`), the TUI logic (`input`), the Markdown decoration pipeline (`decoration`), and the terminal widget (`renderer`).
- **Excellent test coverage**: Comprehensive unit and integration tests (~274 tests total) with detailed comments about mutation targets.
- **Safety and warning-free compilation**: The project compiles with zero warnings under strict `cargo clippy -- -D warnings` rules.
- **No-copy where possible**: Rendering layout calculations and word wrapping leverage Rust's lifetime system (`&str` slices pointing to source strings) to minimize copying.

Despite its solid design, several performance bottlenecks scale poorly with document size ($O(N)$ operations on the main thread during typing and viewport rendering). These are detailed below, along with actionable solutions.

---

## 1. Performance Hotspots & Bottlenecks

### 1.1. $O(N)$ Allocation of the Entire Document String on Every Redecorate Tick
* **File:** [input.rs](file:///Users/watcher/githere/yame/src/input.rs#L1371-L1378)
* **Code:**
  ```rust
  let _ = deco_tx.send(DecorateRequest {
      text: app.textarea.lines().join("\n"),
      mode: app.file_mode.clone(),
      ...
  });
  ```
* **Issue:** 
  On every single keystroke (debounced by 50ms) or forced redecoration, the application joins all lines of the `TextArea` into a single, contiguous string. For large files (e.g., several megabytes), this performs a full copy of the text, allocating a massive string, which leads to high memory churn and GC/allocator pressure in the event loop.
* **Suggested Solution:**
  Refactor the decoration pipeline (`build_decoration_map` and related modules) to work directly on line slices (`&[String]`) or an iterator of lines, rather than requiring a flat `&str`. Since `tui-textarea` naturally maintains text as lines, this avoids the need for joining and allocating a single flat buffer.

---

### 1.2. $O(N)$ String Comparison for Dirty-Checking on Every Keystroke
* **File:** [app.rs](file:///Users/watcher/githere/yame/src/app.rs#L314-L320)
* **Code:**
  ```rust
  pub fn recompute_dirty(&mut self) {
      self.is_dirty = match &self.saved_content {
          Some(saved) => self.textarea.lines() != saved.as_slice(),
          None => !self.textarea.lines().is_empty(),
      };
  }
  ```
* **Issue:**
  `mark_keystroke()` is called on every text-mutating keystroke, which invokes `recompute_dirty()`. This performs a full element-by-element string comparison between the current buffer lines and the snapshot taken during the last save. For large documents, comparing thousands of lines on *every keystroke* wastes CPU cycles.
* **Suggested Solution:**
  Instead of comparing all lines:
  1. **Undo-Stack Tracking ($O(1)$):** Track the state using the undo history. Since `tui-textarea` keeps an undo/redo stack, you can record the "clean cursor" (e.g., the position or length of the undo history stack) at the time of the last save. If the current history index matches the saved history index, the buffer is clean; otherwise, it is dirty.
  2. **Dirty Bit / Modification Counting:** Maintain a simple dirty counter/flag. If undo-stack depth is not exposed by `tui-textarea`, you can check if `saved_content` is dirty using an event-based approach or comparing hashes of lines rather than the full strings (though hash collision risk exists, it's very minor).

---

### 1.3. $O(N)$ Scroll Clamping Scan on Viewport Height Clamping
* **File:** [commands.rs](file:///Users/watcher/githere/yame/src/commands.rs#L89-L94)
* **Code:**
  ```rust
  let above_visual: usize = lines
      .get(app.scroll_top..cursor_row.min(lines.len()))
      .unwrap_or(&[])
      .iter()
      .map(|l| renderer::wrap_line(l, cw).len())
      .sum();
  ```
* **Issue:**
  To clamp the viewport correctly when scrolling down, `clamp_scroll` computes the visual (soft-wrapped) line offset above the cursor. It loops through all lines from `scroll_top` to `cursor_row`, calling `wrap_line` on *every logical line*.
  If a user opens a 10,000-line file and jumps to the end (`Ctrl+End`), the scroll logic will synchronously invoke `wrap_line` (which collects characters into vectors and measures display widths) 10,000 times on the main thread, producing a noticeable lag spike.
* **Suggested Solution:**
  - **Wrap-Width Caching:** Cache the wrapped line heights (e.g., store a `Vec<usize>` containing the visual height of each logical line at the current column width). This can be updated incrementally as lines are edited. Then, summing visual heights becomes a simple slice sum operation ($O(1)$ per keypress once built).

---

### 1.4. Inefficient Word Counting on Non-Markdown Files
* **File:** [decoration/words.rs](file:///Users/watcher/githere/yame/src/decoration/words.rs#L4-L11)
* **Code:**
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
* **Issue:**
  In `FileMode::PlainHighlight` and `FileMode::PlainText`, the app calls `count_words()` to display the word count in the status bar. Because `count_words` runs the `pulldown_cmark::Parser` (a full Markdown parser pass) over the entire document, editing a non-Markdown file (like Rust, JSON, or JavaScript) pays a heavy Markdown parsing penalty on every keystroke.
* **Suggested Solution:**
  If the file mode is not `Markdown`, bypass `pulldown_cmark` entirely and use a fast, standard word counter:
  ```rust
  pub fn count_words_plain(text: &str) -> usize {
      text.split_whitespace().count()
  }
  ```

---

### 1.5. Visual Line Wrapping / Splitting Allocations in Render Loop
* **File:** [renderer/mod.rs](file:///Users/watcher/githere/yame/src/renderer/mod.rs#L677), [renderer/utils.rs](file:///Users/watcher/githere/yame/src/renderer/utils.rs#L47)
* **Code:**
  ```rust
  // renderer/mod.rs
  let wrapped = wrap_line_indented(line, ...); // allocates Vec<&str>

  // renderer/utils.rs (inside split_into_spans)
  let chars: Vec<(usize, char)> = line.char_indices().collect(); // allocates Vec
  ```
* **Issue:**
  During the widget render pass, `wrap_line_indented` is called multiple times on the same line (once for normal render, once for selection drawing, once for search highlights, and once for cursor positioning). Furthermore, `split_into_spans` allocates a collected vector of all characters in the line on every single frame for every line. These redundant visual wrapping steps and collected char-indices vectors generate high memory allocations during scrolling.
* **Suggested Solution:**
  - **Avoid `collect()` in `split_into_spans`:** Perform byte-to-char boundary mapping incrementally using iterators, or keep a running byte index tracking instead of collecting the full character vector.
  - **Cache Wrapped Lines:** Pre-calculate wrapping metadata when the document is parsed or resized, so that rendering doesn't recalculate soft-wrapping from scratch.

---

### 1.6. Synchronous Search Match Scanning on Main Thread
* **File:** [search.rs](file:///Users/watcher/githere/yame/src/search.rs#L69-L104)
* **Code:**
  ```rust
  pub fn update_matches(&mut self, lines: &[String]) {
      ...
      self.matches = find_all_matches(self.compiled.as_ref().unwrap(), lines);
  }
  ```
* **Issue:**
  Every keystroke inside the search input bar invokes `update_matches()`. This runs a full-document search using `Regex::find_from_pos` synchronously on the main event thread. On large documents, typing a search query will block the TUI event loop on each character entered, causing typing lag.
* **Suggested Solution:**
  - **Debounce search updates:** Only trigger `update_matches` if the user stops typing for 100–150ms.
  - **Asynchronous Search:** Offload the search match-finding algorithm (`find_all_matches`) to the background thread alongside the decoration worker.

---

### 1.7. Redundant Frontmatter Scans
* **File:** [decoration/builder/mod.rs](file:///Users/watcher/githere/yame/src/decoration/builder/mod.rs#L159), [decoration/highlight.rs](file:///Users/watcher/githere/yame/src/decoration/highlight.rs#L98)
* **Issue:**
  `detect_frontmatter()` is called twice during a single decoration pass: once inside `build_decoration_map` and once inside `apply_highlight_spans`. Both calls scan the text starting from line 0.
* **Suggested Solution:**
  Store the frontmatter detection result in `BuildState` (e.g., `frontmatter_end_line: Option<usize>`) during the initial pass, and read from it in the highlight post-pass, avoiding the redundant second scan.

---

## 2. General Code Quality & Best Practices

### 2.1. Fast Hashers in `HighlightCache`
* **File:** [highlighting.rs](file:///Users/watcher/githere/yame/src/highlighting.rs#L226)
* **Observation:**
  `HighlightCache` uses Rust's `std::collections::HashMap`, which uses the cryptographically secure SIP hasher by default.
* **Recommendation:**
  For local caches like syntax-highlighted code blocks, cryptographic security is not needed. Switching to a fast, non-cryptographic hasher (such as `AHasher` from `ahash` or `FxHasher` from `rustc-hash`) will improve cache lookup times.

### 2.2. Zero-Width Match Character Alignment in Search
* **File:** [search.rs](file:///Users/watcher/githere/yame/src/search.rs#L230)
* **Code:**
  ```rust
  if m.end() == m.start() {
      byte_pos = m.start() + 1; // potential UTF-8 char boundary issue
      continue;
  }
  ```
* **Observation:**
  Advancing `byte_pos` by `+ 1` byte on zero-width match could land on a middle-byte of a UTF-8 character, potentially causing panics or incorrect matches in succeeding lines.
* **Recommendation:**
  Find the next character boundary using `line.floor_char_boundary(m.start() + 1)` or finding the start of the next character programmatically to ensure code correctness for non-ASCII contents.

### 2.3. Reducing `#[mutants::skip]` annotations
* **Observation:**
  The project contains many `#[mutants::skip]` attributes. While well-reasoned in code comments (explaining why mutations are untestable or caused by TUI layout properties), a few of these functions could be refactored into pure functions, allowing tests to catch mutants without skipping them.
* **Recommendation:**
  Consider isolating visual coordinates mapping and bounds checking into pure mathematical structs (`LayoutCoordinates`), testing them in isolation without mocking the entire `ratatui::Buffer` or `App`.

---

## Conclusion

**Yame** is a stellar terminal Markdown editor codebase. Its current bottlenecks are related to visual soft wrapping and dirty-checking operations doing linear scans on the main thread. Implementing wrap height caching and undo-history-based dirty checking will allow Yame to scale seamlessly from editing short notes to large, multi-megabyte Markdown documents and source files.
