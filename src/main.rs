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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, file_path);

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
    _file_path: PathBuf,
) -> io::Result<()> {
    const POLL_TIMEOUT: Duration = Duration::from_millis(16);

    loop {
        terminal.draw(|_f| { /* placeholder — Phase 7 wires the real renderer */ })?;

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(k)
                    if k.code == KeyCode::Char('q')
                        || (k.modifiers.contains(event::KeyModifiers::CONTROL)
                            && k.code == KeyCode::Char('x')) =>
                {
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() {
    let file_path = parse_args().unwrap_or_else(|_| std::process::exit(1));
    if let Err(e) = run(file_path) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
