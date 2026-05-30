use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use yame::app::{App, is_likely_binary, resolve_file_mode};
use yame::config::{Theme, load_config, supports_italic};

mod cli;
mod commands;
mod input;

#[mutants::skip] // Installs a global panic hook — untestable side effect.
fn setup_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}

#[mutants::skip] // Full terminal I/O orchestration — not unit-testable.
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

    let tab_width = config.layout.tab_width.unwrap_or(4) as usize;
    let powerline_glyphs = config.layout.powerline_glyphs.unwrap_or(true);
    let show_line_numbers = config.layout.line_numbers.unwrap_or(false);
    let highlight_cache = config.highlighting.enabled.then(|| {
        let palette_theme = config
            .highlighting
            .use_palette_colors
            .then(|| yame::highlighting::build_palette_theme(&theme));
        yame::highlighting::HighlightCache::new(
            true,
            config.highlighting.syntect_theme.clone(),
            palette_theme,
        )
    });
    let file_mode = resolve_file_mode(&file_path, &config.filetype);

    // Refuse to open binary files — null bytes would corrupt the editor buffer.
    if is_likely_binary(&file_path) {
        eprintln!(
            "error: '{}' appears to be a binary file.",
            file_path.display()
        );
        eprintln!("yame can only open text files.");
        std::process::exit(1);
    }

    let mut app = App::new(
        file_path,
        theme,
        italic_support,
        powerline_glyphs,
        warnings,
        tab_width,
        highlight_cache,
        file_mode,
        show_line_numbers,
    )?;

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

    let result = input::event_loop(&mut terminal, &mut app, &config.layout);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

#[mutants::skip] // Entry point — calls process::exit, not unit-testable.
fn main() {
    let command = cli::parse_args().unwrap_or_else(|_| std::process::exit(1));
    match command {
        cli::Command::Edit(path) => {
            if let Err(e) = run(path) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        cli::Command::Init { shell } => {
            let shell_name = shell.unwrap_or_else(cli::detect_shell);
            if shell_name != "bash" && shell_name != "zsh" {
                eprintln!("================================================================");
                eprintln!("yame init: unsupported shell '{shell_name}'.");
                eprintln!("================================================================");
                eprintln!("Supported shells: bash, zsh");
                eprintln!("  eval \"$(yame init bash)\"");
                eprintln!("  eval \"$(yame init zsh)\"");
                eprintln!("================================================================");
                std::process::exit(1);
            }
            println!("{}", cli::shell_init_str(&shell_name));
        }
        cli::Command::WriteConfig => {
            cli::run_write_config();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
