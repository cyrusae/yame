/// Binary-crate unit tests.
///
/// Declared in `main.rs` as `#[cfg(test)] mod tests;` so that private items
/// from `commands`, `input`, and `cli` are all in scope via `super::`.
use std::path::PathBuf;
use tui_textarea::TextArea;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use yame::app::App;
use yame::config::Theme;
use yame::decoration::DecorationMap;
use yame::status::{StatusLine, StatusMode};

use super::commands::{clamp_scroll, handle_exit};
use super::input::{handle_goto_line_key, handle_pair_wrap};
use yame::app::get_selection_text;

fn make_app() -> App {
    App {
        textarea: TextArea::default(),
        file_path: Some(PathBuf::from("test.md")),
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
        sticky_col: None,
        content_width: 0,
        clipboard: yame::app::ClipboardState::Uninitialized,
        shortened_path: "test.md".to_string(),
        tab_width: 4,
        highlight_cache: None,
        file_mode: yame::app::FileMode::Markdown,
        show_line_numbers: false,
        search: None,
        typewriter_mode: false,
        focus_mode: false,
        show_shortcuts: false,
        read_only: false,
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
// TEST_COL=82 → content_width = 82 - GUTTER - GUTTER = 80.
// make_app_with_lines pre-sets content_width=80 to match TEST_COL-based tests.
// Tests that exercise wrapping at a narrower width override content_width inline.
const TEST_COL: u16 = 82;
const TEST_CW: usize = 80; // TEST_COL - 2*GUTTER

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
    app.content_width = TEST_CW;
    app
}

#[test]
fn clamp_scroll_cursor_above_viewport_scrolls_up() {
    let mut app = make_app_with_lines(&["a"; 20]);
    app.scroll_top = 5;
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(2, 0));
    clamp_scroll(&mut app, make_editor_area(10), 0);
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
    clamp_scroll(&mut app, make_editor_area(10), 0);
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
    clamp_scroll(&mut app, make_editor_area(10), 3);
    assert!(
        app.scroll_top > 0,
        "cursor near bottom with padding → scroll_top advances"
    );
}

#[test]
fn clamp_scroll_zero_height_does_not_panic() {
    let mut app = make_app_with_lines(&["a"; 5]);
    clamp_scroll(&mut app, make_editor_area(0), 3);
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
    app.content_width = 2; // col_width=4 - 2*GUTTER = 2: "aa" fills one row, no wrap
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
    clamp_scroll(&mut app, make_editor_area(3), 0);
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
    app.content_width = 2; // col_width=4 - 2*GUTTER = 2: "aaaa" wraps to 2 rows
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
    clamp_scroll(&mut app, make_editor_area(3), 0);
    assert_eq!(
        app.scroll_top, 3,
        "must read line new_top-1, not new_top or new_top+1"
    );
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
    app.content_width = 3; // col_width=5 - 2*GUTTER = 3: "aaaaa" wraps to ["aaa","aa"]
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(2, 0));
    clamp_scroll(&mut app, make_editor_area(3), 0);
    assert_eq!(
        app.scroll_top, 0,
        "cursor in sub-row 0 must not trigger scroll"
    );
}

#[test]
fn clamp_scroll_cursor_in_subrow1_exact() {
    // Same layout; cursor col=3 → in second chunk ["aaa","aa"]: char_end of chunk 0 = 3,
    // so cursor_col=3 is NOT < 3 → falls to chunk 1 → sub-row 1.
    // cursor_visual=3, visible=3 → scroll.  headroom=1 → scroll_top=1.
    // Kills: ln89 ==→!= (disables last-chunk fallback; sub-row stays 0 → no scroll → top=0),
    //        ln89 <→<= (cursor_col=3 ≤ char_end=3 → sub-row 0 → no scroll → top=0).
    let mut app = make_app_with_lines(&["a", "a", "aaaaa"]);
    app.content_width = 3; // col_width=5 - 2*GUTTER = 3: "aaaaa" wraps to ["aaa","aa"]
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(2, 3));
    clamp_scroll(&mut app, make_editor_area(3), 0);
    assert_eq!(
        app.scroll_top, 1,
        "cursor in sub-row 1 must scroll to expose it"
    );
}

#[test]
fn clamp_scroll_padding_affects_headroom() {
    // 5 "aa" lines (1 row each at cw=2), cursor row 4, visible=6, bottom_padding=2.
    // cursor_visual=4; 4+2=6 ≥ 6 → scroll.
    // headroom = 6 − 1 − 0 − 2 = 3 → walk rows 3,2,1 → scroll_top=1.
    // Kills: ln100 +→- (1+0-2 underflows usize in debug → panic),
    //        ln100 +→* (1+0*2=1 → headroom=5 → walks all 4 rows → scroll_top=0).
    let mut app = make_app_with_lines(&["aa"; 5]);
    app.content_width = 2; // col_width=4 - 2*GUTTER = 2
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(4, 0));
    clamp_scroll(&mut app, make_editor_area(6), 2);
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
    assert!(
        !is_navigation_key(&k),
        "char 'a' must not be a navigation key"
    );
}

// ── shell_init_str tests ──────────────────────────────────────────────────

#[test]
fn shell_init_str_contains_function_declaration() {
    let s = super::cli::shell_init_str("bash");
    assert!(
        s.contains("yame()"),
        "output must declare a yame() function"
    );
}

#[test]
fn shell_init_str_contains_fd_fzf_guard() {
    let s = super::cli::shell_init_str("bash");
    assert!(
        s.contains("command -v fd") && s.contains("command -v fzf"),
        "output must guard on fd and fzf availability"
    );
}

#[test]
fn shell_init_str_uses_command_yame_passthrough() {
    let s = super::cli::shell_init_str("bash");
    assert!(
        s.contains("command yame"),
        "output must use 'command yame' to bypass the wrapper"
    );
}

#[test]
fn shell_init_str_tier2_includes_hidden_flag() {
    let s = super::cli::shell_init_str("zsh");
    assert!(
        s.contains("--hidden"),
        "tier-2 search must include --hidden so dotfiles can be found"
    );
}

#[test]
fn shell_init_str_excludes_correct_target_dir() {
    // Must use -E "target" (correct), not -E "target/*" (wrong glob).
    let s = super::cli::shell_init_str("bash");
    assert!(
        s.contains(r#"-E "target""#),
        r#"must exclude target directory with -E "target""#
    );
    assert!(
        !s.contains(r#"-E "target/*""#),
        r#"must NOT use the wrong glob form -E "target/*""#
    );
}

#[test]
fn shell_init_str_fallback_prompts_before_creating() {
    let s = super::cli::shell_init_str("zsh");
    assert!(
        s.contains("printf"),
        "fallback must display a prompt before creating a new file"
    );
    assert!(
        s.contains("[y/N]"),
        "fallback prompt must show [y/N] confirmation"
    );
}

#[test]
fn shell_init_str_bash_and_zsh_produce_same_output() {
    assert_eq!(
        super::cli::shell_init_str("bash"),
        super::cli::shell_init_str("zsh"),
        "bash and zsh init output must be identical (single function body)"
    );
}

// ── version_string tests ──────────────────────────────────────────────────

#[test]
fn version_string_starts_with_yame() {
    let v = super::cli::version_string();
    assert!(
        v.starts_with("yame "),
        "version string must start with 'yame '"
    );
}

#[test]
fn version_string_contains_semver() {
    let v = super::cli::version_string();
    // Must contain at least one dot (e.g. "0.1.0") after the "yame " prefix.
    let ver = v.strip_prefix("yame ").expect("must start with 'yame '");
    assert!(
        ver.contains('.'),
        "version portion must be a semver string, got: {ver:?}"
    );
}

#[test]
fn version_string_matches_cargo_pkg_version() {
    let v = super::cli::version_string();
    let expected = format!("yame {}", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        v, expected,
        "version_string() must equal 'yame {{CARGO_PKG_VERSION}}'"
    );
}

// ── typewriter mode tests ─────────────────────────────────────────────────

#[test]
fn typewriter_mode_off_by_default() {
    let app = make_app();
    assert!(
        !app.typewriter_mode,
        "typewriter_mode must default to false"
    );
}

#[test]
fn ctrl_t_toggles_typewriter_mode_on() {
    use super::input::{KeyOutcome, handle_key_event};
    let mut app = make_app();
    let outcome = handle_key_event(&mut app, ctrl(KeyCode::Char('t')));
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(app.typewriter_mode, "Ctrl+T must enable typewriter mode");
}

#[test]
fn ctrl_t_toggles_typewriter_mode_off() {
    use super::input::handle_key_event;
    let mut app = make_app();
    app.typewriter_mode = true;
    handle_key_event(&mut app, ctrl(KeyCode::Char('t')));
    assert!(
        !app.typewriter_mode,
        "second Ctrl+T must disable typewriter mode"
    );
}

#[test]
fn center_scroll_places_cursor_at_midpoint() {
    use super::commands::center_scroll;
    let mut app = make_app_with_lines(&["a"; 30]);
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(20, 0));
    center_scroll(&mut app, make_editor_area(10));
    // cursor_row=20, half=5 → scroll_top should be 15
    assert_eq!(
        app.scroll_top, 15,
        "scroll_top must be cursor_row - viewport_height/2"
    );
}

#[test]
fn center_scroll_clamps_to_zero_near_top() {
    use super::commands::center_scroll;
    let mut app = make_app_with_lines(&["a"; 30]);
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(2, 0));
    center_scroll(&mut app, make_editor_area(10));
    // cursor_row=2, half=5 → saturating_sub → 0
    assert_eq!(app.scroll_top, 0, "scroll_top must not underflow past 0");
}

#[test]
fn center_scroll_exact_midpoint_cursor() {
    use super::commands::center_scroll;
    let mut app = make_app_with_lines(&["a"; 20]);
    app.textarea
        .move_cursor(tui_textarea::CursorMove::Jump(5, 0));
    center_scroll(&mut app, make_editor_area(10));
    // cursor_row=5, half=5 → scroll_top=0
    assert_eq!(app.scroll_top, 0);
}

// ── go-to-line tests ──────────────────────────────────────────────────────

fn key(code: KeyCode) -> crossterm::event::KeyEvent {
    crossterm::event::KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> crossterm::event::KeyEvent {
    crossterm::event::KeyEvent::new(code, KeyModifiers::CONTROL)
}

#[test]
fn goto_line_ctrl_g_enters_mode() {
    use super::input::{KeyOutcome, handle_key_event};
    let mut app = make_app();
    let outcome = handle_key_event(&mut app, ctrl(KeyCode::Char('g')));
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(
        matches!(app.status.mode, StatusMode::GoToLine { .. }),
        "Ctrl+G must enter GoToLine mode"
    );
}

#[test]
fn goto_line_digits_appended_to_buffer() {
    let mut app = make_app();
    app.status.start_goto_line();
    handle_goto_line_key(&mut app, key(KeyCode::Char('4')));
    handle_goto_line_key(&mut app, key(KeyCode::Char('2')));
    assert_eq!(app.status.goto_input(), Some("42"));
}

#[test]
fn goto_line_non_digits_ignored() {
    let mut app = make_app();
    app.status.start_goto_line();
    handle_goto_line_key(&mut app, key(KeyCode::Char('a')));
    handle_goto_line_key(&mut app, key(KeyCode::Char('!')));
    assert_eq!(
        app.status.goto_input(),
        Some(""),
        "letters and symbols must be ignored"
    );
}

#[test]
fn goto_line_backspace_removes_last_digit() {
    let mut app = make_app();
    app.status.start_goto_line();
    handle_goto_line_key(&mut app, key(KeyCode::Char('1')));
    handle_goto_line_key(&mut app, key(KeyCode::Char('2')));
    handle_goto_line_key(&mut app, key(KeyCode::Backspace));
    assert_eq!(app.status.goto_input(), Some("1"));
}

#[test]
fn goto_line_esc_cancels_without_jump() {
    let mut app = make_app();
    app.status.start_goto_line();
    handle_goto_line_key(&mut app, key(KeyCode::Char('1')));
    let (row_before, _) = app.textarea.cursor();
    handle_goto_line_key(&mut app, key(KeyCode::Esc));
    let (row_after, _) = app.textarea.cursor();
    assert!(
        matches!(app.status.mode, StatusMode::Normal),
        "Esc must return to Normal mode"
    );
    assert_eq!(row_before, row_after, "Esc must not move the cursor");
}

#[test]
fn goto_line_enter_with_empty_input_is_noop() {
    let mut app = make_app();
    app.status.start_goto_line();
    let (row_before, _) = app.textarea.cursor();
    handle_goto_line_key(&mut app, key(KeyCode::Enter));
    let (row_after, _) = app.textarea.cursor();
    assert!(matches!(app.status.mode, StatusMode::Normal));
    assert_eq!(
        row_before, row_after,
        "empty input must not move the cursor"
    );
}

#[test]
fn goto_line_enter_jumps_to_correct_line() {
    let mut app = make_app_with_lines(&["line 1", "line 2", "line 3", "line 4", "line 5"]);
    app.status.start_goto_line();
    handle_goto_line_key(&mut app, key(KeyCode::Char('3')));
    handle_goto_line_key(&mut app, key(KeyCode::Enter));
    let (row, col) = app.textarea.cursor();
    assert_eq!(row, 2, "line 3 (1-indexed) == row 2 (0-indexed)");
    assert_eq!(col, 0, "cursor must land at column 0");
    assert!(matches!(app.status.mode, StatusMode::Normal));
}

#[test]
fn goto_line_enter_clamps_to_last_line() {
    let mut app = make_app_with_lines(&["only", "two"]);
    app.status.start_goto_line();
    for c in "9999".chars() {
        handle_goto_line_key(&mut app, key(KeyCode::Char(c)));
    }
    handle_goto_line_key(&mut app, key(KeyCode::Enter));
    let (row, _) = app.textarea.cursor();
    assert_eq!(
        row, 1,
        "line number beyond doc length must clamp to last line"
    );
}

#[test]
fn goto_line_input_capped_at_seven_digits() {
    let mut app = make_app();
    app.status.start_goto_line();
    for c in "12345678".chars() {
        handle_goto_line_key(&mut app, key(KeyCode::Char(c)));
    }
    assert_eq!(
        app.status.goto_input().map(|s| s.len()),
        Some(7),
        "input buffer must be capped at 7 digits"
    );
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
    assert!(
        !text.starts_with("ab"),
        "chars before col_start must be excluded"
    );
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
    clamp_scroll(&mut app, make_editor_area(10), 0);
    assert_eq!(app.scroll_top, 3, "cursor == scroll_top must not scroll up");
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
    clamp_scroll(&mut app, make_editor_area(3), 1);
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
    clamp_scroll(&mut app, make_editor_area(10), 3);
    assert_eq!(
        app.scroll_top, 0,
        "cursor at visible_rows - padding - 1 must not scroll"
    );
}

// ── handle_key_event tests ───────────────────────────────────────────────

use super::input::{KeyOutcome, handle_key_event};

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
    app.textarea.input(crossterm::event::KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
    ));
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
fn handle_key_event_cmd_z_undoes_and_sets_force_redecorate() {
    let mut app = make_app_with_lines(&["hello"]);
    app.textarea.input(crossterm::event::KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::NONE,
    ));
    app.force_redecorate = false;
    let k = crossterm::event::KeyEvent::new(KeyCode::Char('z'), KeyModifiers::SUPER);
    let outcome = handle_key_event(&mut app, k);
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(app.force_redecorate, "Cmd+Z must set force_redecorate");
    assert!(
        app.last_keystroke.is_some(),
        "Cmd+Z must set last_keystroke"
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
fn handle_key_event_cmd_y_redoes_and_sets_force_redecorate() {
    let mut app = make_app_with_lines(&["hello"]);
    app.force_redecorate = false;
    let k = crossterm::event::KeyEvent::new(KeyCode::Char('y'), KeyModifiers::SUPER);
    let outcome = handle_key_event(&mut app, k);
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(app.force_redecorate, "Cmd+Y must set force_redecorate");
}

#[test]
fn handle_key_event_cmd_shift_z_redoes_and_sets_force_redecorate() {
    let mut app = make_app_with_lines(&["hello"]);
    app.force_redecorate = false;
    let k = crossterm::event::KeyEvent::new(
        KeyCode::Char('z'),
        KeyModifiers::SUPER | KeyModifiers::SHIFT,
    );
    let outcome = handle_key_event(&mut app, k);
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(
        app.force_redecorate,
        "Cmd+Shift+Z must set force_redecorate"
    );
}

// When Kitty's keyboard protocol is active (leaked from a previous app that did
// not restore terminal state), the terminal emits Release events in addition to
// Press events.  A Release of 'z' after releasing Ctrl early — i.e. the key
// arrives as (NONE, Char('z'), kind=Release) — must NOT insert a character.
// The event_loop guards against this at dispatch time; this test confirms that
// handle_key_event itself also does the right thing if it ever sees such an event
// (it falls to the `_` arm → textarea.input → tui-textarea's own Release guard).
#[test]
fn release_event_for_bare_z_does_not_insert_character() {
    use crossterm::event::KeyEventKind;
    let mut app = make_app_with_lines(&["hello"]);
    let content_before: Vec<String> = app.textarea.lines().to_vec();

    let k = crossterm::event::KeyEvent::new_with_kind(
        KeyCode::Char('z'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );
    let outcome = handle_key_event(&mut app, k);
    assert_eq!(outcome, KeyOutcome::Continue);
    assert_eq!(
        app.textarea.lines().to_vec(),
        content_before,
        "Release event for bare 'z' must not modify textarea content"
    );
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
    assert_eq!(
        app.scroll_top, 2,
        "Ctrl+Down at bottom must not exceed last line"
    );
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

// ── focus mode: paragraph-bounds logic ───────────────────────────────────────

#[test]
fn focus_paragraph_bounds_no_blank_lines_spans_whole_doc() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["alpha".into(), "beta".into(), "gamma".into()];
    // Cursor anywhere in a single-paragraph doc → whole document range.
    assert_eq!(focus_paragraph_bounds(&lines, 1), (0, 2));
}

#[test]
fn focus_paragraph_bounds_cursor_on_blank_line_is_self() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["alpha".into(), "".into(), "gamma".into()];
    // Blank line → single-line "paragraph".
    assert_eq!(focus_paragraph_bounds(&lines, 1), (1, 1));
}

#[test]
fn focus_paragraph_bounds_first_paragraph() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["hello".into(), "world".into(), "".into(), "other".into()];
    // Cursor on line 0 → paragraph (0, 1).
    assert_eq!(focus_paragraph_bounds(&lines, 0), (0, 1));
}

#[test]
fn focus_paragraph_bounds_second_paragraph() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["hello".into(), "".into(), "world".into(), "foo".into()];
    // Cursor on line 2 → paragraph (2, 3).
    assert_eq!(focus_paragraph_bounds(&lines, 2), (2, 3));
}

#[test]
fn focus_paragraph_bounds_middle_paragraph() {
    use yame::renderer::focus_paragraph_bounds;
    // "a\n\nb\nc\nd\n\ne"
    let lines: Vec<String> = vec![
        "a".into(),
        "".into(),
        "b".into(),
        "c".into(),
        "d".into(),
        "".into(),
        "e".into(),
    ];
    // Cursor at line 3 (inside "b c d" block) → (2, 4).
    assert_eq!(focus_paragraph_bounds(&lines, 3), (2, 4));
}

#[test]
fn focus_paragraph_bounds_empty_doc_returns_zero_zero() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec![];
    assert_eq!(focus_paragraph_bounds(&lines, 0), (0, 0));
}

#[test]
fn focus_paragraph_bounds_cursor_oob_clamps_to_last_line() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["a".into(), "b".into()];
    // cursor_row=99 → clamped to 1 → whole doc (0, 1).
    assert_eq!(focus_paragraph_bounds(&lines, 99), (0, 1));
}

#[test]
fn focus_paragraph_bounds_single_line_doc() {
    use yame::renderer::focus_paragraph_bounds;
    let lines: Vec<String> = vec!["only".into()];
    assert_eq!(focus_paragraph_bounds(&lines, 0), (0, 0));
}

// ── focus mode: toggle keybinding ────────────────────────────────────────────

#[test]
fn focus_mode_off_by_default() {
    let app = make_app();
    assert!(!app.focus_mode, "focus_mode must default to false");
}

#[test]
fn ctrl_d_toggles_focus_mode_on() {
    use super::input::{KeyOutcome, handle_key_event};
    let mut app = make_app();
    let outcome = handle_key_event(&mut app, ctrl(KeyCode::Char('d')));
    assert_eq!(outcome, KeyOutcome::Continue);
    assert!(app.focus_mode, "Ctrl+D must enable focus mode");
}

#[test]
fn ctrl_d_toggles_focus_mode_off() {
    use super::input::handle_key_event;
    let mut app = make_app();
    app.focus_mode = true;
    handle_key_event(&mut app, ctrl(KeyCode::Char('d')));
    assert!(!app.focus_mode, "second Ctrl+D must disable focus mode");
}
