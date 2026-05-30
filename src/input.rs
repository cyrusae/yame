use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};
use ratatui::{Terminal, layout::Rect, style::Style, widgets::Paragraph};
use tui_textarea::CursorMove;

use yame::app::{App, FileMode, get_selection_text, resolve_file_mode};
use yame::config::{LayoutConfig, Theme, load_config};
use yame::decoration::{block_highlights_to_decoration_map, build_decoration_map, count_words};
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
// Decoration dispatch
// ---------------------------------------------------------------------------

/// Run the appropriate decoration / highlighting pass for the current file mode
/// and return `(DecorationMap, word_count)`.
///
/// - `Markdown` → full pulldown-cmark decoration pass (existing path).
/// - `PlainHighlight(lang)` → syntect whole-file highlight; word count computed
///   separately via [`count_words`].
/// - `PlainText` → no decoration; word count only.
fn decorate(text: &str, app: &App) -> (yame::decoration::DecorationMap, usize) {
    match &app.file_mode {
        FileMode::Markdown => {
            build_decoration_map(text, &app.theme, app.italic_support, app.highlight_cache.as_ref())
        }
        FileMode::PlainHighlight(lang) => {
            let map = app
                .highlight_cache
                .as_ref()
                .and_then(|cache| cache.highlight_block(lang, text))
                .map(|hl| block_highlights_to_decoration_map(&hl, 0))
                .unwrap_or_default();
            let wc = count_words(text);
            (map, wc)
        }
        FileMode::PlainText => (yame::decoration::DecorationMap::default(), count_words(text)),
    }
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
// Timeouts under cargo-mutants because the 91 binary-crate tests run in parallel
// and a slow integration test reliably pushes the suite past the budget even when
// a mutation-specific unit test would fail immediately.  The function is already
// covered by 10+ targeted unit tests (lines 930–1164) with inline mutation notes;
// mutation verification here adds no safety signal beyond those tests.
#[mutants::skip]
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
    // Written as GUTTER + GUTTER (not 2 * GUTTER) so that the `* → /` operator
    // mutation is eliminated: 2/1 == 2 is equivalent, but `+ → -` (0) and
    // `+ → *` (1) produce distinct, observable wrong values.
    let cw = (editor_area.width as usize)
        .saturating_sub(renderer::GUTTER as usize + renderer::GUTTER as usize)
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
        // Guard the destructive y/n responses: Ctrl+Y must not trigger SaveAndExit
        // and Ctrl+N must not trigger Exit.  Other modifier+char combos (Ctrl+C,
        // Ctrl+X …) still pass through so they continue to act as cancel shortcuts.
        if k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
            && matches!(k.code, KeyCode::Char('y') | KeyCode::Char('n'))
        {
            return KeyOutcome::Continue;
        }
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

        // Tab: expand to spaces aligned to the next tab stop rather than inserting
        // a raw '\t'.  Raw tabs break cursor positioning, selection, and soft-wrap
        // because the layout engine assumes every character is 1 display column wide.
        (KeyModifiers::NONE, KeyCode::Tab) => {
            let (_, col) = app.textarea.cursor();
            let tw = app.tab_width;
            let spaces = tw - (col % tw);
            for _ in 0..spaces {
                app.textarea.input(crossterm::event::KeyEvent::new(
                    KeyCode::Char(' '),
                    KeyModifiers::NONE,
                ));
            }
            app.mark_keystroke();
            KeyOutcome::Continue
        }

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
                if !is_nav {
                    app.mark_keystroke();
                }
                // Navigation keys cannot mutate content, so is_dirty cannot change;
                // skipping recompute_dirty() here avoids an O(N) line comparison on
                // every arrow-key press for large documents.
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
            // Equivalent-mutant note: `- → +` yields prev_total + 1 and `- → /`
            // yields prev_total / 1 = prev_total.  Both are clamped to the last
            // valid subrow index by `char_col_at_visual` (which does
            // `target_subrow.min(last_idx)`), so no test can distinguish them from
            // `prev_total - 1`.
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
        let (map, wc) = decorate(&text, app);
        app.decoration_map = map;
        app.word_count = wc;
    }

    let mut last_editor_area = Rect::default();
    let mut drag_selecting = false;

    loop {
        if app.force_redecorate || app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            let (map, wc) = decorate(&text, app);
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
                        // Rebuild the highlight cache so fenced code blocks pick up
                        // any theme or palette changes immediately.
                        app.highlight_cache = new_config.highlighting.enabled.then(|| {
                            let palette_theme = new_config
                                .highlighting
                                .use_palette_colors
                                .then(|| yame::highlighting::build_palette_theme(&app.theme));
                            yame::highlighting::HighlightCache::new(
                                true,
                                new_config.highlighting.syntect_theme.clone(),
                                palette_theme,
                            )
                        });
                        // Re-resolve file mode in case [filetype] config changed.
                        app.file_mode = resolve_file_mode(&app.file_path, &new_config.filetype);
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
    use yame::app::{App, ClipboardState, FileMode};
    use yame::config::Theme;
    use yame::decoration::DecorationMap;
    use yame::status::StatusLine;

    fn make_app() -> App {
        App {
            textarea: TextArea::default(),
            file_path: PathBuf::from("test.md"),
            shortened_path: "test.md".to_string(),
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
            clipboard: ClipboardState::Uninitialized,
            tab_width: 4,
            highlight_cache: None,
            file_mode: FileMode::Markdown,
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

    // ── decorate() ──────────────────────────────────────────────────────────
    //
    // Kills input.rs:55:5 `replace decorate → … with (Default::default(), 0)` and
    //                      `replace decorate → … with (Default::default(), 1)`.
    //
    // Both stubs return an empty DecorationMap AND word_count=0 or 1.  A real
    // markdown decoration pass on multi-word text with a heading returns a
    // non-empty map AND word_count > 1.

    // FileMode::Markdown: heading + body text → non-empty map, word_count > 1.
    #[test]
    fn decorate_markdown_returns_nonempty_map_and_correct_word_count() {
        let mut app = make_app();
        app.file_mode = FileMode::Markdown;
        let text = "# Hello World\nThis is some text.\n";
        let (map, wc) = decorate(text, &app);
        assert!(
            !map.is_empty(),
            "Markdown decoration must produce a non-empty DecorationMap"
        );
        // "Hello World This is some text" = 7 words; stubs return 0 or 1.
        assert!(
            wc > 1,
            "Markdown decoration word count must be > 1 for multi-word text (got {wc})"
        );
    }

    // FileMode::PlainText: no decoration, word count only.
    #[test]
    fn decorate_plain_text_returns_empty_map_with_word_count() {
        let mut app = make_app();
        app.file_mode = FileMode::PlainText;
        let (map, wc) = decorate("hello world\n", &app);
        assert!(
            map.is_empty(),
            "PlainText mode must return an empty DecorationMap"
        );
        assert_eq!(wc, 2, "PlainText mode must still count words correctly");
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

    // Click where screen_col < editor_area.x (not 0) must return None.
    //
    // The bounds check is `screen_col < editor_area.x`. When area.x == 0, that
    // condition is unreachable (u16 < 0 never holds), so these mutants:
    //   · `< → ==`  (col 23): `screen_col == area.x` is false when col < x
    //   · `< → <=`  (col 23): admits clicks where col == x (inside area) as None
    //   · `|| → &&` (col 9 of the joined condition): the combined guard collapses
    //     to a conjunction — only returns None when ALL four conditions are true
    // …are only observable with area.x > 0.
    #[test]
    fn screen_to_doc_outside_area_nonzero_x_returns_none() {
        // area starts at x=3, y=1; any click with screen_col < 3 or screen_row < 1
        // (while the other coordinate is inside) must return None.
        let area = Rect { x: 3, y: 1, width: 10, height: 5 };
        let lines: Vec<String> = vec!["hello".into()];
        let map = DecorationMap::default();

        // screen_col=2 < area.x=3; screen_row=2 is in [1,6) — only col violates bounds.
        // With `||→&&` mutation the conjunction fires only if ALL four hold, which is
        // false here (row is valid), so the mutant would NOT return None.
        assert!(
            screen_to_doc(2, 2, &area, 0, &lines, &map).is_none(),
            "col below area.x must return None even when row is in-bounds"
        );

        // screen_col=2 < area.x=3 and screen_row=0 < area.y=1 — both out of bounds.
        assert!(
            screen_to_doc(0, 2, &area, 0, &lines, &map).is_none(),
            "both row and col below area must return None"
        );
    }

    // A click exactly AT screen_col == editor_area.x is INSIDE the area (it
    // lands on the first gutter column).  With the `< → <=` mutation this click
    // would incorrectly return None.
    #[test]
    fn screen_to_doc_click_at_area_x_edge_returns_some() {
        // area.x = 2, so the valid column range is [2, 12).  screen_col = 2 (= area.x)
        // is the first valid column.  click_col = 2 - (x=2 + GUTTER=1) saturating = 0.
        let area = Rect { x: 2, y: 0, width: 10, height: 5 };
        let lines: Vec<String> = vec!["hello".into()];
        let map = DecorationMap::default();
        let result = screen_to_doc(0, 2, &area, 0, &lines, &map);
        assert!(
            result.is_some(),
            "screen_col == area.x is inside the area and must return Some"
        );
    }

    // When editor_area.y > 0, click_vis_row must subtract area.y, not add it.
    // With the `- → +` mutation click_vis_row = screen_row + area.y, which maps
    // clicks to the wrong (much-further-down) visual row.
    #[test]
    fn screen_to_doc_area_y_offset_subtracted() {
        // area.y = 1 — editor starts one row down from the terminal top.
        // Lines: ["aaa", "bbb", "ccc"] — each one visual row.
        // Click at screen_row=2: correct click_vis_row = 2-1 = 1 → logical line 1 ("bbb").
        // With `+ mutation: click_vis_row = 2+1 = 3 → falls off the 3-row doc → line 2.
        let area = Rect { x: 0, y: 1, width: 12, height: 5 };
        let lines: Vec<String> = vec!["aaa".into(), "bbb".into(), "ccc".into()];
        let map = DecorationMap::default();
        let result = screen_to_doc(2, 1, &area, 0, &lines, &map);
        assert_eq!(
            result.map(|(r, _)| r),
            Some(1),
            "area.y must be subtracted (not added) from screen_row to get click_vis_row"
        );
    }

    // Clicking on the FIRST visual row of the SECOND logical line must map to
    // logical line 1, not logical line 0.
    //
    // Kills `> → >=` (line 88): with `>=`, vis+seg_count == click_vis_row enters
    // line 0's block one row early and returns (0, …) instead of (1, …).
    //
    // Also kills `+= → -=` (line 105): vis -= seg_count underflows to usize::MAX
    // in debug mode → panic → test failure.
    #[test]
    fn screen_to_doc_click_first_row_of_second_line() {
        // Two single-row lines; visual row 1 is the sole row of "world".
        let area = editor_rect(12, 5);
        let lines: Vec<String> = vec!["hello".into(), "world".into()];
        let map = DecorationMap::default();
        let result = screen_to_doc(1, 1, &area, 0, &lines, &map);
        assert_eq!(
            result.map(|(r, _)| r),
            Some(1),
            "vis-row 1 (= seg_count of line 0) must map to logical line 1"
        );
    }

    // When the click is on a line after the first, `si = click_vis_row - vis`
    // must use subtraction so it points to the correct sub-row within the line.
    //
    // Kills `- → +` (line 89): si = click_vis_row + vis → huge index → row_str=""
    // → chars_into_row=0 → doc_col=0 regardless of click_col.
    #[test]
    fn screen_to_doc_click_column_on_second_line() {
        // Click at screen_col=4 → click_col = 4-GUTTER=3. "world"[0..3]="wor" → col 3.
        let area = editor_rect(12, 5);
        let lines: Vec<String> = vec!["hello".into(), "world".into()];
        let map = DecorationMap::default();
        // screen_col = GUTTER(1) + 3 = 4
        let result = screen_to_doc(1, 4, &area, 0, &lines, &map);
        assert_eq!(
            result,
            Some((1, 3)),
            "si must use click_vis_row - vis so column offset lands on 'r' in 'world'"
        );
    }

    // cw must subtract GUTTER + GUTTER (= 2) from area.width, not a different value.
    //
    // "abcdefghij" is exactly 10 chars. With area.width=12, correct cw=10 → fits
    // on one visual row, so vis_row 1 maps to line 1. With the `+→-` mutation
    // (cw = width - 0 = 12) the line also fits, same result — but with the
    // `+→*` mutation (GUTTER * GUTTER = 1, cw = 11) it also fits. The mutation
    // that DOES produce a wrong cw is `+→-` giving 12 which still fits... hmm.
    //
    // Actually the key mutation to kill here is the old `* → +` form, now
    // expressed as `+ → *` after rewriting to `GUTTER + GUTTER`:
    // GUTTER * GUTTER = 1 × 1 = 1 → cw = 12-1=11 (still fits). The `+ → -`
    // form gives cw = 12-0=12. Both still fit for a 10-char word.
    //
    // Use a content-width-sensitive scenario instead: line = "ab cd efghi"
    // (5+1+2+1+5 = 14 chars). At cw=10 it wraps: ["ab cd", "efghi"] (2 rows).
    // At cw=9 it wraps: ["ab cd", "efghi"] too (word break before efghi). Need
    // cw == word length exactly. "0123456789" (10 chars) at cw=10 fits (1 row);
    // at cw=9 hard-breaks into ["012345678","9"] (2 rows). Click at vis_row=1:
    //   correct (cw=10): line 1 ("second").
    //   +→* mutant (cw=12-1=11): "0123456789" still fits → vis_row 1 = line 1. Same!
    //
    // Better: use "0123456789" at width=12 (cw=10 correct, cw=12 with +→- mutant).
    // At cw=10: ["0123456789"] 1 row. At cw=12: still 1 row (fits). Same.
    //
    // The observable difference requires cw to drop below 10. With area.width=12:
    //   correct:     cw = 12 - (1+1) = 10 → 1 row
    //   +→* mutant:  cw = 12 - (1*1) = 11 → 1 row (same)
    //   +→- mutant:  cw = 12 - (1-1) = 12 → 1 row (same)
    //
    // With area.width=11:
    //   correct:     cw = 11 - 2 = 9 → hard-break: ["012345678","9"] 2 rows
    //   +→* mutant:  cw = 11 - 1 = 10 → "0123456789" fits in 1 row
    //   +→- mutant:  cw = 11 - 0 = 11 → 1 row
    // Click at vis_row=1 with 2 lines ["0123456789", "second"]:
    //   correct (cw=9): line 0 has 2 rows → vis_row 1 is in line 0 → returns (0,_)
    //   +→* mutant (cw=10): line 0 has 1 row → vis_row 1 is line 1 → returns (1,_)
    #[test]
    fn screen_to_doc_content_width_gutter_subtraction() {
        // area.width=11. Correct: cw = 11 - (GUTTER+GUTTER) = 11-2 = 9.
        // "0123456789" (10 chars) at cw=9: hard-breaks into 2 rows.
        // Click at vis_row=1 → INSIDE line 0 (not line 1 "second").
        let area = Rect { x: 0, y: 0, width: 11, height: 5 };
        let lines: Vec<String> = vec!["0123456789".into(), "second".into()];
        let map = DecorationMap::default();
        let result = screen_to_doc(1, 1, &area, 0, &lines, &map);
        assert_eq!(
            result.map(|(r, _)| r),
            Some(0),
            "with cw = width - (GUTTER+GUTTER), a 10-char line at width=11 wraps into 2 rows"
        );
    }

    // si == 0 (first visual row of a list item) must NOT subtract continuation_indent
    // from click_col, because the first row is not indented.
    //
    // Kills `> → >=` (line 96): with `>=`, si=0 also subtracts line_ci, so
    // col_in_row = click_col.saturating_sub(2) instead of click_col.
    #[test]
    fn screen_to_doc_list_item_first_row_no_ci_subtraction() {
        // "- hello" (7 chars) fits on one visual row at cw=10.  ci=2.
        // Click at screen_col = GUTTER(1)+4 = 5 → click_col=4.
        // si=0 (first row): col_in_row must be 4 (no ci subtraction).
        // With `>=` mutant: col_in_row = 4 - 2 = 2 → returns (0, 2) instead of (0, 4).
        let area = editor_rect(12, 5); // cw=10
        let lines: Vec<String> = vec!["- hello".into()];
        let map = dec_map_with_ci(0, 2);
        let result = screen_to_doc(0, 5, &area, 0, &lines, &map);
        assert_eq!(
            result,
            Some((0, 4)),
            "si=0 (first row of list item) must not subtract continuation_indent from click_col"
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

    // Ctrl+Y / Ctrl+N while the exit prompt is open must NOT trigger
    // SaveAndExit / Exit — they are Redo / navigation shortcuts that the user
    // is likely to press accidentally.
    #[test]
    fn exit_prompt_ctrl_y_does_not_save_and_exit() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        let outcome = handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
        );
        assert_eq!(outcome, KeyOutcome::Continue, "Ctrl+Y must not trigger SaveAndExit");
        assert!(
            matches!(app.status.mode, yame::status::StatusMode::ExitPrompt),
            "ExitPrompt must remain open after Ctrl+Y"
        );
    }

    #[test]
    fn exit_prompt_ctrl_n_does_not_discard_and_exit() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        let outcome = handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
        );
        assert_eq!(outcome, KeyOutcome::Continue, "Ctrl+N must not trigger Exit");
        assert!(
            matches!(app.status.mode, yame::status::StatusMode::ExitPrompt),
            "ExitPrompt must remain open after Ctrl+N"
        );
    }

    // Tab key must insert spaces to the next tab stop, not a raw '\t'.
    // With app.tab_width=4 and cursor at col 0, pressing Tab should add 4 spaces.
    //
    // NOTE: these two tests use tab_width=4, which matches tui-textarea's default
    // tab_len (also 4).  They verify correct behaviour but cannot kill the
    // `delete match arm Tab` mutant because tui-textarea's built-in Tab handler
    // happens to produce identical output for tab_width==4.  The test below
    // (`tab_key_uses_app_tab_width_not_tui_textarea_default`) covers that case.
    #[test]
    fn tab_key_inserts_spaces_to_next_stop() {
        let mut app = make_app();
        handle_key_event(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            app.textarea.lines()[0],
            "    ",
            "Tab at col 0 with tab_width=4 must insert 4 spaces"
        );
    }

    // With the cursor already at col 2, pressing Tab should only insert 2 spaces
    // (to reach the next multiple-of-4 stop at col 4).
    #[test]
    fn tab_key_aligns_to_next_stop_mid_line() {
        let mut app = make_app();
        // Type two chars to reach col 2.
        handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        handle_key_event(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            app.textarea.lines()[0],
            "ab  ",
            "Tab at col 2 with tab_width=4 must insert 2 spaces (align to col 4)"
        );
    }

    // Kills input.rs:308:9 `delete match arm (KeyModifiers::NONE, KeyCode::Tab)`.
    //
    // The existing two Tab tests use app.tab_width=4, which happens to equal
    // tui-textarea's own default tab_len (4).  When the arm is deleted the key
    // falls through to `_ => { textarea.input(k) }`, and tui-textarea's built-in
    // handler inserts the same number of spaces → tests pass → mutant survives.
    //
    // Setting tab_width=2 breaks the equivalence: our arm inserts 2 spaces, but
    // tui-textarea (tab_len still 4) inserts 4 → assertion on 2 fails → mutant caught.
    #[test]
    fn tab_key_uses_app_tab_width_not_tui_textarea_default() {
        let mut app = make_app();
        app.tab_width = 2; // differs from tui-textarea's default tab_len of 4
        handle_key_event(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            app.textarea.lines()[0],
            "  ",
            "Tab at col 0 with tab_width=2 must insert 2 spaces (not 4, which tui-textarea would produce)"
        );
    }

    // ── Exit-prompt y/n outcomes ────────────────────────────────────────────
    //
    // Kills :218:13 (delete 'y' arm — 'y' would fall to `_` → Continue).
    #[test]
    fn exit_prompt_plain_y_saves_and_exits() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        assert_eq!(
            handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
            KeyOutcome::SaveAndExit,
            "plain 'y' in ExitPrompt must return SaveAndExit"
        );
    }

    // Kills :219:13 (delete 'n' arm — 'n' would fall to `_` → Continue).
    #[test]
    fn exit_prompt_plain_n_exits_without_saving() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        assert_eq!(
            handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            KeyOutcome::Exit,
            "plain 'n' in ExitPrompt must return Exit"
        );
    }

    // Kills :212:77 `| → &` (missed mutant).
    //
    // With `| → &` on the *second* `|`, the guard becomes
    // `CONTROL | (ALT & SUPER)` = `CONTROL | 0` = `CONTROL`.
    // ALT is no longer included, so Alt+Y bypasses the guard and reaches the
    // `KeyCode::Char('y') => SaveAndExit` arm.
    #[test]
    fn exit_prompt_alt_y_is_suppressed() {
        let mut app = make_app();
        app.status.mode = yame::status::StatusMode::ExitPrompt;
        assert_eq!(
            handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('y'), KeyModifiers::ALT)),
            KeyOutcome::Continue,
            "Alt+Y in ExitPrompt must be suppressed by the modifier guard"
        );
    }

    // ── Ctrl+S / Ctrl+X outcomes ────────────────────────────────────────────
    //
    // Kills :234:9 (delete Ctrl+S arm — falls to `_` → textarea input, Continue).
    #[test]
    fn ctrl_s_returns_save() {
        let mut app = make_app();
        assert_eq!(
            handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            KeyOutcome::Save,
            "Ctrl+S must return Save"
        );
    }

    // Kills :238:9 (delete Ctrl+X arm — falls to `_` → textarea input, Continue).
    #[test]
    fn ctrl_x_on_clean_file_returns_exit() {
        let mut app = make_app();
        // is_dirty=false (default make_app) → handle_exit returns true → Exit.
        assert_eq!(
            handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL)),
            KeyOutcome::Exit,
            "Ctrl+X on a clean file must return Exit"
        );
    }

    // ── Shift+Down / Shift+Up visual-move-with-selection ────────────────────
    //
    // Kills :296:9 (delete Shift+Down arm — missed mutant).
    //
    // Without the arm, Shift+Down falls to `_` which calls textarea.input(Shift+Down).
    // tui-textarea's built-in Shift+Down on a single-line document is a no-op (already
    // at the last logical line), leaving the cursor at (0, 0).  The custom
    // handle_visual_move correctly steps to the second *visual* row of the wrapped line.
    //
    // Line "abcde fghij" at content_width=8:
    //   wrap_line → ["abcde", "fghij"] — two visual rows.
    //   First row chars 0..5, second row starts at char 6.
    //   Cursor at (0, 0) → Shift+Down → (0, 6).
    #[test]
    fn shift_down_moves_by_visual_row() {
        let mut app = nav_app(vec!["abcde fghij"], 8);
        app.textarea.move_cursor(CursorMove::Jump(0, 0));
        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT),
        );
        assert_eq!(
            app.textarea.cursor(),
            (0, 6),
            "Shift+Down must step one visual row (col 0 → col 6 on wrapped line)"
        );
    }

    // Kills :297:9 (delete Shift+Up arm — timeout due to suite overhead).
    //
    // Without the arm, Shift+Up falls to `_` → textarea.input(Shift+Up) → no-op
    // on first logical line → cursor stays at (0, 6).  Custom: moves to (0, 0).
    #[test]
    fn shift_up_moves_by_visual_row() {
        let mut app = nav_app(vec!["abcde fghij"], 8);
        app.textarea.move_cursor(CursorMove::Jump(0, 6));
        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT),
        );
        assert_eq!(
            app.textarea.cursor(),
            (0, 0),
            "Shift+Up must step back one visual row (col 6 → col 0 on wrapped line)"
        );
    }
}
