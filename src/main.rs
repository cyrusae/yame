use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::Rect, style::Style, widgets::Paragraph,
};
use tui_textarea::CursorMove;

use yame::app::App;
use yame::config::{LayoutConfig, Theme, load_config, supports_italic};
use yame::decoration::{build_decoration_map, count_words};
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
            eprintln!("Usage: yame <file>");
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
    if screen_row < editor_area.y || screen_col < editor_area.x {
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
            let seg_start: usize = wrapped[..si].iter().map(|s| s.chars().count()).sum();
            let doc_col = (seg_start + click_col).min(line.chars().count());
            return Some((li as u16, doc_col as u16));
        }
        vis += seg_count;
    }
    // Click landed below all content — go to last line, column 0.
    Some((lines.len().saturating_sub(1) as u16, 0))
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
        app.decoration_map = build_decoration_map(&text, &app.theme, app.italic_support);
        app.word_count = count_words(&text);
    }

    // Persisted across frames so mouse events can translate screen-absolute
    // coordinates to editor-relative (col, row) + scroll offset.
    let mut last_editor_area = Rect::default();
    // Set on the first Drag event after a Down so that start_selection() is
    // called exactly once per drag gesture (subsequent Drag events just extend).
    let mut drag_selecting = false;

    loop {
        // Fire decoration pass if debounce has elapsed.
        // TODO(v1.5): move to background thread — build_decoration_map and count_words
        // are pure functions; when v1.5 moves them here, replace with tx.send(text) + rx.try_recv().
        if app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            app.decoration_map = build_decoration_map(&text, &app.theme, app.italic_support);
            app.word_count = count_words(&text);
            app.last_keystroke = None;
        }
        app.status.tick();

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

            // Clamp scroll_top so the cursor stays visible after every keystroke,
            // mouse click, or terminal resize.
            //
            // IMPORTANT: we track *visual* rows (after soft/hard wrap), not logical
            // rows.  A logical line that wraps into N visual rows consumes N slots in
            // the viewport, so a pure logical-row clamp lets the cursor drift off the
            // bottom whenever wrapped lines sit between scroll_top and the cursor.
            //
            // BOTTOM_PADDING is applied by firing the scroll clamp BOTTOM_PADDING rows
            // early — the viewport itself stays full height so all rows remain usable
            // mid-document.  The renderer already draws canvas bg past end-of-content,
            // so those virtual padding rows appear naturally.
            //
            // content_width must match the renderer's own calculation exactly so that
            // wrap_line returns the same split as what gets drawn.
            let (cursor_row, cursor_col) = app.textarea.cursor();
            let visible_rows = editor_area.height as usize;
            let lines = app.textarea.lines();
            let cw = (layout.column.width as usize)
                .saturating_sub(2 * renderer::GUTTER as usize)
                .max(1);

            // ── Scroll up: cursor above the viewport ──────────────────────────
            if cursor_row < app.scroll_top {
                app.scroll_top = cursor_row;
            }

            // ── Scroll down: check cursor's visual position ───────────────────
            // Count visual rows occupied by logical lines [scroll_top, cursor_row).
            let above_visual: usize = lines
                .get(app.scroll_top..cursor_row.min(lines.len()))
                .unwrap_or(&[])
                .iter()
                .map(|l| renderer::wrap_line(l, cw).len())
                .sum();

            // Find which visual sub-row within cursor_row the cursor sits on.
            // Mirrors the renderer's cursor-tracking logic (pointer-arithmetic char_start).
            let cursor_line_str = lines.get(cursor_row).map_or("", |s| s.as_str());
            let cursor_wraps = renderer::wrap_line(cursor_line_str, cw);
            let cursor_subrow = cursor_wraps
                .iter()
                .enumerate()
                .rev()
                .find(|(_, wrap)| {
                    let byte_off =
                        (wrap.as_ptr() as usize).wrapping_sub(cursor_line_str.as_ptr() as usize);
                    let char_start = cursor_line_str[..byte_off].chars().count();
                    cursor_col >= char_start
                })
                .map_or(0, |(i, _)| i);

            let cursor_visual = above_visual + cursor_subrow;

            // Fire BOTTOM_PADDING rows early so the cursor never sits flush at the
            // very bottom of the viewport.
            if cursor_visual + BOTTOM_PADDING >= visible_rows {
                // Walk backward from cursor_row, accumulating visual rows, until we
                // find a scroll_top that places the cursor at (last visible row − padding).
                let headroom = visible_rows.saturating_sub(1 + cursor_subrow + BOTTOM_PADDING);
                let mut remaining = headroom;
                let mut new_top = cursor_row;
                while new_top > 0 {
                    let prev_wraps =
                        renderer::wrap_line(lines.get(new_top - 1).map_or("", |s| s.as_str()), cw)
                            .len();
                    if prev_wraps > remaining {
                        break;
                    }
                    remaining -= prev_wraps;
                    new_top -= 1;
                }
                app.scroll_top = new_top;
            }

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

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(k) => {
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
                            app.last_keystroke = Some(std::time::Instant::now());
                            app.recompute_dirty();
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                            app.status.dismiss();
                            app.config_warnings.clear();
                            app.textarea.redo();
                            app.last_keystroke = Some(std::time::Instant::now());
                            app.recompute_dirty();
                        }
                        _ => {
                            // Handle exit prompt key intercepts
                            if matches!(app.status.mode, StatusMode::ExitPrompt) {
                                match k.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        handle_save(app)?;
                                        break;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        break;
                                    }
                                    KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') => {
                                        app.status.mode = StatusMode::Normal;
                                    }
                                    _ => {}
                                }
                            } else {
                                // Dismiss any dismissible message on any keypress
                                app.status.dismiss();
                                app.config_warnings.clear();
                                app.textarea.input(k);
                                app.mark_keystroke();
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
    // Always write a POSIX-compliant trailing newline.  Internally lines have no
    // trailing newline; saved_content stores the same internal representation, so
    // the dirty comparison is unaffected.
    let content = app.textarea.lines().join("\n") + "\n";
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
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
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
}
