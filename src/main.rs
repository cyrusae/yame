// Suppress dead_code during phased build-out; removed in Phase 11 when all modules are wired.
#![allow(dead_code)]

mod app;
mod clipboard;
mod config;
mod decoration;
mod layout;
mod renderer;
mod status;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use config::{Theme, load_config, supports_italic};

fn setup_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort terminal restore on panic; ignore errors.
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}

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

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    layout_config: &config::LayoutConfig,
) -> io::Result<()> {
    use layout::{DEFAULT_MIN_COLS, compute_layout};

    const POLL_TIMEOUT: Duration = Duration::from_millis(16);
    const DEBOUNCE: Duration = Duration::from_millis(50);

    let min_cols = layout_config.min_cols.unwrap_or(DEFAULT_MIN_COLS);

    loop {
        // Fire decoration pass if debounce has elapsed.
        // TODO(v1.5): move to background thread
        if app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let _text = app.textarea.lines().join("\n");
            let _cursor_line = app.textarea.cursor().0;
            // decoration_map and word_count wired in Phase 8
            app.last_keystroke = None;
        }
        app.status.tick();

        terminal.draw(|f| {
            let layout = compute_layout(f.area(), min_cols);
            renderer::render_status_bar(f, layout.status_bar, app);
            renderer::render_info_line(f, layout.info_line, app);
            renderer::render_scrollbar(f, layout.scrollbar, app);
            // Main editor column wired in Phase 7
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
                            clipboard::handle_copy(app);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                            clipboard::handle_paste(app);
                        }
                        _ => {
                            // Handle exit prompt key intercepts
                            if matches!(app.status.mode, status::StatusMode::ExitPrompt) {
                                match k.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                                        handle_save(app)?;
                                        break;
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        break;
                                    }
                                    KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C') => {
                                        app.status.mode = status::StatusMode::Normal;
                                    }
                                    _ => {}
                                }
                            } else {
                                // Dismiss any dismissible message on any keypress
                                app.status.dismiss();
                                app.textarea.input(k);
                                app.mark_keystroke();
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
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

fn handle_save(app: &mut App) -> io::Result<()> {
    let content = app.textarea.lines().join("\n");
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
        app.status.mode = status::StatusMode::ExitPrompt;
        false
    } else {
        true
    }
}

fn main() {
    let file_path = parse_args().unwrap_or_else(|_| std::process::exit(1));
    if let Err(e) = run(file_path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
