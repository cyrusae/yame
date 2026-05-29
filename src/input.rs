use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};
use ratatui::{Terminal, layout::Rect, style::Style, widgets::Paragraph};
use tui_textarea::CursorMove;

use yame::app::App;
use yame::config::{LayoutConfig, Theme, load_config};
use yame::decoration::build_decoration_map;
use yame::layout::{DEFAULT_MIN_COLS, compute_layout};
use yame::renderer;
use yame::status::StatusMode;

use super::commands::{clamp_scroll, handle_exit, handle_save};

// ---------------------------------------------------------------------------
// Key-event outcome
// ---------------------------------------------------------------------------

/// Signals the event loop needs to act on after `handle_key_event` returns.
///
/// Keeping I/O (file saves, config reloads) out of `handle_key_event` makes
/// that function fully unit-testable without a real terminal or filesystem.
#[derive(Debug, PartialEq)]
pub(super) enum KeyOutcome {
    /// Normal dispatch — state mutation complete, keep running.
    Continue,
    /// Ctrl+S / Super+S: persist buffer to disk, then keep running.
    Save,
    /// ExitPrompt Y: persist buffer to disk, then exit the loop.
    SaveAndExit,
    /// ExitPrompt N / Ctrl+X on a clean buffer: exit without saving.
    Exit,
    /// Ctrl+R: reload config from disk and redisplay a confirmation banner.
    ReloadConfig,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Map a screen-absolute (row, col) mouse position to a logical document
/// (row, col) position, accounting for the editor gutter, scroll offset, and
/// soft-wrapped lines. Returns `None` if the click is outside the editor area.
///
/// `decoration_map` is required to look up each line's `continuation_indent`
/// so that list items and blockquotes (whose continuation rows are wrapped at a
/// narrower width and rendered with a visual indent) count the correct number of
/// visual rows and map column positions correctly.
pub(super) fn screen_to_doc(
    screen_row: u16,
    screen_col: u16,
    editor_area: &Rect,
    scroll_top: usize,
    lines: &[String],
    decoration_map: &yame::decoration::DecorationMap,
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
        // Continuation indent for this line (0 for plain paragraphs, ≥2 for
        // list items and blockquotes).  Must match the renderer exactly so that
        // visual row counts are identical.
        let line_ci = decoration_map
            .get(&li)
            .map(|decs| decs.iter().map(|s| s.continuation_indent).max().unwrap_or(0))
            .unwrap_or(0) as usize;
        let cont_width = cw.saturating_sub(line_ci).max(1);
        let wrapped = renderer::wrap_line_indented(line, cw, cont_width);
        let seg_count = wrapped.len().max(1);
        if vis + seg_count > click_vis_row {
            let si = click_vis_row - vis;
            let char_ranges = renderer::wrap_char_ranges(line, &wrapped);
            let seg_char_start = char_ranges.get(si).map_or(0, |&(start, _)| start);
            let row_str = wrapped.get(si).copied().unwrap_or("");
            // Continuation rows (si > 0) are rendered with `line_ci` visual
            // columns of indent before the text.  Subtract that offset so the
            // click column maps to the correct position within `row_str`.
            let col_in_row = if si > 0 {
                click_col.saturating_sub(line_ci)
            } else {
                click_col
            };
            let chars_into_row = renderer::chars_for_display_cols(row_str, col_in_row);
            let doc_col = (seg_char_start + chars_into_row).min(line.chars().count());
            return Some((li as u16, doc_col as u16));
        }
        vis += seg_count;
    }
    Some((lines.len().saturating_sub(1) as u16, 0))
}

/// Returns `true` if the key is a pure cursor-movement key that cannot change
/// document content. Used to skip the decoration debounce timer on nav presses.
pub(super) fn is_navigation_key(k: &crossterm::event::KeyEvent) -> bool {
    // Ctrl+Up/Down are handled in their own explicit arm in handle_key_event
    // (they scroll the viewport rather than edit), so they never reach the `_`
    // arm where is_navigation_key is called.  Nonetheless, matching purely on
    // k.code (ignoring modifiers) means Ctrl+Up is still classified nav here,
    // which is the correct policy if the arm ordering ever changes.
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
/// is no active selection. Does not fall back to the current line.
pub(super) fn get_selection_text(app: &App) -> Option<String> {
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

/// If there is an active selection and `k` is a pair-opener, wrap the
/// selection with the corresponding pair and return `true`.
pub(super) fn handle_pair_wrap(app: &mut App, k: crossterm::event::KeyEvent) -> bool {
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
    let selected = match get_selection_text(app) {
        Some(s) => s,
        None => return false,
    };
    app.textarea.input(k);
    app.textarea.insert_str(format!("{selected}{close}"));
    true
}

// ---------------------------------------------------------------------------
// Key-event dispatcher (pure — no file I/O, no terminal I/O)
// ---------------------------------------------------------------------------

/// Dispatch a single key event, mutating `app` state.
///
/// Returns a [`KeyOutcome`] telling the caller what (if any) I/O action to
/// perform next. File writes, config reloads, and loop termination are the
/// responsibility of the caller (`event_loop`).  This separation makes the
/// entire key-dispatch path unit-testable without a real terminal or filesystem.
pub(super) fn handle_key_event(app: &mut App, k: crossterm::event::KeyEvent) -> KeyOutcome {
    // Any key press re-engages cursor-clamping scroll.
    // Ctrl+Up/Down immediately override this below by setting free_scroll = true again.
    app.free_scroll = false;

    // ── Exit-prompt mode ────────────────────────────────────────────────────
    if matches!(app.status.mode, StatusMode::ExitPrompt) {
        return match k.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => KeyOutcome::SaveAndExit,
            KeyCode::Char('n') | KeyCode::Char('N') => KeyOutcome::Exit,
            KeyCode::Esc
            | KeyCode::Char('c')
            | KeyCode::Char('C')
            | KeyCode::Char('x')
            | KeyCode::Char('X') => {
                app.status.mode = StatusMode::Normal;
                KeyOutcome::Continue
            }
            _ => KeyOutcome::Continue,
        };
    }

    // ── Normal editing mode ─────────────────────────────────────────────────
    match (k.modifiers, k.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) | (KeyModifiers::SUPER, KeyCode::Char('s')) => {
            KeyOutcome::Save
        }

        (KeyModifiers::CONTROL, KeyCode::Char('x')) | (KeyModifiers::NONE, KeyCode::Esc) => {
            if handle_exit(app) {
                KeyOutcome::Exit
            } else {
                KeyOutcome::Continue
            }
        }

        (KeyModifiers::CONTROL, KeyCode::Char('c')) | (KeyModifiers::SUPER, KeyCode::Char('c')) => {
            yame::clipboard::handle_copy(app);
            KeyOutcome::Continue
        }

        (KeyModifiers::CONTROL, KeyCode::Char('v')) | (KeyModifiers::SUPER, KeyCode::Char('v')) => {
            yame::clipboard::handle_paste(app);
            app.force_redecorate = true;
            KeyOutcome::Continue
        }

        (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
            app.status.dismiss();
            app.config_warnings.clear();
            app.textarea.undo();
            app.force_redecorate = true;
            app.last_keystroke = Some(std::time::Instant::now());
            app.recompute_dirty();
            KeyOutcome::Continue
        }

        (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
            app.status.dismiss();
            app.config_warnings.clear();
            app.textarea.redo();
            app.force_redecorate = true;
            app.last_keystroke = Some(std::time::Instant::now());
            app.recompute_dirty();
            KeyOutcome::Continue
        }

        (KeyModifiers::CONTROL, KeyCode::Char('r')) => KeyOutcome::ReloadConfig,

        // Ctrl+Up/Down: scroll viewport without moving cursor.
        (KeyModifiers::CONTROL, KeyCode::Up) => {
            app.scroll_top = app.scroll_top.saturating_sub(1);
            app.free_scroll = true;
            KeyOutcome::Continue
        }

        (KeyModifiers::CONTROL, KeyCode::Down) => {
            let max = app.textarea.lines().len().saturating_sub(1);
            app.scroll_top = (app.scroll_top + 1).min(max);
            app.free_scroll = true;
            KeyOutcome::Continue
        }

        // Visual-line Up/Down: step by displayed row, not by logical line.
        (KeyModifiers::NONE, KeyCode::Down) => handle_visual_move(app, true, false),
        (KeyModifiers::NONE, KeyCode::Up) => handle_visual_move(app, false, false),
        (KeyModifiers::SHIFT, KeyCode::Down) => handle_visual_move(app, true, true),
        (KeyModifiers::SHIFT, KeyCode::Up) => handle_visual_move(app, false, true),

        _ => {
            // Any non-vertical-nav key ends the sticky-column gesture.
            app.sticky_col = None;
            app.status.dismiss();
            app.config_warnings.clear();
            let is_nav = is_navigation_key(&k);

            if !is_nav && handle_pair_wrap(app, k) {
                app.force_redecorate = true;
                app.mark_keystroke();
            } else {
                let prev_line_count = app.textarea.lines().len();
                app.textarea.input(k);
                if app.textarea.lines().len() != prev_line_count {
                    app.force_redecorate = true;
                }
                if is_nav {
                    app.recompute_dirty();
                } else {
                    app.mark_keystroke();
                }
            }
            KeyOutcome::Continue
        }
    }
}

// ---------------------------------------------------------------------------
// Visual-line navigation
// ---------------------------------------------------------------------------

/// Move the cursor one visual row up (`go_down = false`) or down (`go_down =
/// true`), honouring soft-wrap so the cursor steps through displayed rows
/// rather than jumping over wrapped text to the next logical line.
///
/// Uses `app.content_width` (kept current by the event loop) and
/// `app.decoration_map` (for continuation-indent widths on list/blockquote
/// lines) to use exactly the same wrapping as the renderer.
///
/// `app.sticky_col` is set on the first call of a vertical gesture and
/// preserved on subsequent Up/Down presses; any other key clears it (see the
/// `_` arm of `handle_key_event`).
fn handle_visual_move(app: &mut App, go_down: bool, selecting: bool) -> KeyOutcome {
    let cw = app.content_width;
    if cw == 0 {
        // Geometry not yet known (before first render); fall back to native.
        let code = if go_down { KeyCode::Down } else { KeyCode::Up };
        let mods = if selecting {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };
        app.textarea
            .input(crossterm::event::KeyEvent::new(code, mods));
        app.recompute_dirty();
        return KeyOutcome::Continue;
    }

    let (cur_row, cur_col) = app.textarea.cursor();
    let lines = app.textarea.lines();

    // Wrap widths for the current logical line (matches renderer).
    let cur_ci = app
        .decoration_map
        .get(&cur_row)
        .map(|decs| {
            decs.iter()
                .map(|s| s.continuation_indent)
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0) as usize;
    let cur_cont = cw.saturating_sub(cur_ci).max(1);
    let cur_line = lines.get(cur_row).map_or("", |s| s.as_str());

    let (cur_subrow, _cur_char_start, cur_total) =
        renderer::cursor_subrow_info(cur_line, cur_col, cw, cur_cont);

    // Establish (or recover) the sticky column for this gesture.
    // Stored in display columns (not char count) so wide chars are handled correctly.
    let vcol = *app
        .sticky_col
        .get_or_insert_with(|| renderer::cursor_vcol(cur_line, cur_col, cw, cur_cont));

    // Determine target (logical row, subrow-within-that-row).
    let (tgt_row, tgt_subrow) = if go_down {
        if cur_subrow + 1 < cur_total {
            (cur_row, cur_subrow + 1)
        } else if cur_row + 1 < lines.len() {
            (cur_row + 1, 0)
        } else {
            return KeyOutcome::Continue; // already at last visual row
        }
    } else {
        if cur_subrow > 0 {
            (cur_row, cur_subrow - 1)
        } else if cur_row > 0 {
            let prev = cur_row - 1;
            let prev_ci = app
                .decoration_map
                .get(&prev)
                .map(|decs| {
                    decs.iter()
                        .map(|s| s.continuation_indent)
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0) as usize;
            let prev_cont = cw.saturating_sub(prev_ci).max(1);
            let prev_line = lines.get(prev).map_or("", |s| s.as_str());
            let prev_total = renderer::wrap_line_indented(prev_line, cw, prev_cont)
                .len()
                .max(1);
            (prev, prev_total - 1)
        } else {
            return KeyOutcome::Continue; // already at first visual row
        }
    };

    // Convert (tgt_subrow, vcol) → logical char column in the target line.
    let tgt_line = lines.get(tgt_row).map_or("", |s| s.as_str());
    let tgt_ci = app
        .decoration_map
        .get(&tgt_row)
        .map(|decs| {
            decs.iter()
                .map(|s| s.continuation_indent)
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0) as usize;
    let tgt_cont = cw.saturating_sub(tgt_ci).max(1);
    let tgt_col = renderer::char_col_at_visual(tgt_line, tgt_subrow, vcol, cw, tgt_cont);

    // Apply or extend selection.
    if selecting {
        if app.textarea.selection_range().is_none() {
            app.textarea.start_selection();
        }
    } else {
        app.textarea.cancel_selection();
    }

    app.textarea
        .move_cursor(CursorMove::Jump(tgt_row as u16, tgt_col as u16));
    app.recompute_dirty();
    KeyOutcome::Continue
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

#[mutants::skip] // Terminal I/O loop — requires a real terminal backend + live event stream; not unit-testable.
pub(super) fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    layout_config: &LayoutConfig,
) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
    const POLL_TIMEOUT: Duration = Duration::from_millis(16);
    const DEBOUNCE: Duration = Duration::from_millis(50);
    const BOTTOM_PADDING: usize = 3;
    const SCROLL_LINES: usize = 3;

    let min_cols = layout_config.min_cols.unwrap_or(DEFAULT_MIN_COLS);

    // Initial decoration pass.
    {
        let text = app.textarea.lines().join("\n");
        let (map, wc) =
            build_decoration_map(&text, &app.theme, app.italic_support, app.highlight_cache.as_ref());
        app.decoration_map = map;
        app.word_count = wc;
    }

    let mut last_editor_area = Rect::default();
    let mut drag_selecting = false;

    loop {
        if app.force_redecorate || app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            let (map, wc) = build_decoration_map(
                &text,
                &app.theme,
                app.italic_support,
                app.highlight_cache.as_ref(),
            );
            app.decoration_map = map;
            app.word_count = wc;
            app.last_keystroke = None;
            app.force_redecorate = false;
        }
        app.status.tick();

        // Pre-draw scroll clamp
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
            // Keep content_width current so handle_visual_move wraps identically
            // to the renderer.  Computed here (pre-draw) so it is valid before
            // the first key event arrives.
            app.content_width = (pre_editor_area.width as usize)
                .saturating_sub(renderer::GUTTER as usize + renderer::GUTTER as usize)
                .max(1);

            // Clamp is skipped while the user is free-scrolling (mouse wheel or
            // Ctrl+Up/Down).  free_scroll persists until a key press, mouse click,
            // drag, or terminal resize clears it (scroll and hover events do not).
            if !app.free_scroll {
                clamp_scroll(
                    app,
                    pre_editor_area,
                    pre_layout.column.width,
                    BOTTOM_PADDING,
                );
            }
        }

        execute!(io::stdout(), BeginSynchronizedUpdate)?;
        terminal.draw(|f| {
            let layout = compute_layout(f.area(), min_cols);

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

            last_editor_area = editor_area;
        })?;
        execute!(io::stdout(), EndSynchronizedUpdate)?;

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(k) => match handle_key_event(app, k) {
                    KeyOutcome::Continue => {}
                    KeyOutcome::Save => {
                        handle_save(app)?;
                    }
                    KeyOutcome::SaveAndExit => {
                        handle_save(app)?;
                        break;
                    }
                    KeyOutcome::Exit => break,
                    KeyOutcome::ReloadConfig => {
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
                        app.last_keystroke = Some(std::time::Instant::now());
                    }
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        let max = app.textarea.lines().len().saturating_sub(1);
                        app.scroll_top = (app.scroll_top + SCROLL_LINES).min(max);
                        app.free_scroll = true;
                    }
                    MouseEventKind::ScrollUp => {
                        app.scroll_top = app.scroll_top.saturating_sub(SCROLL_LINES);
                        app.free_scroll = true;
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Click re-engages cursor-clamping scroll.
                        app.free_scroll = false;
                        drag_selecting = false;
                        if let Some((doc_row, doc_col)) = screen_to_doc(
                            mouse.row,
                            mouse.column,
                            &last_editor_area,
                            app.scroll_top,
                            app.textarea.lines(),
                            &app.decoration_map,
                        ) {
                            app.textarea.cancel_selection();
                            app.textarea.move_cursor(CursorMove::Jump(doc_row, doc_col));
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        // Drag moves the cursor, so re-engage cursor-clamping scroll.
                        app.free_scroll = false;
                        if let Some((doc_row, doc_col)) = screen_to_doc(
                            mouse.row,
                            mouse.column,
                            &last_editor_area,
                            app.scroll_top,
                            app.textarea.lines(),
                            &app.decoration_map,
                        ) {
                            if !drag_selecting {
                                app.textarea.start_selection();
                                drag_selecting = true;
                            }
                            app.textarea.move_cursor(CursorMove::Jump(doc_row, doc_col));
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    // Viewport geometry changed — re-engage cursor-clamping scroll
                    // so the cursor is guaranteed visible after the resize.
                    app.free_scroll = false;
                }
                _ => {
                    // Unknown events (FocusGained, FocusLost, mouse hover, …) do
                    // NOT clear free_scroll — they are background events that should
                    // not interrupt an explicit scroll the user initiated.
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;
    use tui_textarea::TextArea;
    use yame::app::App;
    use yame::config::Theme;
    use yame::decoration::DecorationMap;
    use yame::status::StatusLine;

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
            sticky_col: None,
            content_width: 0,
            clipboard: None,
            initial_file_empty: false,
            highlight_cache: None,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    // Kills: input.rs:269:24 replace && with || in handle_key_event.
    // With ||: !is_nav || handle_pair_wrap short-circuits to true for any non-nav key
    // (handle_pair_wrap is never evaluated), so the if-branch is entered without
    // calling textarea.input → the textarea stays empty instead of receiving 'a'.
    #[test]
    fn typing_char_reaches_textarea() {
        let mut app = make_app();
        handle_key_event(&mut app, key(KeyCode::Char('a')));
        assert_eq!(
            app.textarea.lines()[0],
            "a",
            "typed char must reach textarea"
        );
    }

    // Kills: input.rs:269:16 delete ! in handle_key_event.
    // Without !: is_nav && handle_pair_wrap short-circuits to false for all non-nav
    // keys (is_nav=false), so pair-wrap is never called; instead textarea.input('(')
    // is called → just "(" instead of the wrapped "(hello)".
    #[test]
    fn pair_wrap_with_selection_wraps() {
        let mut app = make_app();
        app.textarea.insert_str("hello");
        app.textarea.move_cursor(CursorMove::Head);
        app.textarea.start_selection();
        app.textarea.move_cursor(CursorMove::End);
        handle_key_event(&mut app, key(KeyCode::Char('(')));
        assert_eq!(
            app.textarea.lines()[0],
            "(hello)",
            "pair-wrap must wrap the selection"
        );
    }

    // Kills: input.rs:275:47 replace != with == in handle_key_event.
    // With ==: force_redecorate is set only when line count has NOT changed;
    // pressing Enter adds a new line, so with the mutation force_redecorate is NOT set.
    #[test]
    fn enter_sets_force_redecorate() {
        let mut app = make_app();
        app.force_redecorate = false;
        handle_key_event(&mut app, key(KeyCode::Enter));
        assert!(
            app.force_redecorate,
            "Enter adds a line — force_redecorate must be true"
        );
    }

    // ── Visual-line navigation ───────────────────────────────────────────────

    fn nav_app(lines: Vec<&str>, content_width: usize) -> App {
        let mut app = make_app();
        app.content_width = content_width;
        app.textarea = TextArea::new(lines.into_iter().map(String::from).collect());
        app
    }

    // Down stays within the same logical line when it wraps.
    // "abcde fghij" at width 8 wraps: first row "abcde" (cols 0-4),
    // second row "fghij" (cols 6-10). Cursor at col 0 → Down → col 6.
    #[test]
    fn visual_down_within_wrapped_line() {
        let mut app = nav_app(vec!["abcde fghij"], 8);
        app.textarea.move_cursor(CursorMove::Jump(0, 0));
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(
            app.textarea.cursor(),
            (0, 6),
            "Down must land on second visual row of same logical line"
        );
    }

    // Up reverses the within-line move.
    #[test]
    fn visual_up_within_wrapped_line() {
        let mut app = nav_app(vec!["abcde fghij"], 8);
        app.textarea.move_cursor(CursorMove::Jump(0, 6));
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(
            app.textarea.cursor(),
            (0, 0),
            "Up must return to first visual row of same logical line"
        );
    }

    // Down on the last visual row of line 0 crosses to line 1.
    #[test]
    fn visual_down_crosses_logical_line() {
        let mut app = nav_app(vec!["abc", "def"], 20);
        app.textarea.move_cursor(CursorMove::Jump(0, 2));
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(
            app.textarea.cursor(),
            (1, 2),
            "Down from last visual row must cross to next logical line"
        );
    }

    // Up on the first visual row of line 1 crosses back to line 0.
    #[test]
    fn visual_up_crosses_logical_line() {
        let mut app = nav_app(vec!["abc", "def"], 20);
        app.textarea.move_cursor(CursorMove::Jump(1, 2));
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(
            app.textarea.cursor(),
            (0, 2),
            "Up from first visual row must cross back to previous logical line"
        );
    }

    // Sticky col is set on the first Down and preserved on the second, so
    // moving through a short middle line restores the column on a longer line.
    // Lines: ["abcde", "ab", "abcde"], width 20 (no wrapping).
    // Cursor at (0, 4): Down → (1, 2) [clamped]; Down → (2, 4) [restored].
    #[test]
    fn sticky_col_preserved_through_short_line() {
        let mut app = nav_app(vec!["abcde", "ab", "abcde"], 20);
        app.textarea.move_cursor(CursorMove::Jump(0, 4));
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.textarea.cursor(), (1, 2), "clamped to short line");
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(
            app.textarea.cursor(),
            (2, 4),
            "sticky col must restore original column on longer line"
        );
    }

    // Any non-vertical-nav key must clear sticky_col.
    #[test]
    fn sticky_col_cleared_by_non_vertical_key() {
        let mut app = nav_app(vec!["abcde", "ab", "abcde"], 20);
        app.textarea.move_cursor(CursorMove::Jump(0, 4));
        handle_key_event(&mut app, key(KeyCode::Down)); // sets sticky_col = 4
        assert!(app.sticky_col.is_some(), "sticky_col set after Down");
        handle_key_event(&mut app, key(KeyCode::Right)); // non-vertical → clears
        assert!(
            app.sticky_col.is_none(),
            "sticky_col must be cleared by Right"
        );
    }

    // Down at the last line/row is a no-op (cursor stays put).
    #[test]
    fn visual_down_at_last_row_is_noop() {
        let mut app = nav_app(vec!["abc"], 20);
        app.textarea.move_cursor(CursorMove::Jump(0, 1));
        handle_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(
            app.textarea.cursor(),
            (0, 1),
            "Down at last row must not move cursor"
        );
    }

    // Up at the first row is a no-op.
    #[test]
    fn visual_up_at_first_row_is_noop() {
        let mut app = nav_app(vec!["abc"], 20);
        app.textarea.move_cursor(CursorMove::Jump(0, 1));
        handle_key_event(&mut app, key(KeyCode::Up));
        assert_eq!(
            app.textarea.cursor(),
            (0, 1),
            "Up at first row must not move cursor"
        );
    }

    // ── screen_to_doc ────────────────────────────────────────────────────────

    // Helper: editor area spanning the full terminal at (0,0), width includes
    // two GUTTER columns (1 each side), so content_width = area.width - 2.
    fn editor_rect(width: u16, height: u16) -> Rect {
        Rect { x: 0, y: 0, width, height }
    }

    // Helper: build a DecorationMap with a single span on logical line `li`
    // whose only non-default field is `continuation_indent`.
    fn dec_map_with_ci(li: usize, ci: u8) -> DecorationMap {
        use yame::decoration::StyledSpan;
        let mut map = DecorationMap::default();
        map.insert(li, vec![StyledSpan { continuation_indent: ci, ..StyledSpan::default() }]);
        map
    }

    // Click outside the editor area returns None.
    #[test]
    fn screen_to_doc_outside_area_returns_none() {
        let area = editor_rect(12, 5);
        let lines: Vec<String> = vec!["hello".into()];
        let map = DecorationMap::default();
        // Row above area
        assert!(screen_to_doc(0, 0, &Rect { x: 0, y: 2, width: 12, height: 5 }, 0, &lines, &map).is_none());
        // Col outside area
        assert!(screen_to_doc(0, 20, &area, 0, &lines, &map).is_none());
    }

    // Plain (no continuation indent) line: click at gutter+2 → doc col 2.
    #[test]
    fn screen_to_doc_plain_line_click() {
        // area width=12 → GUTTER=1 each side → cw=10
        let area = editor_rect(12, 5);
        let lines: Vec<String> = vec!["hello world".into()];
        let map = DecorationMap::default();
        // screen_col = GUTTER + 2 = 3 → click_col = 2 → char 2 = 'l'
        let result = screen_to_doc(0, 3, &area, 0, &lines, &map);
        assert_eq!(result, Some((0, 2)), "plain click must map col correctly");
    }

    // Regression: click on the *third* visual row of a wrapped list item must
    // map to the same logical line (0), not the next logical line (1).
    //
    // Setup: cw=10, ci=2 → cont_width=8
    //   line 0: "- abc defgh ijk"
    //     wrap_line_indented → ["- abc", "defgh", "ijk"]   (3 visual rows)
    //   line 1: "next line"
    //
    // Old bug: wrap_line (ignoring ci) gave ["- abc", "defgh ijk"] (2 rows),
    // so vis row 2 was counted as the start of line 1.
    #[test]
    fn screen_to_doc_list_item_third_wrap_row_lands_on_correct_logical_line() {
        let area = editor_rect(12, 10); // cw = 10
        let lines: Vec<String> = vec!["- abc defgh ijk".into(), "next line".into()];
        let map = dec_map_with_ci(0, 2);
        // Visual row 2 is the "ijk" continuation row of line 0.
        // screen_col = GUTTER(1) + ci(2) = 3 → clicking at the first char of "ijk".
        let result = screen_to_doc(2, 3, &area, 0, &lines, &map);
        assert_eq!(
            result.map(|(r, _)| r),
            Some(0),
            "third visual row of wrapped list item must map to logical line 0"
        );
    }

    // Column mapping on a continuation row must subtract the continuation
    // indent before computing the char position.
    //
    // Continuation row "defgh" of "- abc defgh ijk" at ci=2:
    //   screen_col = GUTTER(1) + ci(2) + 3 = 6 → click_col=5, col_in_row=5-2=3
    //   "defgh"[0..3] = "def" → char index 3 within "defgh" → char 8 in original
    //   ("- abc " = 6 chars, "defgh" starts at 6, char 3 within it → global 9)
    #[test]
    fn screen_to_doc_list_item_continuation_column_adjusted() {
        let area = editor_rect(12, 10); // cw = 10
        // "- abc defgh ijk": '- abc ' = 6 chars, 'defgh' starts at char 6
        let lines: Vec<String> = vec!["- abc defgh ijk".into(), "next line".into()];
        let map = dec_map_with_ci(0, 2);
        // Visual row 1 = "defgh" continuation row.
        // screen_col = 1 (GUTTER) + 2 (ci) + 3 = 6 → click_col=5, col_in_row=3
        // → chars_for_display_cols("defgh", 3) = 3 → doc_col = 6 + 3 = 9
        let result = screen_to_doc(1, 6, &area, 0, &lines, &map);
        assert_eq!(
            result,
            Some((0, 9)),
            "continuation row column must be adjusted by continuation_indent"
        );
    }

    // ── Navigation inertness ─────────────────────────────────────────────────

    // Navigation keys (Up/Down/Left/Right) must NOT set last_keystroke.
    //
    // Setting last_keystroke arms the 50ms debounce timer that triggers a full
    // decoration pass.  Pure cursor movement cannot change content, so the
    // decoration map is already valid — re-running it wastes CPU and causes
    // perceptible lag on large files.
    //
    // Kills: any mutation that routes nav keys through mark_keystroke() instead
    // of recompute_dirty().
    #[test]
    fn nav_keys_do_not_set_last_keystroke() {
        for code in [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
        ] {
            let mut app = make_app();
            app.textarea.insert_str("line one\nline two");
            app.last_keystroke = None;
            handle_key_event(&mut app, KeyEvent::new(code, KeyModifiers::NONE));
            assert!(
                app.last_keystroke.is_none(),
                "{code:?} must not set last_keystroke (would trigger redundant decoration pass)"
            );
        }
    }

    // ── Exit-prompt cancellation ─────────────────────────────────────────────

    // Pressing Esc while in ExitPrompt must return to Normal mode without
    // exiting (KeyOutcome::Continue).
    //
    // Regression guard for FEEDBACK-1 §1.1: the original code matched
    // (NONE, Esc) at the outer level, shadowing the ExitPrompt handler.
    #[test]
    fn exit_prompt_esc_cancels_and_returns_to_normal() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        let outcome = handle_key_event(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(outcome, KeyOutcome::Continue, "Esc in ExitPrompt must return Continue");
        assert!(
            matches!(app.status.mode, yame::status::StatusMode::Normal),
            "Esc in ExitPrompt must restore Normal mode"
        );
    }

    // Pressing 'c' (bare, or Ctrl+C) while in ExitPrompt must also cancel,
    // not copy to clipboard.  The ExitPrompt handler matches on k.code only,
    // so modifiers do not affect it.
    #[test]
    fn exit_prompt_c_cancels_regardless_of_modifier() {
        for modifiers in [KeyModifiers::NONE, KeyModifiers::CONTROL] {
            let mut app = make_app();
            app.status.mode = yame::status::StatusMode::ExitPrompt;
            let outcome =
                handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('c'), modifiers));
            assert_eq!(
                outcome,
                KeyOutcome::Continue,
                "'c' (mods={modifiers:?}) in ExitPrompt must return Continue"
            );
            assert!(
                matches!(app.status.mode, yame::status::StatusMode::Normal),
                "'c' (mods={modifiers:?}) in ExitPrompt must restore Normal mode"
            );
        }
    }
}
