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

/// Map a screen-absolute (row, col) mouse position to a logical document
/// (row, col) position, accounting for the editor gutter, scroll offset, and
/// soft-wrapped lines. Returns `None` if the click is outside the editor area.
#[mutants::skip] // Terminal event loop — requires a real terminal backend.
pub(super) fn screen_to_doc(
    screen_row: u16,
    screen_col: u16,
    editor_area: &Rect,
    scroll_top: usize,
    lines: &[String],
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
        let wrapped = renderer::wrap_line(line, cw);
        let seg_count = wrapped.len().max(1);
        if vis + seg_count > click_vis_row {
            let si = click_vis_row - vis;
            let char_ranges = renderer::wrap_char_ranges(line, &wrapped);
            let seg_char_start = char_ranges.get(si).map_or(0, |&(start, _)| start);
            let doc_col = (seg_char_start + click_col).min(line.chars().count());
            return Some((li as u16, doc_col as u16));
        }
        vis += seg_count;
    }
    Some((lines.len().saturating_sub(1) as u16, 0))
}

/// Returns true if the key is a pure cursor-movement key that cannot change
/// document content. Used to skip the decoration debounce timer on nav presses.
pub(super) fn is_navigation_key(k: &crossterm::event::KeyEvent) -> bool {
    use crossterm::event::KeyModifiers;
    // Ctrl+Up/Down are viewport-scroll keys — they don't edit content.
    let ctrl_scroll = k.modifiers == KeyModifiers::CONTROL
        && matches!(k.code, KeyCode::Up | KeyCode::Down);
    ctrl_scroll
        || matches!(
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

pub(super) fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    layout_config: &LayoutConfig,
) -> io::Result<()> {
    const POLL_TIMEOUT: Duration = Duration::from_millis(16);
    const DEBOUNCE: Duration = Duration::from_millis(50);
    const BOTTOM_PADDING: usize = 3;
    const SCROLL_LINES: usize = 3;

    let min_cols = layout_config.min_cols.unwrap_or(DEFAULT_MIN_COLS);

    // Initial decoration pass.
    {
        let text = app.textarea.lines().join("\n");
        let (map, wc) = build_decoration_map(&text, &app.theme, app.italic_support);
        app.decoration_map = map;
        app.word_count = wc;
    }

    let mut last_editor_area = Rect::default();
    let mut drag_selecting = false;

    loop {
        if app.force_redecorate || app.last_keystroke.is_some_and(|t| t.elapsed() >= DEBOUNCE) {
            let text = app.textarea.lines().join("\n");
            let (map, wc) = build_decoration_map(&text, &app.theme, app.italic_support);
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
            // Clamp is skipped while the user is free-scrolling (mouse wheel or
            // Ctrl+Up/Down).  free_scroll persists across frames until any non-scroll
            // event clears it at the top of the event-poll block below.
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
            // Any event re-engages cursor-clamping scroll, except scroll events
            // themselves which immediately set free_scroll = true below.
            app.free_scroll = false;
            match event::read()? {
                Event::Key(k) => {
                    if matches!(app.status.mode, StatusMode::ExitPrompt) {
                        match k.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                handle_save(app)?;
                                break;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                break;
                            }
                            KeyCode::Esc
                            | KeyCode::Char('c')
                            | KeyCode::Char('C')
                            | KeyCode::Char('x')
                            | KeyCode::Char('X') => {
                                app.status.mode = StatusMode::Normal;
                            }
                            _ => {}
                        }
                    } else {
                        match (k.modifiers, k.code) {
                            (KeyModifiers::CONTROL, KeyCode::Char('s'))
                            | (KeyModifiers::SUPER, KeyCode::Char('s')) => {
                                handle_save(app)?;
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('x'))
                            | (KeyModifiers::NONE, KeyCode::Esc) => {
                                if handle_exit(app) {
                                    break;
                                }
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('c'))
                            | (KeyModifiers::SUPER, KeyCode::Char('c')) => {
                                yame::clipboard::handle_copy(app);
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('v'))
                            | (KeyModifiers::SUPER, KeyCode::Char('v')) => {
                                yame::clipboard::handle_paste(app);
                                app.force_redecorate = true;
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                                app.status.dismiss();
                                app.config_warnings.clear();
                                app.textarea.undo();
                                app.force_redecorate = true;
                                app.last_keystroke = Some(std::time::Instant::now());
                                app.recompute_dirty();
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                                app.status.dismiss();
                                app.config_warnings.clear();
                                app.textarea.redo();
                                app.force_redecorate = true;
                                app.last_keystroke = Some(std::time::Instant::now());
                                app.recompute_dirty();
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
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
                            // Ctrl+Up/Down: scroll viewport without moving cursor.
                            (KeyModifiers::CONTROL, KeyCode::Up) => {
                                app.scroll_top = app.scroll_top.saturating_sub(1);
                                app.free_scroll = true;
                            }
                            (KeyModifiers::CONTROL, KeyCode::Down) => {
                                let max = app.textarea.lines().len().saturating_sub(1);
                                app.scroll_top = (app.scroll_top + 1).min(max);
                                app.free_scroll = true;
                            }
                            _ => {
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
                            }
                        }
                    }
                }
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
                        drag_selecting = false;
                        if let Some((doc_row, doc_col)) = screen_to_doc(
                            mouse.row,
                            mouse.column,
                            &last_editor_area,
                            app.scroll_top,
                            app.textarea.lines(),
                        ) {
                            app.textarea.cancel_selection();
                            app.textarea.move_cursor(CursorMove::Jump(doc_row, doc_col));
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if let Some((doc_row, doc_col)) = screen_to_doc(
                            mouse.row,
                            mouse.column,
                            &last_editor_area,
                            app.scroll_top,
                            app.textarea.lines(),
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
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
