Ah, *yame*! That makes perfect sense. Terminal editors are notorious for this because keyboard input parsing, terminal mode toggling, and redrawing the screen are inherently sticky to test.

Since you have an agent actively refactoring or writing tests for *yame*, this is the perfect moment to move past simple line coverage and make it build a bulletproof harness.

Here are the specific, high-leverage areas you should grill the agent about covering to eliminate those `MISSED` and `TIMEOUT` clusters for good:

---

## 1. The "Deadlock" Audit (Fixing the TIMEOUTs)

Because you saw a `TIMEOUT` when a match guard was manipulated, the agent needs to explicitly audit how inputs interact with the editor's main loop and state.

* **The Grill:** *"Look at where `TIMEOUT` occurred on line 419. If a key event is received but a condition isn't met, are we accidentally dropping the key without advancing the cursor or state, causing an infinite loop? Write tests that simulate unhandled keys to ensure they break or return an explicit `Unhandled` state instead of spinning."*
* **The Architecture:** Ensure the input loop uses a strict step-by-step frame processing architecture:

---

## 2. Table-Driven Key Combination Matrices

Instead of writing a separate test for every single key (which agents love to do because it pads their metrics), force it to write a **parameterized matrix test** for `src/input.rs`.

* **The Grill:** *"Create a table-driven test suite that feeds a matrix of key inputs into `handle_key_event` and `handle_search_key`. The matrix must include normal characters, control modifiers (`Ctrl+S`, `Ctrl+C`), navigation (`Arrows`, `Home`, `End`), and edge cases like holding down multiple modifiers (`Ctrl+Alt+S`). Ensure every single variant asserts a specific, mutated change in the application state."*

---

## 3. The "Read-Only" Mode Inversion

You had a missed mutant on line 401: `replace match guard !app.read_only with true`. This means the agent isn't testing what happens when *yame* is in a protected state.

* **The Grill:** *"We are completely blind to unauthorized edits. Write dual-state tests. For every input action that modifies text (typing characters, backspace, deleting a line), execute it twice in the test: once when `app.read_only == false` (asserting the text changed) and once when `app.read_only == true` (asserting the text remained completely untouched)."*

---

## 4. Backspace and Boundary Conditions (The Classic TUI Traps)

While the agent is in `src/input.rs`, make it cover the edge cases that human developers usually break when they refactor editor logic later.

* **The Grill:** *"Generate explicit property or boundary tests for the following text manipulation states:"*
* **The Empty File:** Pressing backspace or delete when the buffer is entirely empty.
* **Line Merging:** Pressing backspace at index `0` of a line (does it cleanly merge with the previous line without tearing the buffer?).
* **The Invisible Margin:** Moving the cursor past the end of a line or past the bottom of the file buffer.



---

## 5. Clipboard and External Ecosystem Inversions

If *yame* supports cutting, copying, or popping open system clipboard integrations, `cargo mutants` can wreak havoc by replacing strings with empty slices or stubbing out the clipboard system.

* **The Grill:** *"If you are using system clipboard traits, ensure we have a mock clipboard injector in tests so we can assert that copy/paste loops aren't silently failing or consuming events without acting on them."*

---

### How to prompt the agent right now:

You can literally hand it a prompt like this:

> "In *yame*, `src/input.rs` has significant blind spots regarding keyboard modifiers, match guards, and state traps (causing timeouts). Do not write simple unit tests. Implement a table-driven integration matrix that injects simulated `KeyEvent` structures into `handle_search_key` and `handle_key_event`. Explicitly test the `read_only` guard inversion, ensure no unhandled keys cause infinite loops, and assert distinct state changes for `Ctrl` vs `Super` vs plain character inputs."
