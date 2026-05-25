use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use yame::app::App;
use yame::config::{Theme, load_config, supports_italic};

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

#[mutants::skip] // Reads std::env::args() — side-effectful, not unit-testable.
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
    let powerline_glyphs = config.layout.powerline_glyphs.unwrap_or(false);
    let mut app = App::new(
        file_path,
        theme,
        italic_support,
        powerline_glyphs,
        warnings,
        tab_width,
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
    use std::path::PathBuf;
    use tui_textarea::TextArea;

    use crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::layout::Rect;

    use yame::app::App;
    use yame::config::Theme;
    use yame::decoration::DecorationMap;
    use yame::status::{StatusLine, StatusMode};

    use super::commands::{clamp_scroll, handle_exit};
    use super::input::{get_selection_text, handle_pair_wrap};

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("test.md"),
            is_dirty: false,
            saved_content: None,
            theme: Theme::default_theme(),
            italic_support: false,
            powerline_glyphs: false,
            last_keystroke: None,
            force_redecorate: false,
            decoration_map: DecorationMap::default(),
            word_count: 0,
            status: StatusLine::default(),
            config_warnings: vec![],
            scroll_top: 0,
            free_scroll: false,
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
    // col_width=82 → cw = 82 - 2*GUTTER. GUTTER=1 → cw=80. Using 82 gives cw=80.
    const TEST_COL: u16 = 82;

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
        let mut app = make_app_with_lines(&["a"; 5]);
        clamp_scroll(&mut app, make_editor_area(0), TEST_COL, 3);
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
        app.textarea.start_selection();
        for _ in 0..5 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        assert_eq!(get_selection_text(&app), Some("hello".to_string()));
    }

    #[test]
    fn get_selection_text_multiline() {
        let mut app = make_app_with_lines(&["abc", "def"]);
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
        let k = crossterm::event::KeyEvent::new(KeyCode::Char('['), KeyModifiers::CONTROL);
        let handled = handle_pair_wrap(&mut app, k);
        assert!(!handled, "Ctrl chord must not trigger pair wrap");
    }

    // ── free_scroll / decoupled scroll tests ─────────────────────────────────

    use super::input::is_navigation_key;

    fn ctrl_key(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn scroll_down_increases_scroll_top_without_moving_cursor() {
        // Simulate what the ScrollDown handler does: increment scroll_top, set free_scroll.
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 0;
        let (cursor_before, _) = app.textarea.cursor();
        let max = app.textarea.lines().len().saturating_sub(1);
        app.scroll_top = (app.scroll_top + 3).min(max);
        app.free_scroll = true;
        let (cursor_after, _) = app.textarea.cursor();
        assert_eq!(app.scroll_top, 3, "scroll_top advanced by SCROLL_LINES");
        assert_eq!(
            cursor_before, cursor_after,
            "cursor must not move on scroll"
        );
        assert!(app.free_scroll, "free_scroll must be set");
    }

    #[test]
    fn scroll_up_decreases_scroll_top_without_moving_cursor() {
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 6;
        let (cursor_before, _) = app.textarea.cursor();
        app.scroll_top = app.scroll_top.saturating_sub(3);
        app.free_scroll = true;
        let (cursor_after, _) = app.textarea.cursor();
        assert_eq!(app.scroll_top, 3, "scroll_top decreased by SCROLL_LINES");
        assert_eq!(
            cursor_before, cursor_after,
            "cursor must not move on scroll"
        );
        assert!(app.free_scroll, "free_scroll must be set");
    }

    #[test]
    fn scroll_up_saturates_at_zero() {
        let mut app = make_app_with_lines(&["a"; 10]);
        app.scroll_top = 1;
        app.scroll_top = app.scroll_top.saturating_sub(5); // would go negative
        assert_eq!(app.scroll_top, 0, "scroll_top must not go below 0");
    }

    #[test]
    fn scroll_down_saturates_at_last_line() {
        let mut app = make_app_with_lines(&["a"; 5]);
        let max = app.textarea.lines().len().saturating_sub(1); // = 4
        app.scroll_top = (app.scroll_top + 100).min(max);
        assert_eq!(app.scroll_top, 4, "scroll_top must not exceed last line");
    }

    #[test]
    fn ctrl_up_is_navigation_key() {
        let k = ctrl_key(KeyCode::Up);
        assert!(
            is_navigation_key(&k),
            "Ctrl+Up must be classified as navigation (no debounce)"
        );
    }

    #[test]
    fn ctrl_down_is_navigation_key() {
        let k = ctrl_key(KeyCode::Down);
        assert!(
            is_navigation_key(&k),
            "Ctrl+Down must be classified as navigation (no debounce)"
        );
    }

    #[test]
    fn plain_up_is_navigation_key() {
        // Regression: plain Up must still be navigation after is_navigation_key refactor.
        let k = make_key(KeyCode::Up);
        assert!(
            is_navigation_key(&k),
            "plain Up must remain a navigation key"
        );
    }
}
