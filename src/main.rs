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
mod picker;
mod preview;

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
fn run(file_path: Option<PathBuf>, read_only: bool) -> io::Result<()> {
    setup_panic_hook();

    // Binary check before touching the terminal — the error goes to the
    // normal terminal so it's visible regardless of TUI state.
    if let Some(p) = &file_path
        && is_likely_binary(p)
    {
        eprintln!("error: '{}' appears to be a binary file.", p.display());
        eprintln!("yame can only open text files.");
        std::process::exit(1);
    }

    // Enter the alternate screen *before* loading config / computing
    // decorations.  This prevents the terminal from briefly showing
    // scrollback or lf's UI during initialisation — the blank alternate
    // screen is visually neutral, so the first draw lands on a clean slate.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // All remaining initialisation + the event loop run inside the alternate
    // screen.  We store the result and always clean up, even on error.
    let result = init_and_run(&mut terminal, file_path, read_only);

    // Best-effort cleanup — preserve the original error if cleanup also fails.
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    result
}

/// Load config, build the app, run the event loop.
///
/// Separated from `run` so the terminal is always cleaned up even when
/// initialisation (config load, file parse, decoration build) fails.
#[mutants::skip] // I/O orchestration — not unit-testable.
fn init_and_run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    file_path: Option<PathBuf>,
    read_only: bool,
) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
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
    let file_mode = match &file_path {
        Some(p) => resolve_file_mode(p, &config.filetype),
        // Untitled buffers default to Markdown; resolved again on first save-as.
        None => yame::app::FileMode::Markdown,
    };

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

    app.read_only = read_only;

    if read_only {
        app.status
            .set_dismissible("Read-only mode — editing disabled  [any key to dismiss]");
    } else if !italic_support {
        app.status.set_dismissible(
            "⚠ Terminal does not support italics — using color fallback  [any key to dismiss]",
        );
    }

    input::event_loop(terminal, &mut app, &config.layout)
}

#[mutants::skip] // Entry point — calls process::exit, not unit-testable.
fn main() {
    let command = cli::parse_args().unwrap_or_else(|_| std::process::exit(1));
    match command {
        cli::Command::Edit { path, read_only } => {
            if let Err(e) = run(path, read_only) {
                // path: Option<PathBuf>
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
        cli::Command::Preview { path } => {
            if let Err(e) = preview::render_preview(&path) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
