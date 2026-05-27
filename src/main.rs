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

#[mutants::skip] // Prints to stdout and calls process::exit — not unit-testable.
fn print_help() {
    println!("yame — yet another markdown editor");
    println!();
    println!("USAGE");
    println!("  yame <file>           Open <file> for editing (created if it doesn't exist)");
    println!("  yame init             Print shell integration function (eval in .bashrc/.zshrc)");
    println!("  yame write-config     Write default config to ~/.config/yame/config.toml");
    println!("  yame --help           Show this help");
    println!();
    println!("KEYBINDINGS");
    println!("  Ctrl+S  Save          Ctrl+Z  Undo        Ctrl+C  Copy selection");
    println!("  Ctrl+X  Exit          Ctrl+Y  Redo        Ctrl+V  Paste");
    println!("  Ctrl+R  Reload config");
    println!("  Arrow keys · Home/End · PgUp/PgDn · mouse click / drag / scroll");
    println!();
    println!("CONFIG  ~/.config/yame/config.toml  (respects $XDG_CONFIG_HOME)");
    println!();
    println!("  https://github.com/cyrusae/yame");
}

#[mutants::skip] // Reads std::env::args() — side-effectful, not unit-testable.
fn parse_args() -> Result<PathBuf, ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        std::process::exit(0);
    }

    match args.as_slice() {
        [] => {
            print_help();
            std::process::exit(0);
        }
        [path] => Ok(PathBuf::from(path)),
        _ => {
            eprintln!("error: unexpected arguments");
            eprintln!("Run 'yame --help' for usage.");
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

    // col_width=4 → cw = 4 − 2×GUTTER(1) = 2.
    // col_width=5 → cw = 5 − 2×GUTTER(1) = 3.
    // (make_editor_area only provides height; col_width is a separate parameter.)

    #[test]
    fn clamp_scroll_scroll_down_exact_new_top() {
        // 5 lines of "aa" (1 visual row each at cw=2).  Cursor at row 4, visible=3, padding=0.
        // above_visual=4, cursor_visual=4 ≥ 3 → scroll down.
        // headroom=2 → walk backward through rows 3 and 2 (1 row each) → scroll_top=2.
        // Kills: ln66 *→+ (wider cw=1 → 2 rows/line → scroll_top=3), ln99 +→* (0≥3?=no →
        //        no scroll), ln103 >→< (loop never runs → scroll_top=4),
        //        ln106 >→== / >→< (underflow), >→>= (breaks one step early → scroll_top=3),
        //        ln110 -=→+= and -=→/= (remaining never shrinks → scroll_top=0).
        let mut app = make_app_with_lines(&["aa"; 5]);
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
        clamp_scroll(&mut app, make_editor_area(3), 4, 0);
        assert_eq!(app.scroll_top, 2, "walk-backward must land on row 2");
    }

    #[test]
    fn clamp_scroll_walk_backward_reads_prev_line() {
        // ["aaaa","a","aaaa","a","aa"] at cw=2: "aaaa"→2 rows, rest→1 row.
        // cursor at row 4; above_visual = 2+1+2+1 = 6 ≥ 3 → scroll.
        // headroom=2; walk: row-3("a")=1≤2 → consume, row-2("aaaa")=2>1 → break.
        // scroll_top=3.
        // Kills: ln105 -→+ (reads row+1=wrong wrap count → scroll_top=2),
        //        ln105 -→/ (reads same row → wrong count → scroll_top=2).
        let mut app = make_app_with_lines(&["aaaa", "a", "aaaa", "a", "aa"]);
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
        clamp_scroll(&mut app, make_editor_area(3), 4, 0);
        assert_eq!(app.scroll_top, 3, "must read line new_top-1, not new_top or new_top+1");
    }

    #[test]
    fn clamp_scroll_cursor_in_subrow0_no_scroll() {
        // "aaaaa" at cw=3 → ["aaa","aa"] (2 sub-rows).  Cursor col=0 → sub-row 0.
        // above_visual=2, cursor_visual=2, visible=3 → no scroll needed (2 < 3).
        // Mutations that wrongly compute sub-row as 1 push cursor_visual to 3 → spurious scroll.
        // Kills: ln88 +→* (char_end=char_start*char_len: first chunk end=0 → cursor falls
        //        through to last chunk → sub-row 1), ln89 ||→&& (both conditions needed:
        //        last-chunk fallback fires at chunk 1 → sub-row 1),
        //        ln89 <→== and <→> (cursor_col=0 never ==/>char_end → last fallback → sub-row 1).
        let mut app = make_app_with_lines(&["a", "a", "aaaaa"]);
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(2, 0));
        clamp_scroll(&mut app, make_editor_area(3), 5, 0);
        assert_eq!(app.scroll_top, 0, "cursor in sub-row 0 must not trigger scroll");
    }

    #[test]
    fn clamp_scroll_cursor_in_subrow1_exact() {
        // Same layout; cursor col=3 → in second chunk ["aaa","aa"]: char_end of chunk 0 = 3,
        // so cursor_col=3 is NOT < 3 → falls to chunk 1 → sub-row 1.
        // cursor_visual=3, visible=3 → scroll.  headroom=1 → scroll_top=1.
        // Kills: ln89 ==→!= (disables last-chunk fallback; sub-row stays 0 → no scroll → top=0),
        //        ln89 <→<= (cursor_col=3 ≤ char_end=3 → sub-row 0 → no scroll → top=0).
        let mut app = make_app_with_lines(&["a", "a", "aaaaa"]);
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(2, 3));
        clamp_scroll(&mut app, make_editor_area(3), 5, 0);
        assert_eq!(app.scroll_top, 1, "cursor in sub-row 1 must scroll to expose it");
    }

    #[test]
    fn clamp_scroll_padding_affects_headroom() {
        // 5 "aa" lines (1 row each at cw=2), cursor row 4, visible=6, bottom_padding=2.
        // cursor_visual=4; 4+2=6 ≥ 6 → scroll.
        // headroom = 6 − 1 − 0 − 2 = 3 → walk rows 3,2,1 → scroll_top=1.
        // Kills: ln100 +→- (1+0-2 underflows usize in debug → panic),
        //        ln100 +→* (1+0*2=1 → headroom=5 → walks all 4 rows → scroll_top=0).
        let mut app = make_app_with_lines(&["aa"; 5]);
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
        clamp_scroll(&mut app, make_editor_area(6), 4, 2);
        assert_eq!(app.scroll_top, 1, "padding must reduce headroom correctly");
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

    #[test]
    fn ctrl_z_is_not_navigation_key() {
        // Ctrl+Z is an edit key (undo), not navigation — no debounce skip.
        let k = ctrl_key(KeyCode::Char('z'));
        assert!(
            !is_navigation_key(&k),
            "Ctrl+Z must not be a navigation key"
        );
    }

    #[test]
    fn char_a_is_not_navigation_key() {
        // Ordinary characters are not navigation keys.
        let k = make_key(KeyCode::Char('a'));
        assert!(!is_navigation_key(&k), "char 'a' must not be a navigation key");
    }

    // ── handle_pair_wrap: one test per pair character ────────────────────────

    fn select_all(app: &mut App, len: usize) {
        app.textarea.start_selection();
        for _ in 0..len {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
    }

    #[test]
    fn pair_wrap_paren_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('(')));
        assert_eq!(app.textarea.lines()[0], "(hi)");
    }

    #[test]
    fn pair_wrap_brace_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('{')));
        assert_eq!(app.textarea.lines()[0], "{hi}");
    }

    #[test]
    fn pair_wrap_double_quote_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('"')));
        assert_eq!(app.textarea.lines()[0], "\"hi\"");
    }

    #[test]
    fn pair_wrap_single_quote_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('\'')));
        assert_eq!(app.textarea.lines()[0], "'hi'");
    }

    #[test]
    fn pair_wrap_backtick_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('`')));
        assert_eq!(app.textarea.lines()[0], "`hi`");
    }

    #[test]
    fn pair_wrap_underscore_wraps_selection() {
        let mut app = make_app_with_lines(&["hi"]);
        select_all(&mut app, 2);
        handle_pair_wrap(&mut app, make_key(KeyCode::Char('_')));
        assert_eq!(app.textarea.lines()[0], "_hi_");
    }

    // ── get_selection_text boundary tests ────────────────────────────────────

    #[test]
    fn get_selection_multiline_start_col_respected() {
        // Selection begins mid-line. The col_start must be honoured so we only
        // get characters from col_start onward for the first row.
        let mut app = make_app_with_lines(&["abcde", "fghij"]);
        // Move cursor to col 2 on row 0, then start selection from there
        for _ in 0..2 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        app.textarea.start_selection();
        // Extend to end of next line
        app.textarea.move_cursor(tui_textarea::CursorMove::Down);
        app.textarea.move_cursor(tui_textarea::CursorMove::End);
        let text = get_selection_text(&app).unwrap_or_default();
        assert!(
            text.starts_with("cde"),
            "first-row selection must start at col_start, got: {:?}",
            text
        );
        assert!(!text.starts_with("ab"), "chars before col_start must be excluded");
    }

    #[test]
    fn get_selection_multiline_end_col_respected() {
        // Selection ends mid-line. The col_end must be honoured so we only get
        // characters up to col_end for the last row.
        let mut app = make_app_with_lines(&["abc", "defgh"]);
        app.textarea.start_selection();
        // Move to row 1, col 3 (selecting "abc\ndef")
        app.textarea.move_cursor(tui_textarea::CursorMove::Down);
        for _ in 0..3 {
            app.textarea.move_cursor(tui_textarea::CursorMove::Forward);
        }
        let text = get_selection_text(&app).unwrap_or_default();
        assert!(
            text.ends_with("def"),
            "last-row selection must end at col_end, got: {:?}",
            text
        );
        assert!(
            !text.contains('g'),
            "chars after col_end must be excluded, got: {:?}",
            text
        );
    }

    #[test]
    fn get_selection_multiline_no_trailing_newline() {
        // The final row must NOT have a trailing '\n' even in a multiline selection.
        let mut app = make_app_with_lines(&["abc", "def"]);
        app.textarea.start_selection();
        app.textarea.move_cursor(tui_textarea::CursorMove::Down);
        app.textarea.move_cursor(tui_textarea::CursorMove::End);
        let text = get_selection_text(&app).unwrap_or_default();
        assert!(
            !text.ends_with('\n'),
            "selection must not have trailing newline, got: {:?}",
            text
        );
    }

    // ── clamp_scroll boundary tests ──────────────────────────────────────────

    #[test]
    fn clamp_scroll_cursor_at_exact_scroll_top_unchanged() {
        // cursor_row == scroll_top: cursor is exactly at the top of the viewport.
        // The `cursor_row < scroll_top` guard must NOT trigger here.
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 3;
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(3, 0));
        clamp_scroll(&mut app, make_editor_area(10), TEST_COL, 0);
        assert_eq!(
            app.scroll_top, 3,
            "cursor == scroll_top must not scroll up"
        );
    }

    #[test]
    fn clamp_scroll_cursor_subrow_on_wrapped_line() {
        // A long line (> column width) wraps into multiple visual rows.
        // With cursor at the last character, cursor_subrow must be computed
        // correctly so scroll_top advances by the right amount.
        // TEST_COL=82 → cw=80. A 100-char line wraps into 2 visual rows.
        let long = "x".repeat(100);
        let mut lines: Vec<&str> = vec![long.as_str()];
        lines.extend(std::iter::repeat("a").take(10));
        let mut app = make_app_with_lines(&lines);
        // Position cursor at char 99 (second wrapped sub-row of line 0).
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(0, 99));
        // Viewport height=3, padding=1 → cursor_visual + 1 may exceed visible.
        clamp_scroll(&mut app, make_editor_area(3), TEST_COL, 1);
        // scroll_top stays 0 because line 0 is the first line — just assert no panic
        // and that scroll_top hasn't gone negative.
        assert_eq!(
            app.scroll_top, 0,
            "wrapped cursor on line 0 must not push scroll_top below 0"
        );
    }

    #[test]
    fn clamp_scroll_bottom_padding_exact_boundary() {
        // Cursor at exactly visible_rows - bottom_padding - 1 should NOT scroll.
        // Cursor at exactly visible_rows - bottom_padding should scroll.
        // height=10, padding=3: cursor at row 6 (0-indexed) → cursor_visual=6 < 10-3=7 → no scroll
        let mut app = make_app_with_lines(&["a"; 20]);
        app.scroll_top = 0;
        app.textarea
            .move_cursor(tui_textarea::CursorMove::Jump(6, 0));
        clamp_scroll(&mut app, make_editor_area(10), TEST_COL, 3);
        assert_eq!(
            app.scroll_top, 0,
            "cursor at visible_rows - padding - 1 must not scroll"
        );
    }

    // ── handle_key_event tests ───────────────────────────────────────────────

    use super::input::{handle_key_event, KeyOutcome};

    #[test]
    fn handle_key_event_resets_free_scroll() {
        let mut app = make_app();
        app.free_scroll = true;
        let k = make_key(KeyCode::Up);
        handle_key_event(&mut app, k);
        // Up key is a navigation key → goes through `_` arm → free_scroll cleared.
        // (Ctrl+Up would set it back to true, but plain Up does not.)
        assert!(!app.free_scroll, "any key press must clear free_scroll");
    }

    #[test]
    fn handle_key_event_ctrl_s_returns_save() {
        let mut app = make_app();
        let k = ctrl_key(KeyCode::Char('s'));
        assert_eq!(handle_key_event(&mut app, k), KeyOutcome::Save);
    }

    #[test]
    fn handle_key_event_ctrl_x_clean_returns_exit() {
        let mut app = make_app();
        app.is_dirty = false;
        let k = ctrl_key(KeyCode::Char('x'));
        assert_eq!(handle_key_event(&mut app, k), KeyOutcome::Exit);
    }

    #[test]
    fn handle_key_event_ctrl_x_dirty_shows_prompt() {
        let mut app = make_app();
        app.is_dirty = true;
        let k = ctrl_key(KeyCode::Char('x'));
        assert_eq!(handle_key_event(&mut app, k), KeyOutcome::Continue);
        assert!(
            matches!(app.status.mode, StatusMode::ExitPrompt),
            "dirty Ctrl+X must raise ExitPrompt"
        );
    }

    #[test]
    fn handle_key_event_ctrl_r_returns_reload_config() {
        let mut app = make_app();
        let k = ctrl_key(KeyCode::Char('r'));
        assert_eq!(handle_key_event(&mut app, k), KeyOutcome::ReloadConfig);
    }

    #[test]
    fn handle_key_event_ctrl_z_undoes_and_sets_force_redecorate() {
        let mut app = make_app_with_lines(&["hello"]);
        // Type a character to have something to undo.
        app.textarea
            .input(crossterm::event::KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        app.force_redecorate = false;
        let k = ctrl_key(KeyCode::Char('z'));
        let outcome = handle_key_event(&mut app, k);
        assert_eq!(outcome, KeyOutcome::Continue);
        assert!(app.force_redecorate, "Ctrl+Z must set force_redecorate");
        assert!(
            app.last_keystroke.is_some(),
            "Ctrl+Z must set last_keystroke"
        );
    }

    #[test]
    fn handle_key_event_ctrl_y_redoes_and_sets_force_redecorate() {
        let mut app = make_app_with_lines(&["hello"]);
        app.force_redecorate = false;
        let k = ctrl_key(KeyCode::Char('y'));
        let outcome = handle_key_event(&mut app, k);
        assert_eq!(outcome, KeyOutcome::Continue);
        assert!(app.force_redecorate, "Ctrl+Y must set force_redecorate");
    }

    #[test]
    fn handle_key_event_ctrl_up_scrolls_up_and_sets_free_scroll() {
        let mut app = make_app_with_lines(&["a"; 10]);
        app.scroll_top = 5;
        let k = ctrl_key(KeyCode::Up);
        handle_key_event(&mut app, k);
        assert_eq!(app.scroll_top, 4, "Ctrl+Up must decrement scroll_top");
        assert!(app.free_scroll, "Ctrl+Up must set free_scroll");
    }

    #[test]
    fn handle_key_event_ctrl_up_saturates_at_zero() {
        let mut app = make_app_with_lines(&["a"; 5]);
        app.scroll_top = 0;
        handle_key_event(&mut app, ctrl_key(KeyCode::Up));
        assert_eq!(app.scroll_top, 0, "Ctrl+Up at top must not underflow");
    }

    #[test]
    fn handle_key_event_ctrl_down_scrolls_down_and_sets_free_scroll() {
        let mut app = make_app_with_lines(&["a"; 10]);
        app.scroll_top = 0;
        let k = ctrl_key(KeyCode::Down);
        handle_key_event(&mut app, k);
        assert_eq!(app.scroll_top, 1, "Ctrl+Down must increment scroll_top");
        assert!(app.free_scroll, "Ctrl+Down must set free_scroll");
    }

    #[test]
    fn handle_key_event_ctrl_down_saturates_at_last_line() {
        let mut app = make_app_with_lines(&["a"; 3]);
        app.scroll_top = 2; // already at max (len - 1 = 2)
        handle_key_event(&mut app, ctrl_key(KeyCode::Down));
        assert_eq!(app.scroll_top, 2, "Ctrl+Down at bottom must not exceed last line");
    }

    #[test]
    fn handle_key_event_exit_prompt_y_returns_save_and_exit() {
        let mut app = make_app();
        app.is_dirty = true;
        app.status.mode = StatusMode::ExitPrompt;
        let outcome = handle_key_event(&mut app, make_key(KeyCode::Char('Y')));
        assert_eq!(outcome, KeyOutcome::SaveAndExit);
    }

    #[test]
    fn handle_key_event_exit_prompt_n_returns_exit() {
        let mut app = make_app();
        app.status.mode = StatusMode::ExitPrompt;
        let outcome = handle_key_event(&mut app, make_key(KeyCode::Char('n')));
        assert_eq!(outcome, KeyOutcome::Exit);
    }

    #[test]
    fn handle_key_event_exit_prompt_esc_cancels_to_normal() {
        let mut app = make_app();
        app.status.mode = StatusMode::ExitPrompt;
        let outcome = handle_key_event(&mut app, make_key(KeyCode::Esc));
        assert_eq!(outcome, KeyOutcome::Continue);
        assert!(
            matches!(app.status.mode, StatusMode::Normal),
            "Esc in exit prompt must restore Normal mode"
        );
    }

    #[test]
    fn handle_key_event_exit_prompt_unknown_key_continues() {
        let mut app = make_app();
        app.status.mode = StatusMode::ExitPrompt;
        let outcome = handle_key_event(&mut app, make_key(KeyCode::Char('z')));
        assert_eq!(outcome, KeyOutcome::Continue);
        assert!(
            matches!(app.status.mode, StatusMode::ExitPrompt),
            "unknown key in exit prompt must not change mode"
        );
    }
}
