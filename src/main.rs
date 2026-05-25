use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{
        BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::Rect, style::Style, widgets::Paragraph,
};
use tui_textarea::CursorMove;

use yame::app::App;
use yame::config::{LayoutConfig, Theme, load_config, supports_italic};
use yame::decoration::build_decoration_map;
use yame::renderer;
use yame::status::StatusMode;

#[mutants::skip] // Installs a global panic hook — untestable side effect with no return value.
fn setup_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort terminal restore on panic; ignore errors.
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}

#[mutants::skip] // Reads std::env::args() — side-effectful, not unit-testable without refactoring.
fn parse_args() -> Result<PathBuf, ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.len() {
        1 => Ok(PathBuf::from(&args[0])),
        _ => {
            eprintln!("yame — yet another markdown editor\n");
            eprintln!("Usage: yame <file.md>");
            eprintln!("       Opens <file.md> for editing, creating it if it does not exist.");
            eprintln!("\nNote: a file named exactly 'init' is a valid target; yame init");
            eprintln!("      support is planned but not yet implemented.");
            Err(())
        }
    }
}

#[mutants::skip] // Full terminal I/O orchestration — no unit-testable return value.
fn run(file_path: PathBuf) -> io::Result<()> {
    setup_panic_hook();

    let (config, config_warnings) = load_config();
    let italic_support = supports_italic();
    let mut warnings = config_warnings;
    let theme = Theme::from_config(
        &config.palette,
        &config.theme,
        &config.headings,
        &mut warnings,
    );

    let mut app = App::new(file_path, theme, italic_support, warnings)?;

    // Queue italic fallback warning if the terminal doesn't support italic rendering.
    if !italic_support {
        app.status.set_dismissible(
            "⚠ Terminal does not support italics — using color fallback  [any key to dismiss]",
        );
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app, &config.layout);

    // Always restore terminal, even on error.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[mutants::skip] // Terminal event loop — requires a real terminal backend, not unit-testable.
/// Map a screen-absolute (row, col) mouse position to a logical document
/// (row, col) position, accounting for the editor gutter, scroll offset, and
/// soft-wrapped lines.  Returns `None` if the click is outside the editor area.
fn screen_to_doc(
    screen_row: u16,
    screen_col: u16,
    editor_area: &Rect,
    scroll_top: usize,
    lines: &[String],
) -> Option<(u16, u16)> {
    if screen_row < editor_area.y
        || screen_col < editor_area.x
        || screen_row >= editor_area.y + editor_area.height
        || screen_col >= editor_area.x + editor_area.width
    {
        return None;
    }
    let cw = (editor_area.width as usize)
        .saturating_sub(2 * renderer::GUTTER as usize)
        .max(1);
    let click_vis_row = (screen_row - editor_area.y) as usize;
    let click_col = screen_col.saturating_sub(editor_area.x + renderer::GUTTER) as usize;

    let mut vis = 0usize;
    for (li, line) in lines.iter().enumerate().skip(scroll_top) {
        let wrapped = renderer::wrap_line(line, cw);
        let seg_count = wrapped.len().max(1);
        if vis + seg_count > click_vis_row {
            let si = click_vis_row - vis;
            let char_ranges = renderer::wrap_char_ranges(line, &wrapped);
            let seg_char_start = char_ranges.get(si).map_or(0, |&(start, _)| start);
            let doc_col = (seg_char_start + click_col).min(line.chars().count());
            return Some((li as u16, doc_col as u16));
        }
        vis += seg_count;
    }
    // Click landed below all content — go to last line, column 0.
    Some((lines.len().saturating_sub(1) as u16, 0))
}

/// Returns true if the key is a pure cursor-movement key that cannot change
/// document content. Used to skip the decoration debounce timer on nav presses.
fn is_navigation_key(k: &crossterm::event::KeyEvent) -> bool {
    matches!(
        k.code,
        KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
    )
}

/// Extract the currently-selected text from the textarea, or `None` if there
/// is no active selection.  Does not fall back to the current line.
fn get_selection_text(app: &App) -> Option<String> {
    let ((row_start, col_start), (row_end, col_end)) = app.textarea.selection_range()?;
    let lines = app.textarea.lines();
    if row_start == row_end {
        let chars: Vec<char> = lines[row_start].chars().collect();
        Some(chars[col_start..col_end.min(chars.len())].iter().collect())
    } else {
        let mut result = String::new();
        for row in row_start..=row_end {
            if row >= lines.len() {
                break;
            }
            let chars: Vec<char> = lines[row].chars().collect();
            let start = if row == row_start { col_start } else { 0 };
            let end = if row == row_end {
                col_end.min(chars.len())
            } else {
                chars.len()
            };
            result.extend(&chars[start..end]);
            if row < row_end {
                result.push('\n');
            }
        }
        Some(result)
    }
}

/// If there is an active selection and `k` is a pair-opener (`(`, `[`, `{`,
/// `"`, `'`, `` ` ``, `*`, `_`), wrap the selection with the corresponding
/// pair and return `true`.  Returns `false` in all other cases so the caller
/// can fall through to normal input handling.
fn handle_pair_wrap(app: &mut App, k: crossterm::event::KeyEvent) -> bool {
    // Only fire on bare character presses — never on Ctrl/Alt chords.
    if k.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return false;
    }
    let close = match k.code {
        KeyCode::Char('(') => ')',
        KeyCode::Char('[') => ']',
        KeyCode::Char('{') => '}',
        KeyCode::Char('"') => '"',
        KeyCode::Char('\'') => '\'',
        KeyCode::Char('`') => '`',
        KeyCode::Char('*') => '*',
        KeyCode::Char('_') => '_',
        _ => return false,
    };
    // Capture the selected text before any mutation.
    let selected = match get_selection_text(app) {
        Some(s) => s,
        None => return false,
    };
    // `input(k)` calls `insert_char(open)` which internally calls
    // `delete_selection()` first, then places the open delimiter.
    app.textarea.input(k);
    // Append the captured text and the closing delimiter.
    app.textarea.insert_str(format!("{selected}{close}"));
    true
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    layout_config: &LayoutConfig,
) -> io::Result<()> {
    use yame::layout::{DEFAULT_MIN_COLS, compute_layout};

    const POLL_TIMEOUT: Duration = Duration::from_millis(16);
    const DEBOUNCE: Duration = Duration::from_millis(50);
    /// Virtual empty rows kept below the cursor at end-of-document.
    /// Implemented by firing the scroll clamp early (not by shrinking the viewport),
    /// so all terminal rows remain usable when the cursor is mid-document.
    const BOTTOM_PADDING: usize = 3;
    /// Lines moved per scroll-wheel tick.
    const SCROLL_LINES: usize = 3;

    let min_cols = layout_config.min_cols.unwrap_or(DEFAULT_MIN_COLS);

    // Initial decoration pass — populate before the first frame so the file
    // renders with bold/italic/etc. immediately on open, without needing a
    // keystroke to trigger the debounce.
    {
        let text = app.textarea.lines().join("\n");
        let (map, wc) = build_decoration_map(&text, &app.theme, app.italic_support);
        app.decoration_map = map;
        app.word_count = wc;
    }

    // Persisted across frames so mouse events can translate screen-absolute
    // coordinates to editor-relative (col, row) + scroll offset.
    let mut last_editor_area = Rect::default();
    // Set on the first Drag event after a Down so that start_selection() is
    // called exactly once per drag gesture (subsequent Drag events just extend).
    let mut drag_selecting = false;

    loop {
        // Fire decoration pass if debounce has elapsed.
        // v1.5 migration point: move to background thread — build_decoration_map and count_words
        // are pure functions; swap this block for tx.send(text) + rx.try_recv() when ready.
        if app.force_redecorate || app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            let (map, wc) = build_decoration_map(&text, &app.theme, app.italic_support);
            app.decoration_map = map;
            app.word_count = wc;
            app.last_keystroke = None;
            app.force_redecorate = false;
        }
        app.status.tick();

        // ── Pre-draw scroll clamp ─────────────────────────────────────────────
        // Compute layout from the current terminal size so clamp_scroll can run
        // before terminal.draw(), keeping the draw closure as a pure render.
        // terminal.size() and f.area() inside draw() should agree; if a resize
        // races between the two calls it will be corrected on the next iteration.
        {
            let term_size = terminal.size()?;
            let term_area = Rect::new(0, 0, term_size.width, term_size.height);
            let pre_layout = compute_layout(term_area, min_cols);
            let pre_editor_area = if !app.config_warnings.is_empty() && pre_layout.column.height > 0
            {
                Rect {
                    y: pre_layout.column.y + 1,
                    height: pre_layout.column.height.saturating_sub(1),
                    ..pre_layout.column
                }
            } else {
                pre_layout.column
            };
            clamp_scroll(
                app,
                pre_editor_area,
                pre_layout.column.width,
                BOTTOM_PADDING,
            );
        }

        execute!(io::stdout(), BeginSynchronizedUpdate)?;
        terminal.draw(|f| {
            let layout = compute_layout(f.area(), min_cols);

            // Flood-fill the full content area (everything above the info/status rows)
            // with the editor background so the gutters on either side of the centered
            // column share the same colour — one unified canvas rather than two
            // distinct gutter strips flanking the column.
            let content_bg_area = Rect {
                x: layout.full.x,
                y: layout.full.y,
                width: layout.full.width,
                height: layout.column.height,
            };
            f.render_widget(
                Paragraph::new("").style(Style::default().bg(app.theme.bg)),
                content_bg_area,
            );

            // Config warning banner — occupies the first row of the column when present.
            // Dismissed on any keystroke (see event handling below).
            let editor_area = if !app.config_warnings.is_empty() && layout.column.height > 0 {
                let warn_area = Rect {
                    height: 1,
                    ..layout.column
                };
                let msg = format!(" ⚠  {}  [any key to dismiss]", app.config_warnings[0]);
                f.render_widget(
                    Paragraph::new(msg)
                        .style(Style::default().fg(app.theme.warning).bg(app.theme.ui_bar)),
                    warn_area,
                );
                Rect {
                    y: layout.column.y + 1,
                    height: layout.column.height.saturating_sub(1),
                    ..layout.column
                }
            } else {
                layout.column
            };

            let view = renderer::MarkdownView {
                lines: app.textarea.lines(),
                decoration_map: &app.decoration_map,
                scroll_top: app.scroll_top,
                cursor: app.textarea.cursor(),
                selection: app.textarea.selection_range(),
                theme: &app.theme,
                column_width: layout.column.width,
            };
            f.render_widget(view, editor_area);
            renderer::render_status_bar(f, layout.status_bar, app);
            renderer::render_info_line(f, layout.info_line, app);

            // Persist so the mouse handler can translate coordinates next frame.
            last_editor_area = editor_area;
        })?;
        execute!(io::stdout(), EndSynchronizedUpdate)?;

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(k) => {
                    // ── ExitPrompt modal: intercept ALL keys before global shortcuts ──────
                    // Without this guard, Esc hits the outer (NONE, Esc) arm and re-fires
                    // handle_exit (stuck in a loop), and Ctrl+C triggers clipboard copy
                    // instead of canceling. The modal owns its entire key space.
                    if matches!(app.status.mode, StatusMode::ExitPrompt) {
                        match k.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                handle_save(app)?;
                                break;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                break;
                            }
                            // Esc, c/C (any modifier, so Ctrl+C works), and x/X cancel.
                            KeyCode::Esc
                            | KeyCode::Char('c')
                            | KeyCode::Char('C')
                            | KeyCode::Char('x')
                            | KeyCode::Char('X') => {
                                app.status.mode = StatusMode::Normal;
                            }
                            _ => {}
                        }
                    } else {
                        match (k.modifiers, k.code) {
                            (KeyModifiers::CONTROL, KeyCode::Char('s'))
                            | (KeyModifiers::SUPER, KeyCode::Char('s')) => {
                                handle_save(app)?;
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('x'))
                            | (KeyModifiers::NONE, KeyCode::Esc) => {
                                if handle_exit(app) {
                                    break;
                                }
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('c'))
                            | (KeyModifiers::SUPER, KeyCode::Char('c')) => {
                                yame::clipboard::handle_copy(app);
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('v'))
                            | (KeyModifiers::SUPER, KeyCode::Char('v')) => {
                                yame::clipboard::handle_paste(app);
                                app.force_redecorate = true;
                            }
                            // Undo/redo: tui-textarea uses Ctrl+U/Ctrl+R internally,
                            // but we expose the conventional Ctrl+Z / Ctrl+Y bindings
                            // by calling the methods directly. After each operation we
                            // recompute the dirty flag so undoing back to saved state
                            // clears it, and trigger a decoration refresh.
                            (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                                app.status.dismiss();
                                app.config_warnings.clear();
                                app.textarea.undo();
                                app.force_redecorate = true;
                                app.last_keystroke = Some(std::time::Instant::now());
                                app.recompute_dirty();
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                                app.status.dismiss();
                                app.config_warnings.clear();
                                app.textarea.redo();
                                app.force_redecorate = true;
                                app.last_keystroke = Some(std::time::Instant::now());
                                app.recompute_dirty();
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                                let (new_config, new_warnings) = load_config();
                                let mut warnings = new_warnings;
                                app.theme = Theme::from_config(
                                    &new_config.palette,
                                    &new_config.theme,
                                    &new_config.headings,
                                    &mut warnings,
                                );
                                app.config_warnings = warnings;
                                app.status
                                    .set_timed("Config reloaded.", Duration::from_millis(1500));
                                // Trigger a decoration rebuild so the new theme colors
                                // are applied immediately on the next frame.
                                app.last_keystroke = Some(std::time::Instant::now());
                            }
                            _ => {
                                // Dismiss any dismissible message on any content keypress.
                                app.status.dismiss();
                                app.config_warnings.clear();
                                // Capture nav flag before input() consumes the event.
                                let is_nav = is_navigation_key(&k);

                                // Smart pair wrapping: if there's an active selection and
                                // the user types a bracket/quote opener, wrap the selection
                                // with the corresponding pair instead of replacing it.
                                if !is_nav && handle_pair_wrap(app, k) {
                                    app.force_redecorate = true;
                                    app.mark_keystroke();
                                } else {
                                    let prev_line_count = app.textarea.lines().len();
                                    app.textarea.input(k);
                                    if app.textarea.lines().len() != prev_line_count {
                                        app.force_redecorate = true;
                                    }
                                    // Navigation keys move the cursor without changing content —
                                    // skip the decoration debounce timer to avoid a redundant
                                    // full re-parse of the document on every arrow-key press.
                                    if is_nav {
                                        app.recompute_dirty();
                                    } else {
                                        app.mark_keystroke();
                                    }
                                }
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        // Scroll events are intercepted before tui-textarea to avoid
                        // two problems:
                        //   1. tui-textarea accumulates its own internal scroll offset
                        //      that diverges from our scroll_top, producing "ghost"
                        //      scroll steps that must be unwound before cursor moves.
                        //   2. On scroll-up mid-viewport, the visual clamp only fires
                        //      at the bottom boundary, so cursor drifts through the
                        //      visible area without the viewport moving at all.
                        //
                        // Fix: move cursor ourselves, and on ScrollUp also decrement
                        // scroll_top immediately so the viewport follows the gesture
                        // without waiting for the cursor to hit the top of the view.
                        // The visual clamp corrects any overshoot on the next frame.
                        MouseEventKind::ScrollDown => {
                            for _ in 0..SCROLL_LINES {
                                app.textarea.move_cursor(CursorMove::Down);
                            }
                            // scroll_top is driven upward by the visual clamp when
                            // the cursor approaches the bottom — no manual adjustment.
                        }
                        MouseEventKind::ScrollUp => {
                            for _ in 0..SCROLL_LINES {
                                app.textarea.move_cursor(CursorMove::Up);
                            }
                            // Immediately pull the viewport up so content moves on
                            // every tick.  The clamp corrects if this undershoots.
                            app.scroll_top = app.scroll_top.saturating_sub(SCROLL_LINES);
                        }
                        // Click: reposition cursor. Drag: extend selection.
                        //
                        // tui-textarea's built-in mouse handling relies on knowing
                        // the area it was rendered into (set via Widget::render). We
                        // use a custom MarkdownView renderer so tui-textarea never
                        // receives that area, making its internal mouse logic unreliable.
                        // Instead we compute the document position ourselves and use
                        // CursorMove::Jump, which correctly handles soft-wrap offsets
                        // and our scroll_top.
                        MouseEventKind::Down(MouseButton::Left) => {
                            drag_selecting = false;
                            if let Some((doc_row, doc_col)) = screen_to_doc(
                                mouse.row,
                                mouse.column,
                                &last_editor_area,
                                app.scroll_top,
                                app.textarea.lines(),
                            ) {
                                app.textarea.cancel_selection();
                                app.textarea.move_cursor(CursorMove::Jump(doc_row, doc_col));
                            }
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            if let Some((doc_row, doc_col)) = screen_to_doc(
                                mouse.row,
                                mouse.column,
                                &last_editor_area,
                                app.scroll_top,
                                app.textarea.lines(),
                            ) {
                                if !drag_selecting {
                                    // Anchor the selection at the Down position
                                    // (cursor is already there from the Down handler).
                                    app.textarea.start_selection();
                                    drag_selecting = true;
                                }
                                // move_cursor preserves selection when
                                // selection_start.is_some(), extending it to here.
                                app.textarea.move_cursor(CursorMove::Jump(doc_row, doc_col));
                            }
                        }
                        _ => {} // Up, Moved, etc. — nothing to do
                    }
                }
                Event::Resize(_, _) => {
                    // next draw() picks up new dimensions automatically
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[mutants::skip] // Calls std::fs::write — I/O side effect; status message logic tested in Phase 11.
fn handle_save(app: &mut App) -> io::Result<()> {
    // Build the content to write.  Normally we always append a POSIX trailing
    // newline.  Exception: if the file was empty (0 bytes or new) at load time
    // and the buffer is still empty, write nothing — avoids growing a 0-byte
    // file to a 1-byte bare newline on a no-op save.
    let lines = app.textarea.lines();
    let content = if app.initial_file_empty && lines == [""] {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    // Create parent directories if they don't exist (e.g. `yame notes/new.md`).
    if let Some(parent) = app.file_path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        app.status.set_dismissible(format!("⚠ Save failed: {e}"));
        return Ok(());
    }
    match std::fs::write(&app.file_path, &content) {
        Ok(()) => {
            app.saved_content = Some(app.textarea.lines().to_vec());
            app.is_dirty = false;
            app.status.set_timed("Saved.", Duration::from_millis(1500));
        }
        Err(e) => {
            app.status.set_dismissible(format!("⚠ Save failed: {e}"));
        }
    }
    Ok(())
}

/// Clamp `app.scroll_top` so the cursor remains visible within `editor_area`.
///
/// Extracted from the draw closure so that `terminal.draw` can be a pure render
/// with no state mutations.  Call this with the pre-draw layout values (derived
/// from `terminal.size()`) each iteration before invoking `terminal.draw`.
///
/// `col_width`      – full column width in terminal cells (includes gutters).
/// `bottom_padding` – virtual empty rows to keep below the cursor.
fn clamp_scroll(app: &mut App, editor_area: Rect, col_width: u16, bottom_padding: usize) {
    let (cursor_row, cursor_col) = app.textarea.cursor();
    let visible_rows = editor_area.height as usize;
    let lines = app.textarea.lines();
    let cw = (col_width as usize)
        .saturating_sub(2 * renderer::GUTTER as usize)
        .max(1);

    // ── Scroll up: cursor above the viewport ──────────────────────────────────
    if cursor_row < app.scroll_top {
        app.scroll_top = cursor_row;
    }

    // ── Scroll down: check cursor's visual (post-wrap) position ───────────────
    // Count visual rows occupied by logical lines [scroll_top, cursor_row).
    let above_visual: usize = lines
        .get(app.scroll_top..cursor_row.min(lines.len()))
        .unwrap_or(&[])
        .iter()
        .map(|l| renderer::wrap_line(l, cw).len())
        .sum();

    // Find which visual sub-row within cursor_row the cursor sits on.
    // Uses wrap_char_ranges so that spaces skipped at soft-wrap boundaries are
    // accounted for — without it, char_start would be off by 1 after the first
    // soft break, misidentifying the cursor's sub-row and terminal column.
    let cursor_line_str = lines.get(cursor_row).map_or("", |s| s.as_str());
    let cursor_wraps = renderer::wrap_line(cursor_line_str, cw);
    let cursor_subrow = {
        let char_ranges = renderer::wrap_char_ranges(cursor_line_str, &cursor_wraps);
        let mut subrow = 0usize;
        for (i, &(char_start, char_len)) in char_ranges.iter().enumerate() {
            let char_end = char_start + char_len;
            if cursor_col < char_end || i + 1 == char_ranges.len() {
                subrow = i;
                break;
            }
        }
        subrow
    };

    let cursor_visual = above_visual + cursor_subrow;

    // Fire `bottom_padding` rows early so the cursor never sits flush at the
    // very bottom of the viewport.
    if cursor_visual + bottom_padding >= visible_rows {
        // Walk backward from cursor_row, accumulating visual rows, until we
        // find a scroll_top that places the cursor at (last visible row − padding).
        let headroom = visible_rows.saturating_sub(1 + cursor_subrow + bottom_padding);
        let mut remaining = headroom;
        let mut new_top = cursor_row;
        while new_top > 0 {
            let prev_wraps =
                renderer::wrap_line(lines.get(new_top - 1).map_or("", |s| s.as_str()), cw).len();
            if prev_wraps > remaining {
                break;
            }
            remaining -= prev_wraps;
            new_top -= 1;
        }
        app.scroll_top = new_top;
    }
}

/// Returns true if the app should exit.
fn handle_exit(app: &mut App) -> bool {
    if app.is_dirty {
        app.status.mode = StatusMode::ExitPrompt;
        false
    } else {
        true
    }
}

#[mutants::skip] // Entry point — calls process::exit, not unit-testable.
fn main() {
    let file_path = parse_args().unwrap_or_else(|_| std::process::exit(1));
    if let Err(e) = run(file_path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tui_textarea::TextArea;

    use yame::config::Theme;
    use yame::decoration::DecorationMap;
    use yame::status::{StatusLine, StatusMode};

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("test.md"),
            is_dirty: false,
            saved_content: None,
            theme: Theme::default_theme(),
            italic_support: false,
            last_keystroke: None,
            force_redecorate: false,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
            clipboard: None,
            initial_file_empty: false,
        }
    }

    #[test]
    fn handle_exit_clean_returns_true() {
        let mut app = make_app();
        app.is_dirty = false;
        assert!(handle_exit(&mut app), "clean file should exit immediately");
        assert!(
            matches!(app.status.mode, StatusMode::Normal),
            "status unchanged for clean exit"
        );
    }

    #[test]
    fn handle_exit_dirty_returns_false_and_prompts() {
        let mut app = make_app();
        app.is_dirty = true;
        assert!(!handle_exit(&mut app), "dirty file must not exit");
        assert!(
            matches!(app.status.mode, StatusMode::ExitPrompt),
            "dirty exit must show ExitPrompt"
        );
    }

    // ── clamp_scroll tests ────────────────────────────────────────────────────
    // col_width=82 → cw = 82 - 2*GUTTER.  GUTTER=2 → cw=78.
    // All test lines are short enough that wrap_line returns 1 segment each.
    const TEST_COL: u16 = 82; // gives cw=78 with GUTTER=2

    fn make_editor_area(height: u16) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: TEST_COL,
            height,
        }
    }

    fn make_app_with_lines(lines: &[&str]) -> App {
        let mut app = make_app();
        app.textarea = TextArea::new(lines.iter().map(|s| s.to_string()).collect());
        app
    }

    #[test]
    fn clamp_scroll_cursor_above_viewport_scrolls_up() {
        // scroll_top=5, cursor at line 2 → must scroll up to 2.
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 5;
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(2, 0));
        clamp_scroll(&mut app, make_editor_area(10), TEST_COL, 0);
        assert_eq!(
            app.scroll_top, 2,
            "cursor above viewport → scroll_top = cursor_row"
        );
    }

    #[test]
    fn clamp_scroll_cursor_in_viewport_unchanged() {
        // scroll_top=0, cursor at line 3, viewport height=10 → no change.
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 0;
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(3, 0));
        clamp_scroll(&mut app, make_editor_area(10), TEST_COL, 0);
        assert_eq!(
            app.scroll_top, 0,
            "cursor inside viewport → scroll_top unchanged"
        );
    }

    #[test]
    fn clamp_scroll_cursor_at_bottom_with_padding_scrolls_down() {
        // 20 short lines, scroll_top=0, cursor at line 9, viewport=10, padding=3.
        // cursor_visual=9, 9+3=12 >= 10 → must scroll down.
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 0;
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(9, 0));
        clamp_scroll(&mut app, make_editor_area(10), TEST_COL, 3);
        assert!(
            app.scroll_top > 0,
            "cursor near bottom with padding → scroll_top advances"
        );
    }

    #[test]
    fn clamp_scroll_zero_height_does_not_panic() {
        // Degenerate case: zero-height editor (e.g. terminal too small).
        let mut app = make_app_with_lines(&["a"; 5]);
        clamp_scroll(&mut app, make_editor_area(0), TEST_COL, 3);
        // Must not panic; scroll_top may be anything.
    }

    // ── get_selection_text tests ──────────────────────────────────────────────

    fn make_key(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn get_selection_text_none_when_no_selection() {
        let app = make_app_with_lines(&["hello world"]);
        assert_eq!(get_selection_text(&app), None);
    }

    #[test]
    fn get_selection_text_single_line() {
        let mut app = make_app_with_lines(&["hello world"]);
        // Place cursor at col 0, start selection, move right 5 chars → selects "hello".
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        assert_eq!(get_selection_text(&app), Some("hello".to_string()));
    }

    #[test]
    fn get_selection_text_multiline() {
        let mut app = make_app_with_lines(&["abc", "def"]);
        // Start selection at (0,0), move down one line → selects "abc\ndef" up to (1,3).
        app.textarea.start_selection();
        app.textarea.move_cursor(tui_textarea::CursorMove::Down);
        app.textarea.move_cursor(tui_textarea::CursorMove::End);
        let text = get_selection_text(&app).unwrap_or_default();
        assert!(
            text.contains('\n'),
            "multiline selection must include newline"
        );
        assert!(text.starts_with("abc"), "first line preserved");
    }

    // ── handle_pair_wrap tests ────────────────────────────────────────────────

    #[test]
    fn pair_wrap_bracket_wraps_selection() {
        let mut app = make_app_with_lines(&["hello"]);
        // Select "hello" (all 5 chars).
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        let handled = handle_pair_wrap(&mut app, make_key(KeyCode::Char('[')));
        assert!(handled, "pair wrap must return true when selection present");
        let line = app.textarea.lines()[0].clone();
        assert_eq!(line, "[hello]", "selection wrapped with square brackets");
    }

    #[test]
    fn pair_wrap_star_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        app.textarea.start_selection();
        for _ in 0..2 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('*')));
        assert_eq!(app.textarea.lines()[0], "*hi*");
    }

    #[test]
    fn pair_wrap_no_selection_returns_false() {
        let mut app = make_app_with_lines(&["hello"]);
        // No active selection — must not wrap and must return false.
        let handled = handle_pair_wrap(&mut app, make_key(KeyCode::Char('[')));
        assert!(!handled, "no selection → pair wrap is a no-op");
        assert_eq!(app.textarea.lines()[0], "hello", "content unchanged");
    }

    #[test]
    fn pair_wrap_non_pair_key_returns_false() {
        let mut app = make_app_with_lines(&["hello"]);
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        let handled = handle_pair_wrap(&mut app, make_key(KeyCode::Char('a')));
        assert!(!handled, "non-pair key → pair wrap is a no-op");
    }

    #[test]
    fn pair_wrap_ctrl_chord_ignored() {
        let mut app = make_app_with_lines(&["hello"]);
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        // Ctrl+[ should not trigger pair wrapping.
        let k = crossterm::event::KeyEvent::new(KeyCode::Char('['), KeyModifiers::CONTROL);
        let handled = handle_pair_wrap(&mut app, k);
        assert!(!handled, "Ctrl chord must not trigger pair wrap");
    }
}
