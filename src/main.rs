use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal, backend::CrosstermBackend, layout::Rect, style::Style, widgets::Paragraph,
};

use yame::app::App;
use yame::config::{LayoutConfig, Theme, load_config, supports_italic};
use yame::decoration::{build_decoration_map, count_words};
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
fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    layout_config: &LayoutConfig,
) -> io::Result<()> {
    use yame::layout::{DEFAULT_MIN_COLS, compute_layout};
    use yame::renderer;

    const POLL_TIMEOUT: Duration = Duration::from_millis(16);
    const DEBOUNCE: Duration = Duration::from_millis(50);

    let min_cols = layout_config.min_cols.unwrap_or(DEFAULT_MIN_COLS);

    // Persisted across frames so mouse events can translate screen-absolute
    // coordinates to editor-relative (col, row) + scroll offset.
    let mut last_editor_area = Rect::default();

    loop {
        // Fire decoration pass if debounce has elapsed.
        // TODO(v1.5): move to background thread — build_decoration_map and count_words
        // are pure functions; when v1.5 moves them here, replace with tx.send(text) + rx.try_recv().
        if app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            let cursor_line = app.textarea.cursor().0;
            app.decoration_map =
                build_decoration_map(&text, &app.theme, app.italic_support, cursor_line);
            app.word_count = count_words(&text);
            app.last_keystroke = None;
        }
        app.status.tick();

        terminal.draw(|f| {
            let layout = compute_layout(f.area(), min_cols);

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
            // mouse click, or terminal resize — runs against the live editor area each frame.
            let (cursor_row, _) = app.textarea.cursor();
            let visible_rows = editor_area.height as usize;
            if cursor_row < app.scroll_top {
                app.scroll_top = cursor_row;
            }
            if cursor_row >= app.scroll_top + visible_rows {
                app.scroll_top = cursor_row.saturating_sub(visible_rows.saturating_sub(1));
            }

            let view = renderer::MarkdownView {
                lines: app.textarea.lines(),
                decoration_map: &app.decoration_map,
                scroll_top: app.scroll_top,
                cursor: app.textarea.cursor(),
                selection: app.textarea.selection_range(),
                theme: &app.theme,
                italic_support: app.italic_support,
                column_width: layout.column.width,
            };
            f.render_widget(view, editor_area);
            renderer::render_status_bar(f, layout.status_bar, app);
            renderer::render_info_line(f, layout.info_line, app);
            renderer::render_scrollbar(f, layout.scrollbar, app);

            // Persist so the mouse handler can translate coordinates next frame.
            last_editor_area = editor_area;
        })?;

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(k) => {
                    match (k.modifiers, k.code) {
                        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                            handle_save(app)?;
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('x')) => {
                            if handle_exit(app) {
                                break;
                            }
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                            yame::clipboard::handle_copy(app);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                            yame::clipboard::handle_paste(app);
                        }
                        // Undo/redo: pass to tui-textarea then recompute dirty,
                        // since undoing to the saved state should clear the flag.
                        (KeyModifiers::CONTROL, KeyCode::Char('z'))
                        | (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                            app.status.dismiss();
                            app.config_warnings.clear();
                            app.textarea.input(k);
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
                Event::Mouse(mut mouse) => {
                    // tui-textarea expects coordinates relative to its widget origin
                    // and uses them as (logical_row, col).  Translate from
                    // screen-absolute by subtracting the editor area's top-left and
                    // adding the current scroll offset so clicks land on the correct
                    // logical line regardless of centering margin or scroll position.
                    mouse.column = mouse.column.saturating_sub(last_editor_area.x);
                    let rel_row = mouse.row.saturating_sub(last_editor_area.y) as usize;
                    mouse.row = rel_row.saturating_add(app.scroll_top) as u16;
                    app.textarea.input(Event::Mouse(mouse));
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

#[mutants::skip] // Entry point — calls process::exit, not unit-testable.
fn main() {
    let file_path = parse_args().unwrap_or_else(|_| std::process::exit(1));
    if let Err(e) = run(file_path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
