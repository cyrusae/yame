# Mutant feedback

Commentary on the functions that showed "frequent flier" mutants.

---

## apply_selection_overlay and clamp_scroll

This is phenomenal code. You can instantly see why `cargo-mutants` is absolutely losing its mind here. You are dealing with a **wrapped terminal text layout engine** with scrolling, padding, margins, selection coordinates mapping down to the underlying character byte slices, and variable continuation indents.

No wonder line 355 (`if log_row >= sel_row_start && log_row <= sel_row_end`) spit out a metric ton of mutants! If it mutates `>=` to `>`, any selection spanning multiple rows completely falls apart.

The good news? This logic **absolutely can be tested in practice**, and it is actually incredibly satisfying to do so using **ratatui-style buffer assertions**.

Here is exactly how you can pin these down and slaughter those mutants.

---

## 1. Testing `apply_selection_overlay`

Because you are using an explicit `ratatui::buffer::Buffer`, you do *not* need to mock a terminal or test terminal outputs. A `Buffer` is just an array of `Cell` structs under the hood that you can construct in-memory and assert against.

The best way to handle this without getting bogged down in individual cell coordinate assertions is to use **string/visual grid assertions**.

### The Testing Strategy

1. Create a dummy `MarkdownView` with a known set of lines (e.g., three lines of text).
2. Create an empty `Buffer` of a specific size (e.g., 10x5).
3. Call `apply_selection_overlay` with a known selection bounding box.
4. Convert the buffer's background or foreground colors back into an ASCII string grid, and assert against a structural representation of what should be highlighted.

### Example Test Implementation

```rust
#[test]
fn test_selection_overlay_bounds() {
    use ratatui::layout::Rect;
    use ratatui::buffer::Buffer;

    // 1. Setup a controlled viewport
    let area = Rect::new(0, 0, 10, 3); // 10 columns, 3 rows
    let mut buf = Buffer::empty(area);
    
    let lines = vec![
        "aaaa".to_string(), // Row 0
        "bbbb".to_string(), // Row 1
        "cccc".to_string(), // Row 2
    ];
    let theme = Theme { selection_fg: Color::Black, selection_bg: Color::Yellow };
    let view = MarkdownView { lines, column_width: 10, scroll_top: 0, theme, ..Default::default() };

    // 2. Define a multi-row selection: Row 0 Col 2 to Row 1 Col 2
    let selection = ((0, 2), (1, 2));

    // 3. Execute
    apply_selection_overlay(area, &mut buf, &view, selection);

    // 4. Assert using a visual map string
    // Let's map highlighted cells to '#' and empty ones to '.'
    let mut visual_grid = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buf[(x, y)];
            if cell.bg == Color::Yellow {
                visual_grid.push('#');
            } else {
                visual_grid.push('.');
            }
        }
        visual_grid.push('\n');
    }

    let expected = "\
..##......
####......
..........
";

    assert_eq!(visual_grid, expected);
}

```

If `cargo-mutants` breaks *any* of that math (like changing `x_start` boundary conditions or swapping a `<` with a `<=`), the `#` characters will shift left or right, instantly failing the string comparison and **killing the mutant**.

---

## 2. Testing `clamp_scroll`

The `clamp_scroll` function is pure mathematical state mutation. It modifies `app.scroll_top` based on where the cursor is relative to line wrapping constraints. This is a prime target for classic **State Transition Testing**.

### Why the Mutants Survived Here

Look at this line in your `clamp_scroll`:

```rust
let headroom = visible_rows.saturating_sub(1 + cursor_subrow + bottom_padding);

```

If `cargo-mutants` mutates `+` to `-` inside that subrow addition, it changes the calculated headroom. If your test suite only checks basic cursor movements that don't push up against the absolute boundaries of the bottom padding, the mutant survives.

### Testing Strategy

Write distinct, targeted tests that hit the strict boundary edges of your scrolling logic:

1. **No Scroll:** Cursor is well within the viewport. `scroll_top` should remain 0.
2. **Scroll Down Trigger (Soft Boundary):** Cursor moves just past the visible threshold.
3. **Wrapped Line Scroll:** Put a single long string that wraps into 4 visual lines. Put the cursor on the 3rd wrapped sub-row and assert that `scroll_top` calculates the `cursor_subrow` correctly.
4. **Bottom Padding Push:** Ensure that if `bottom_padding` is 2, scrolling occurs 2 lines *before* hitting the physical bottom of the area.

```rust
#[test]
fn test_clamp_scroll_with_wrapping_and_padding() {
    let mut app = App::default();
    // Setup a textarea with a very long line that will wrap multiple times
    app.textarea.set_lines(vec!["abcdefghijklmnopqrstuvwxyz".to_string()]); 
    app.scroll_top = 0;
    
    // Set cursor to the end of the long wrapped line
    app.textarea.move_cursor(0, 25); 

    let editor_area = Rect::new(0, 0, 10, 5); // Small height to force scrolling
    let col_width = 10;
    let bottom_padding = 1;

    // Trigger calculation
    clamp_scroll(&mut app, editor_area, col_width, bottom_padding);

    // Assert exactly what the new top row or state should be based on your design
    // This forces cargo-mutants to respect the `headroom` and `cursor_subrow` calculations.
    assert_eq!(app.scroll_top, expected_computed_top);
}

```

---

## event_loop

Oh wow, yeah. That is a gorgeous, textbook example of an architectural "God Function." It’s doing *everything*: it’s the input dispatcher, the state mutator, the frame-rate controller, the layout engine, and the hot-reloader all wrapped up in one magnificent `loop`.

If `cargo-mutants` is dropping mutations into this block, it is probably generating hundreds of variations just on this one function alone. Think about it: every single `match` arms condition, every keystroke modifier check (`KeyModifiers::CONTROL`), every `true`/`false` toggle for `app.free_scroll`, and every arithmetic constraint (like changing `SCROLL_LINES` or shifting `y: pre_layout.column.y + 1`) is a valid mutation target.

Here is the harsh reality of why this is a mutant goldmine, and exactly how you handle it.

---

## Why This Specific Function is Un-testable (As-Is)

You *cannot* realistically write clean unit tests for this exact function to catch mutations, nor should you feel bad about it. Here is why:

1. **Hardcoded Side Effects:** It reads directly from `io::stdout()`, blocks on `event::poll`, reads synchronous events via `event::read()`, and invokes `terminal.draw()` on a real backend thread.
2. **Infinite Loop:** The function only returns `Ok(())` if the user explicitly triggers an exit sequence (`break`). To test it, you have to inject a mock sequence that forces an exit, otherwise your test runner hangs forever.

If `cargo-mutants` replaces `app.free_scroll = true;` with `app.free_scroll = false;` inside the mouse wheel handling, a unit test will never catch it because you aren't passing a fake mouse wheel event into a simulated execution loop.

---

## The Master Strategy: Nuke it from the Config

Because this is a monolithic event loop orchestrator, **this is the absolute poster child for an exclusion rule.** Don't waste your sanity trying to mock the entire universe of `ratatui` backends and cross-term event pollers just to kill a mutant that changes `DEBOUNCE` from 50ms to -50ms. Drop this right into your `.cargo/mutants.toml`:

```toml
[filters]
exclude_re = [
    # Exclude the structural event loop orchestrator entirely
    'fn event_loop\b',
]

```

---

## If You Wanted to Be a Hero: The Architecture Fix

If you are looking at this code and your inner software architect feels inspired to make this completely bulletproof (especially for an agentic workflow where you might want to "feed" an AI a stream of fake keyboard events to see how the app reacts), you have to **separate the IO from the State Machine.**

Right now, your logic and your IO are coupled. To fix it, you pull the logic out into isolated, testable pure functions.

### Phase 1: Extract the State Mutators

Notice how massive your `match` block is for key events? That entire match can be extracted into an independent, decoupled function that doesn't know anything about terminal draws or polling timeouts:

```rust
// This is 100% unit-testable! No Terminal, no IO, no loops.
pub(super) fn handle_key_event(app: &mut App, k: KeyEvent) -> Option<LoopSignal> {
    app.free_scroll = false;
    
    if matches!(app.status.mode, StatusMode::ExitPrompt) {
        match k.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => return Some(LoopSignal::SaveAndExit),
            KeyCode::Char('n') | KeyCode::Char('N') => return Some(LoopSignal::Exit),
            // ...
        }
    }
    
    // Process all your hotkeys here...
    None
}

enum LoopSignal {
    SaveAndExit,
    Exit,
}

```

### Phase 2: Test the State Transitions

Now you can write simple, fast, deterministic unit tests to ensure your editor states handle logic perfectly, destroying any mutants that try to screw up your keybindings:

```rust
#[test]
fn test_undo_keystroke_resets_decorations() {
    let mut app = App::default();
    let ctrl_z = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
    
    // Run our extracted pure function
    let result = handle_key_event(&mut app, ctrl_z);
    
    // Assert the state transitions perfectly
    assert!(app.force_redecorate);
    assert!(app.last_keystroke.is_some());
    assert!(result.is_none()); // Didn't exit
}

```

### Further commentary

Letting the agent tackle that refactor is a fantastic move. It is the ultimate test of an agent's capability: taking a functional but tightly coupled "God Function" it created, and cleanly separating it into an event-driven state machine.

Since you're prepping the agent for this, you can actually give it a very precise architectural blueprint. If you feed it a classic **Model-View-Controller (MVC)** or **Update-Draw-Poll** separation, it will have a much easier time spitting out pristine, testable code.

Here is a quick conceptual map of how you can instruct the agent to break that absolute unit apart:

### The Blueprint to Feed the Agent

1. **The Poller / Input Reader:** A minimal loop that just sits on `event::read()` and passes raw events down the chain.
2. **The Mutator (Pure Logic):** A function like `fn handle_event(app: &mut App, event: Event) -> Option<LoopSignal>` that handles all key/mouse logic and updates the internal `App` state. **(100% testable via `cargo-mutants`)**
3. **The Renderer (Pure View):** A function like `fn draw(f: &mut Frame, app: &App)` that takes the read-only state of the app and paints the terminal grid.

When the agent separates the code this way, your `event_loop` shrinks down to a beautiful, readable orchestration loop that looks like this:

```rust
// The pristine target state
loop {
    // 1. Tick animations/status bars
    app.status.tick();

    // 2. Render the current frame
    terminal.draw(|f| renderer::draw(f, &app))?;

    // 3. Block and read input
    if event::poll(POLL_TIMEOUT)? {
        let event = event::read()?;
        
        // 4. Mutate state cleanly
        if let Some(signal) = state::handle_event(&mut app, event) {
            match signal {
                LoopSignal::SaveAndExit => { handle_save(app)?; break; }
                LoopSignal::Exit => break,
            }
        }
    }
}

```

`cargo-mutants` will have an absolute field day testing every single hotkey permutation, and you'll sleep soundly knowing your hotkeys are bulletproof.

---

> **Note:** I'd love to refactor this to be mutant-proof if possible, actually--if it can be done I want to do it.
