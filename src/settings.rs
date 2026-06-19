use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::config::{Config, Theme, parse_hex_color};

// ---------------------------------------------------------------------------
// Modal geometry — shared by renderer and hit-test logic
// ---------------------------------------------------------------------------

pub const MODAL_W: u16 = 56;
/// Number of field rows visible at once.
pub const VISIBLE_FIELDS: usize = 12;
/// Total modal height (top border + tab row + separator + fields + separator + hint + bottom border).
pub const MODAL_H: u16 = VISIBLE_FIELDS as u16 + 6;

// ---------------------------------------------------------------------------
// Tab / field descriptors
// ---------------------------------------------------------------------------

pub const TAB_NAMES: &[&str] = &["Appearance", "Editor", "File"];

/// Kind of settings field — controls how navigation and editing work.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    /// Boolean toggle — Space/Enter flips immediately.
    Toggle,
    /// Hex color string (`#rrggbb`) — Enter enters inline edit mode.
    HexColor,
    /// Decimal integer — Enter enters inline edit mode.  `allow_none` means an
    /// empty value is valid and serialises as `None`.
    Number { allow_none: bool },
    /// Arbitrary text — Enter enters inline edit mode.
    Text,
    /// Decimal float in 0.0–1.0 — Enter enters inline edit mode.
    Float,
    /// Fixed option list — Enter/←/→ cycles through options without entering
    /// edit mode.  Each option is committed to config immediately on selection.
    Cycler { options: &'static [&'static str] },
}

pub struct FieldDef {
    pub label: &'static str,
    pub kind: FieldKind,
}

// ---------------------------------------------------------------------------
// Cycler option lists
// ---------------------------------------------------------------------------

pub const PRESET_OPTIONS: &[&str] = &[
    "",
    "catppuccin-mocha",
    "catppuccin-macchiato",
    "catppuccin-frappe",
    "catppuccin-latte",
    "dracula",
    "nord",
    "gruvbox-dark",
    "rose-pine",
    "rose-pine-moon",
    "rose-pine-dawn",
    "solarized-dark",
    "solarized-light",
    "tokyo-night",
    "github-light",
];

pub const BLEND_OPTIONS: &[&str] = &[
    "0.0", "0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7", "0.8", "0.9", "1.0",
];

pub const UNKNOWN_AS_OPTIONS: &[&str] = &["markdown", "plain"];

// ---------------------------------------------------------------------------
// Appearance tab fields
// ---------------------------------------------------------------------------

pub const APPEARANCE_CORE: &[FieldDef] = &[
    FieldDef {
        label: "Preset",
        kind: FieldKind::Cycler {
            options: PRESET_OPTIONS,
        },
    },
    FieldDef {
        label: "Rainbow headings",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Text",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Accent",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Muted",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Code",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Background",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Warning",
        kind: FieldKind::HexColor,
    },
];

// sentinel index: the "Show more…" / "Show less…" row
pub const APPEARANCE_EXPAND_IDX: usize = APPEARANCE_CORE.len();

pub const APPEARANCE_EXTENDED: &[FieldDef] = &[
    FieldDef {
        label: "Bold color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Italic color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Strikethrough color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Blockquote color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Link text color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Link URL color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Done item color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Rule color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Code bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Fenced bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Heading bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Selection bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Selection fg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "UI bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "UI bar",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "UI text",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Delimiter blend",
        kind: FieldKind::Cycler {
            options: BLEND_OPTIONS,
        },
    },
    FieldDef {
        label: "Highlight bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Highlight fg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Frontmatter key",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "Frontmatter bg",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H1 color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H2 color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H3 color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H4 color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H5 color",
        kind: FieldKind::HexColor,
    },
    FieldDef {
        label: "H6 color",
        kind: FieldKind::HexColor,
    },
];

// ---------------------------------------------------------------------------
// Editor tab fields
// ---------------------------------------------------------------------------

pub const EDITOR_FIELDS: &[FieldDef] = &[
    FieldDef {
        label: "Line numbers",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Powerline glyphs",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Typewriter mode",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Focus mode",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Read-only",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Min columns",
        kind: FieldKind::Number { allow_none: false },
    },
    FieldDef {
        label: "Max columns",
        kind: FieldKind::Number { allow_none: true },
    },
    FieldDef {
        label: "Tab width",
        kind: FieldKind::Number { allow_none: false },
    },
];

// ---------------------------------------------------------------------------
// File tab fields
// ---------------------------------------------------------------------------

pub const FILE_FIELDS: &[FieldDef] = &[
    FieldDef {
        label: "Highlighting",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Use palette colors",
        kind: FieldKind::Toggle,
    },
    FieldDef {
        label: "Syntect theme",
        kind: FieldKind::Text,
    },
    FieldDef {
        label: "Unknown files as",
        kind: FieldKind::Cycler {
            options: UNKNOWN_AS_OPTIONS,
        },
    },
    FieldDef {
        label: "Extra MD extensions",
        kind: FieldKind::Text,
    },
];

// ---------------------------------------------------------------------------
// SettingsModal
// ---------------------------------------------------------------------------

/// All mutable state for the open settings panel.
pub struct SettingsModal {
    pub tab: usize,
    pub field: usize,
    pub editing: bool,
    pub input_buf: String,
    pub expanded: bool,
    /// Snapshot of runtime toggles so the Editor tab can show/edit them.
    pub typewriter_mode: bool,
    pub focus_mode: bool,
    pub read_only: bool,
    /// The working config being edited.  Committed to disk on each field change.
    pub config: Config,
    /// Resolved theme computed from `config` — used to show derived defaults for
    /// fields that are `None` in the config (i.e. inheriting from the preset or
    /// built-in defaults).  Refreshed after every `CommitSettings`.
    pub resolved: Theme,
    /// Scroll offset within the field list.
    pub scroll: usize,
}

impl SettingsModal {
    pub fn new(
        config: Config,
        resolved: Theme,
        typewriter_mode: bool,
        focus_mode: bool,
        read_only: bool,
    ) -> Self {
        Self {
            tab: 0,
            field: 0,
            editing: false,
            input_buf: String::new(),
            expanded: false,
            typewriter_mode,
            focus_mode,
            read_only,
            config,
            resolved,
            scroll: 0,
        }
    }

    /// Number of visible rows in the current tab (including the expand toggle row).
    pub fn field_count(&self) -> usize {
        match self.tab {
            0 => {
                // core + expand row + (extended if expanded)
                APPEARANCE_CORE.len()
                    + 1
                    + if self.expanded {
                        APPEARANCE_EXTENDED.len()
                    } else {
                        0
                    }
            }
            1 => EDITOR_FIELDS.len(),
            2 => FILE_FIELDS.len(),
            _ => 0,
        }
    }

    /// Field definition for the given (tab, field) index.  Returns `None` for
    /// the expand-toggle row.
    pub fn field_def(&self, field: usize) -> Option<&FieldDef> {
        match self.tab {
            0 => {
                if field < APPEARANCE_CORE.len() {
                    Some(&APPEARANCE_CORE[field])
                } else if field == APPEARANCE_EXPAND_IDX {
                    None // expand row
                } else {
                    APPEARANCE_EXTENDED.get(field - APPEARANCE_EXPAND_IDX - 1)
                }
            }
            1 => EDITOR_FIELDS.get(field),
            2 => FILE_FIELDS.get(field),
            _ => None,
        }
    }

    /// Human-readable current value string for a field.
    pub fn field_value(&self, field: usize) -> String {
        let cfg = &self.config;
        match self.tab {
            0 => {
                if field < APPEARANCE_CORE.len() {
                    appearance_core_value(cfg, &self.resolved, field)
                } else if field == APPEARANCE_EXPAND_IDX {
                    String::new() // handled separately by renderer
                } else {
                    appearance_extended_value(
                        cfg,
                        &self.resolved,
                        field - APPEARANCE_EXPAND_IDX - 1,
                    )
                }
            }
            1 => editor_value(self, field),
            2 => file_value(cfg, field),
            _ => String::new(),
        }
    }

    /// Parse `input_buf` and store into the config for the given tab/field.
    /// Returns `true` if the value was valid and stored.
    pub fn commit_input(&mut self) -> bool {
        let buf = self.input_buf.trim().to_string();
        match self.tab {
            0 => {
                if self.field < APPEARANCE_CORE.len() {
                    commit_appearance_core(&mut self.config, self.field, &buf)
                } else if self.field == APPEARANCE_EXPAND_IDX {
                    false
                } else {
                    commit_appearance_extended(
                        &mut self.config,
                        self.field - APPEARANCE_EXPAND_IDX - 1,
                        &buf,
                    )
                }
            }
            1 => commit_editor(self, &buf),
            2 => commit_file(&mut self.config, self.field, &buf),
            _ => false,
        }
    }

    /// Toggle the boolean field at the current position.  Returns `true` if a
    /// boolean was toggled (i.e. the field is a `Toggle` kind).
    pub fn toggle_current(&mut self) -> bool {
        match self.tab {
            0 => match self.field {
                1 => {
                    if let Some(preset) = &self.config.palette.preset.clone() {
                        self.config.palette.preset = if preset.ends_with("-expanded") {
                            Some(preset.trim_end_matches("-expanded").to_string())
                        } else {
                            Some(format!("{preset}-expanded"))
                        };
                        true
                    } else {
                        false // no preset active — nothing to expand
                    }
                }
                _ => false,
            },
            1 => match self.field {
                0 => {
                    let v = self.config.layout.line_numbers.unwrap_or(false);
                    self.config.layout.line_numbers = Some(!v);
                    true
                }
                1 => {
                    let v = self.config.layout.powerline_glyphs.unwrap_or(true);
                    self.config.layout.powerline_glyphs = Some(!v);
                    true
                }
                2 => {
                    self.typewriter_mode = !self.typewriter_mode;
                    true
                }
                3 => {
                    self.focus_mode = !self.focus_mode;
                    true
                }
                4 => {
                    self.read_only = !self.read_only;
                    true
                }
                _ => false,
            },
            2 => match self.field {
                0 => {
                    self.config.highlighting.enabled = !self.config.highlighting.enabled;
                    true
                }
                1 => {
                    self.config.highlighting.use_palette_colors =
                        !self.config.highlighting.use_palette_colors;
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Begin editing the current field: populate `input_buf` with the current value.
    pub fn start_editing(&mut self) {
        self.input_buf = self.field_value(self.field);
        self.editing = true;
    }

    /// Cancel editing without committing.
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.input_buf.clear();
    }

    /// Returns true when the currently focused field is a `Cycler`.
    pub fn is_cycler_field(&self) -> bool {
        matches!(
            self.field_def(self.field).map(|d| &d.kind),
            Some(FieldKind::Cycler { .. })
        )
    }

    /// Advance (`forward = true`) or retreat through a `Cycler` field's option
    /// list, committing the new value immediately.  Returns `true` on success.
    pub fn cycle_current(&mut self, forward: bool) -> bool {
        let options: &'static [&'static str] = {
            let Some(def) = self.field_def(self.field) else {
                return false;
            };
            match &def.kind {
                FieldKind::Cycler { options } => options,
                _ => return false,
            }
        };
        let current = self.field_value(self.field);
        let idx = options
            .iter()
            .position(|&o| o == current.as_str())
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % options.len()
        } else {
            (idx + options.len() - 1) % options.len()
        };
        self.input_buf = options[next].to_string();
        let ok = self.commit_input();
        self.input_buf.clear();
        ok
    }

    /// Move field cursor down, clamping to `field_count() - 1`.
    /// Also adjusts `scroll` to keep the focused field visible.
    pub fn move_down(&mut self, visible_rows: usize) {
        let max = self.field_count().saturating_sub(1);
        if self.field < max {
            self.field += 1;
            if self.field >= self.scroll + visible_rows {
                self.scroll = self.field + 1 - visible_rows;
            }
        }
    }

    /// Move field cursor up, adjusting scroll if needed.
    pub fn move_up(&mut self) {
        if self.field > 0 {
            self.field -= 1;
            if self.field < self.scroll {
                self.scroll = self.field;
            }
        }
    }

    /// Switch to the next tab, resetting field and scroll.
    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % TAB_NAMES.len();
        self.field = 0;
        self.scroll = 0;
        self.editing = false;
        self.input_buf.clear();
    }

    /// Switch to the previous tab, resetting field and scroll.
    pub fn prev_tab(&mut self) {
        self.tab = (self.tab + TAB_NAMES.len() - 1) % TAB_NAMES.len();
        self.field = 0;
        self.scroll = 0;
        self.editing = false;
        self.input_buf.clear();
    }

    /// Returns the resolved RGB for a color field's current value, for swatch
    /// rendering.  `None` when the value is absent, invalid, or the field is
    /// not a color.
    pub fn field_rgb(&self, field: usize) -> Option<(u8, u8, u8)> {
        let def = self.field_def(field)?;
        if def.kind != FieldKind::HexColor {
            return None;
        }
        let v = self.field_value(field);
        if v.is_empty() {
            return None;
        }
        parse_hex_color(&v).ok()
    }

    /// Returns the resolved RGB for `input_buf` (live preview while editing).
    pub fn input_rgb(&self) -> Option<(u8, u8, u8)> {
        parse_hex_color(self.input_buf.trim()).ok()
    }
}

// ---------------------------------------------------------------------------
// Mouse hit-test
// ---------------------------------------------------------------------------

/// Where a mouse event landed relative to the settings modal.
#[derive(Debug, PartialEq)]
pub enum ModalHit {
    /// A visible field row; carries the absolute field index.
    Field(usize),
    /// A tab label in the tab bar; carries the tab index.
    Tab(usize),
    /// Completely outside the modal's bounding rectangle.
    Outside,
    /// Inside the modal but on a non-interactive surface (border, separator, hint row).
    Inert,
}

impl SettingsModal {
    /// Classify a mouse position relative to the modal rendered over `editor_area`.
    pub fn hit_test(&self, row: u16, col: u16, editor_area: Rect) -> ModalHit {
        let bx = editor_area.x + (editor_area.width.saturating_sub(MODAL_W)) / 2;
        let by = editor_area.y + (editor_area.height.saturating_sub(MODAL_H)) / 2;

        // Outside modal bounds entirely.
        if col < bx || col >= bx + MODAL_W || row < by || row >= by + MODAL_H {
            return ModalHit::Outside;
        }

        // Tab bar row (by + 1).
        if row == by + 1 {
            if col < bx + 2 {
                return ModalHit::Inert; // left border column
            }
            let rel = (col - (bx + 2)) as usize;
            let mut x = 0usize;
            for (i, name) in TAB_NAMES.iter().enumerate() {
                let name_len = name.chars().count();
                if rel < x + name_len {
                    return ModalHit::Tab(i);
                }
                x += name_len;
                if i + 1 < TAB_NAMES.len() {
                    if rel < x + 3 {
                        return ModalHit::Inert; // clicked " · " separator
                    }
                    x += 3;
                }
            }
            return ModalHit::Inert;
        }

        // Field rows (by + 3 .. by + 3 + VISIBLE_FIELDS).
        let fields_start_y = by + 3;
        if row >= fields_start_y && row < fields_start_y + VISIBLE_FIELDS as u16 {
            let visual_row = (row - fields_start_y) as usize;
            let field_idx = self.scroll + visual_row;
            if field_idx < self.field_count() {
                return ModalHit::Field(field_idx);
            }
        }

        // Inside the modal bounds but on a non-interactive surface (top/bottom border,
        // separator rows, hint row, or a field row past the end of the list).
        ModalHit::Inert
    }

    /// Move cursor to `field`, cancelling any active edit.
    /// Adjusts `scroll` to keep the field visible.  No-op if out of range.
    pub fn jump_to_field(&mut self, field: usize) {
        if field >= self.field_count() {
            return;
        }
        if self.editing {
            self.cancel_editing();
        }
        self.field = field;
        if self.field < self.scroll {
            self.scroll = self.field;
        } else if self.field >= self.scroll + VISIBLE_FIELDS {
            self.scroll = self.field + 1 - VISIBLE_FIELDS;
        }
    }

    /// Activate the currently focused field on a mouse click: toggle booleans, advance cyclers,
    /// and toggle the expand row.  Text/color/number fields are left for keyboard Enter.
    pub fn activate_clicked_field(&mut self) -> SettingsOutcome {
        // Expand toggle row
        if self.tab == 0 && self.field == APPEARANCE_EXPAND_IDX {
            self.expanded = !self.expanded;
            if !self.expanded && self.field > APPEARANCE_EXPAND_IDX {
                self.field = APPEARANCE_EXPAND_IDX;
            }
            return SettingsOutcome::Continue;
        }
        // Cycler: advance one step forward
        if self.is_cycler_field() {
            return if self.cycle_current(true) {
                SettingsOutcome::Committed
            } else {
                SettingsOutcome::Continue
            };
        }
        // Toggle: flip
        if self.toggle_current() {
            return SettingsOutcome::Committed;
        }
        SettingsOutcome::Continue
    }

    /// Switch to `tab`, resetting field and scroll.  No-op if `tab` is already active.
    pub fn switch_to_tab(&mut self, tab: usize) {
        if tab >= TAB_NAMES.len() || tab == self.tab {
            return;
        }
        self.tab = tab;
        self.field = 0;
        self.scroll = 0;
        self.editing = false;
        self.input_buf.clear();
    }
}

// ---------------------------------------------------------------------------
// Key handler (pure — no I/O)
// ---------------------------------------------------------------------------

/// Result of handling a key inside the settings modal.
pub enum SettingsOutcome {
    /// Nothing to do beyond continuing the loop.
    Continue,
    /// User committed a field; caller should write + apply the config.
    Committed,
    /// Close the modal.
    Close,
}

/// Dispatch a key event inside the settings modal.
///
/// Mutates `modal` state and returns a `SettingsOutcome` telling the caller
/// what to do next.  The caller (event_loop) handles I/O (disk write, apply).
pub fn handle_settings_key(
    modal: &mut SettingsModal,
    k: crossterm::event::KeyEvent,
    visible_rows: usize,
) -> SettingsOutcome {
    // ── While editing a text / number / color field ────────────────────────
    if modal.editing {
        match (k.modifiers, k.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => {
                modal.cancel_editing();
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let ok = modal.commit_input();
                modal.editing = false;
                modal.input_buf.clear();
                if ok {
                    return SettingsOutcome::Committed;
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                modal.input_buf.pop();
            }
            (mods, KeyCode::Char(c))
                if !mods.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                ) =>
            {
                modal.input_buf.push(c);
            }
            _ => {}
        }
        return SettingsOutcome::Continue;
    }

    // ── Navigation / actions ────────────────────────────────────────────────
    match (k.modifiers, k.code) {
        // Close
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::F(2)) => {
            SettingsOutcome::Close
        }

        // Tab navigation — Tab always switches; ←/→ cycle on cycler fields, switch tab otherwise.
        (KeyModifiers::NONE, KeyCode::Tab) => {
            modal.next_tab();
            SettingsOutcome::Continue
        }
        (KeyModifiers::SHIFT, KeyCode::Tab) => {
            modal.prev_tab();
            SettingsOutcome::Continue
        }
        (KeyModifiers::NONE, KeyCode::Right) => {
            if modal.is_cycler_field() {
                if modal.cycle_current(true) {
                    SettingsOutcome::Committed
                } else {
                    SettingsOutcome::Continue
                }
            } else {
                modal.next_tab();
                SettingsOutcome::Continue
            }
        }
        (KeyModifiers::NONE, KeyCode::Left) => {
            if modal.is_cycler_field() {
                if modal.cycle_current(false) {
                    SettingsOutcome::Committed
                } else {
                    SettingsOutcome::Continue
                }
            } else {
                modal.prev_tab();
                SettingsOutcome::Continue
            }
        }

        // Field navigation
        (KeyModifiers::NONE, KeyCode::Down) => {
            modal.move_down(visible_rows);
            SettingsOutcome::Continue
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            modal.move_up();
            SettingsOutcome::Continue
        }

        // Activate / toggle
        (KeyModifiers::NONE, KeyCode::Enter) | (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            // Expand toggle row
            if modal.tab == 0 && modal.field == APPEARANCE_EXPAND_IDX {
                modal.expanded = !modal.expanded;
                if !modal.expanded && modal.field > APPEARANCE_EXPAND_IDX {
                    modal.field = APPEARANCE_EXPAND_IDX;
                }
                return SettingsOutcome::Continue;
            }

            // Cycler fields — Enter/Space advance to next option.
            if modal.is_cycler_field() {
                return if modal.cycle_current(true) {
                    SettingsOutcome::Committed
                } else {
                    SettingsOutcome::Continue
                };
            }

            // Boolean toggle fields
            if modal.toggle_current() {
                return SettingsOutcome::Committed;
            }

            // Text / number / color fields — enter edit mode
            if let Some(def) = modal.field_def(modal.field) {
                match def.kind {
                    FieldKind::Toggle | FieldKind::Cycler { .. } => {}
                    _ => modal.start_editing(),
                }
            }
            SettingsOutcome::Continue
        }

        _ => SettingsOutcome::Continue,
    }
}

// ---------------------------------------------------------------------------
// Color conversion helper
// ---------------------------------------------------------------------------

/// Convert a resolved `Color::Rgb` to its `#rrggbb` hex string.
/// Returns an empty string for non-Rgb variants (should not occur in practice).
pub fn color_to_hex(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Value getters per tab
// ---------------------------------------------------------------------------

/// Return the explicit config string if set, else the resolved theme colour.
fn opt_or_resolved(v: &Option<String>, resolved: Color) -> String {
    v.clone().unwrap_or_else(|| color_to_hex(resolved))
}

fn appearance_core_value(cfg: &Config, resolved: &Theme, field: usize) -> String {
    match field {
        0 => cfg
            .palette
            .preset
            .clone()
            .map(|p| p.trim_end_matches("-expanded").to_string())
            .unwrap_or_default(),
        1 => bool_str(
            cfg.palette
                .preset
                .as_deref()
                .is_some_and(|p| p.ends_with("-expanded")),
        ),
        2 => opt_or_resolved(&cfg.palette.text, resolved.text),
        3 => opt_or_resolved(&cfg.palette.accent, resolved.accent),
        4 => opt_or_resolved(&cfg.palette.muted, resolved.muted),
        5 => opt_or_resolved(&cfg.palette.code, resolved.code_color),
        6 => opt_or_resolved(&cfg.palette.bg, resolved.bg),
        7 => opt_or_resolved(&cfg.palette.warning, resolved.warning),
        _ => String::new(),
    }
}

fn appearance_extended_value(cfg: &Config, resolved: &Theme, idx: usize) -> String {
    match idx {
        0 => opt_or_resolved(&cfg.theme.bold_color, resolved.bold_color),
        1 => opt_or_resolved(&cfg.theme.italic_color, resolved.italic_color),
        2 => opt_or_resolved(&cfg.theme.strikethrough_color, resolved.strikethrough_color),
        3 => opt_or_resolved(&cfg.theme.blockquote_color, resolved.blockquote_color),
        4 => opt_or_resolved(&cfg.theme.link_text_color, resolved.link_text),
        5 => opt_or_resolved(&cfg.theme.link_url_color, resolved.link_url),
        6 => opt_or_resolved(&cfg.theme.todo_done, resolved.todo_done),
        7 => opt_or_resolved(&cfg.theme.rule_color, resolved.rule_color),
        8 => opt_or_resolved(&cfg.theme.code_bg, resolved.code_bg),
        9 => opt_or_resolved(&cfg.theme.fenced_bg, resolved.fenced_bg),
        10 => opt_or_resolved(&cfg.theme.heading_bg, resolved.heading_bg),
        11 => opt_or_resolved(&cfg.theme.selection_bg, resolved.selection_bg),
        12 => opt_or_resolved(&cfg.theme.selection_fg, resolved.selection_fg),
        13 => opt_or_resolved(&cfg.theme.ui_bg, resolved.ui_bg),
        14 => opt_or_resolved(&cfg.theme.ui_bar, resolved.ui_bar),
        15 => opt_or_resolved(&cfg.theme.ui_text, resolved.ui_text),
        16 => format!(
            "{:.1}",
            cfg.theme
                .delimiter_blend
                .unwrap_or(resolved.delimiter_blend)
        ),
        17 => opt_or_resolved(&cfg.theme.highlight_bg, resolved.highlight_bg),
        18 => opt_or_resolved(&cfg.theme.highlight_fg, resolved.highlight_fg),
        19 => opt_or_resolved(&cfg.theme.frontmatter_key, resolved.frontmatter_key),
        20 => opt_or_resolved(&cfg.theme.frontmatter_bg, resolved.frontmatter_bg),
        21 => opt_or_resolved(&cfg.headings.h1, resolved.headings.h1),
        22 => opt_or_resolved(&cfg.headings.h2, resolved.headings.h2),
        23 => opt_or_resolved(&cfg.headings.h3, resolved.headings.h3),
        24 => opt_or_resolved(&cfg.headings.h4, resolved.headings.h4),
        25 => opt_or_resolved(&cfg.headings.h5, resolved.headings.h5),
        26 => opt_or_resolved(&cfg.headings.h6, resolved.headings.h6),
        _ => String::new(),
    }
}

fn editor_value(modal: &SettingsModal, field: usize) -> String {
    let cfg = &modal.config;
    match field {
        0 => bool_str(cfg.layout.line_numbers.unwrap_or(false)),
        1 => bool_str(cfg.layout.powerline_glyphs.unwrap_or(true)),
        2 => bool_str(modal.typewriter_mode),
        3 => bool_str(modal.focus_mode),
        4 => bool_str(modal.read_only),
        5 => cfg
            .layout
            .min_cols
            .map(|n| n.to_string())
            .unwrap_or_else(|| "60".to_string()),
        6 => cfg
            .layout
            .max_cols
            .map(|n| n.to_string())
            .unwrap_or_else(|| "none (≤50% of terminal)".to_string()),
        7 => cfg
            .layout
            .tab_width
            .map(|n| n.to_string())
            .unwrap_or_else(|| "4".to_string()),
        _ => String::new(),
    }
}

fn file_value(cfg: &Config, field: usize) -> String {
    match field {
        0 => bool_str(cfg.highlighting.enabled),
        1 => bool_str(cfg.highlighting.use_palette_colors),
        2 => cfg.highlighting.syntect_theme.clone(),
        3 => cfg.filetype.unknown_as.clone(),
        4 => cfg.filetype.extra_markdown_extensions.join(", "),
        _ => String::new(),
    }
}

fn bool_str(b: bool) -> String {
    if b {
        "on".to_string()
    } else {
        "off".to_string()
    }
}

// ---------------------------------------------------------------------------
// Commit helpers
// ---------------------------------------------------------------------------

fn set_opt_hex(field: &mut Option<String>, v: &str) -> bool {
    if v.is_empty() {
        *field = None;
        return true;
    }
    if parse_hex_color(v).is_ok() {
        *field = Some(v.to_string());
        true
    } else {
        false
    }
}

fn commit_appearance_core(cfg: &mut Config, field: usize, v: &str) -> bool {
    match field {
        0 => {
            cfg.palette.preset = if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            };
            // When a preset is chosen, clear individual overrides so the preset colors apply.
            // (Individual fields win over the preset, so keeping them would make the cycle a no-op.)
            if cfg.palette.preset.is_some() {
                cfg.palette.text = None;
                cfg.palette.accent = None;
                cfg.palette.muted = None;
                cfg.palette.code = None;
                cfg.palette.bg = None;
                cfg.palette.warning = None;
            }
            true
        }
        1 => false, // Toggle — handled by toggle_current
        2 => set_opt_hex(&mut cfg.palette.text, v),
        3 => set_opt_hex(&mut cfg.palette.accent, v),
        4 => set_opt_hex(&mut cfg.palette.muted, v),
        5 => set_opt_hex(&mut cfg.palette.code, v),
        6 => set_opt_hex(&mut cfg.palette.bg, v),
        7 => set_opt_hex(&mut cfg.palette.warning, v),
        _ => false,
    }
}

fn commit_appearance_extended(cfg: &mut Config, idx: usize, v: &str) -> bool {
    match idx {
        0 => set_opt_hex(&mut cfg.theme.bold_color, v),
        1 => set_opt_hex(&mut cfg.theme.italic_color, v),
        2 => set_opt_hex(&mut cfg.theme.strikethrough_color, v),
        3 => set_opt_hex(&mut cfg.theme.blockquote_color, v),
        4 => set_opt_hex(&mut cfg.theme.link_text_color, v),
        5 => set_opt_hex(&mut cfg.theme.link_url_color, v),
        6 => set_opt_hex(&mut cfg.theme.todo_done, v),
        7 => set_opt_hex(&mut cfg.theme.rule_color, v),
        8 => set_opt_hex(&mut cfg.theme.code_bg, v),
        9 => set_opt_hex(&mut cfg.theme.fenced_bg, v),
        10 => set_opt_hex(&mut cfg.theme.heading_bg, v),
        11 => set_opt_hex(&mut cfg.theme.selection_bg, v),
        12 => set_opt_hex(&mut cfg.theme.selection_fg, v),
        13 => set_opt_hex(&mut cfg.theme.ui_bg, v),
        14 => set_opt_hex(&mut cfg.theme.ui_bar, v),
        15 => set_opt_hex(&mut cfg.theme.ui_text, v),
        16 => {
            if v.is_empty() {
                cfg.theme.delimiter_blend = None;
                return true;
            }
            if let Ok(f) = v.parse::<f32>() {
                cfg.theme.delimiter_blend = Some(f.clamp(0.0, 1.0));
                true
            } else {
                false
            }
        }
        17 => set_opt_hex(&mut cfg.theme.highlight_bg, v),
        18 => set_opt_hex(&mut cfg.theme.highlight_fg, v),
        19 => set_opt_hex(&mut cfg.theme.frontmatter_key, v),
        20 => set_opt_hex(&mut cfg.theme.frontmatter_bg, v),
        21 => set_opt_hex(&mut cfg.headings.h1, v),
        22 => set_opt_hex(&mut cfg.headings.h2, v),
        23 => set_opt_hex(&mut cfg.headings.h3, v),
        24 => set_opt_hex(&mut cfg.headings.h4, v),
        25 => set_opt_hex(&mut cfg.headings.h5, v),
        26 => set_opt_hex(&mut cfg.headings.h6, v),
        _ => false,
    }
}

fn commit_editor(modal: &mut SettingsModal, v: &str) -> bool {
    match modal.field {
        // toggles handled by toggle_current; shouldn't reach here
        0..=4 => false,
        5 => {
            if let Ok(n) = v.parse::<u16>() {
                modal.config.layout.min_cols = Some(n);
                true
            } else {
                false
            }
        }
        6 => {
            if v == "unlimited" || v.is_empty() {
                modal.config.layout.max_cols = None;
                true
            } else if let Ok(n) = v.parse::<u16>() {
                modal.config.layout.max_cols = Some(n);
                true
            } else {
                false
            }
        }
        7 => {
            if let Ok(n) = v.parse::<u16>() {
                modal.config.layout.tab_width = Some(n.max(1));
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn commit_file(cfg: &mut Config, field: usize, v: &str) -> bool {
    match field {
        // toggles handled by toggle_current
        0 | 1 => false,
        2 => {
            cfg.highlighting.syntect_theme = v.to_string();
            true
        }
        3 => {
            let lower = v.to_lowercase();
            if lower == "plain" || lower == "markdown" {
                cfg.filetype.unknown_as = lower;
                true
            } else {
                false
            }
        }
        4 => {
            cfg.filetype.extra_markdown_extensions = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            true
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use ratatui::layout::Rect;

    fn make_modal() -> SettingsModal {
        SettingsModal::new(
            Config::default(),
            Theme::default_theme(),
            false,
            false,
            false,
        )
    }

    #[test]
    fn field_count_appearance_core_only() {
        let m = make_modal();
        // APPEARANCE_CORE.len() + 1 expand row
        assert_eq!(m.field_count(), APPEARANCE_CORE.len() + 1);
    }

    #[test]
    fn field_count_appearance_expanded() {
        let mut m = make_modal();
        m.expanded = true;
        assert_eq!(
            m.field_count(),
            APPEARANCE_CORE.len() + 1 + APPEARANCE_EXTENDED.len()
        );
    }

    #[test]
    fn field_count_editor_tab() {
        let mut m = make_modal();
        m.tab = 1;
        assert_eq!(m.field_count(), EDITOR_FIELDS.len());
    }

    #[test]
    fn field_count_file_tab() {
        // Kills: `delete match arm 2` in field_count — tab=2 would return 0.
        let mut m = make_modal();
        m.tab = 2;
        assert_eq!(m.field_count(), FILE_FIELDS.len());
    }

    #[test]
    fn field_def_editor_tab_returns_some() {
        // Kills: `delete match arm 1` in field_def — tab=1 would return None.
        let mut m = make_modal();
        m.tab = 1;
        assert!(
            m.field_def(0).is_some(),
            "tab=1 field 0 must return Some from field_def"
        );
    }

    #[test]
    fn field_def_expand_row_returns_none() {
        // Kills: `replace < with <=` in field_def — the mutation would try
        // APPEARANCE_CORE[APPEARANCE_CORE.len()] → out-of-bounds panic.
        let m = make_modal(); // tab=0
        assert!(
            m.field_def(APPEARANCE_EXPAND_IDX).is_none(),
            "expand row must return None from field_def"
        );
    }

    #[test]
    fn field_value_editor_tab_returns_nonempty() {
        // Kills: `delete match arm 1` in field_value — tab=1 would return "".
        // field 5 = min_cols, defaults to "60" so we get a concrete non-empty value.
        let mut m = make_modal();
        m.tab = 1;
        assert_eq!(
            m.field_value(5),
            "60",
            "tab=1 field 5 (min_cols default) must return \"60\""
        );
    }

    #[test]
    fn field_value_expand_row_returns_empty() {
        // Kills: `replace < with <=` in field_value — the mutation would call
        // appearance_core_value with an out-of-bounds index, producing a panic or
        // wrong string instead of the expected empty string.
        let m = make_modal(); // tab=0
        assert_eq!(
            m.field_value(APPEARANCE_EXPAND_IDX),
            String::new(),
            "expand row must return empty string from field_value"
        );
    }

    #[test]
    fn toggle_line_numbers_flips() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 0;
        assert!(!m.config.layout.line_numbers.unwrap_or(false));
        m.toggle_current();
        assert!(m.config.layout.line_numbers.unwrap_or(false));
    }

    #[test]
    fn toggle_typewriter_mode_flips() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 2;
        assert!(!m.typewriter_mode);
        m.toggle_current();
        assert!(m.typewriter_mode);
    }

    #[test]
    fn commit_valid_hex_color_stores_value() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2; // text color
        m.input_buf = "#ff0000".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.text, Some("#ff0000".to_string()));
    }

    #[test]
    fn commit_invalid_hex_color_returns_false() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2;
        m.input_buf = "notacolor".to_string();
        assert!(!m.commit_input());
        assert!(m.config.palette.text.is_none());
    }

    #[test]
    fn commit_empty_hex_clears_option() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2;
        m.config.palette.text = Some("#ffffff".to_string());
        m.input_buf = String::new();
        assert!(m.commit_input());
        assert!(m.config.palette.text.is_none());
    }

    #[test]
    fn commit_max_cols_unlimited_clears_option() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 6;
        m.config.layout.max_cols = Some(80);
        m.input_buf = "unlimited".to_string();
        assert!(m.commit_input());
        assert!(m.config.layout.max_cols.is_none());
    }

    #[test]
    fn commit_tab_width_min_one() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 7;
        m.input_buf = "0".to_string();
        // 0 parses as u16 but max(1) clamps to 1
        assert!(m.commit_input());
        assert_eq!(m.config.layout.tab_width, Some(1));
    }

    #[test]
    fn field_rgb_returns_resolved_default_when_config_unset() {
        let m = make_modal();
        // palette.text is None → falls back to resolved theme (Mocha text #cdd6f4)
        let rgb = m.field_rgb(2);
        assert!(rgb.is_some(), "should show resolved default color");
        assert_eq!(rgb, Some((0xcd, 0xd6, 0xf4))); // Catppuccin Mocha text
    }

    #[test]
    fn field_rgb_returns_tuple_for_set_value() {
        let mut m = make_modal();
        m.config.palette.text = Some("#ff0000".to_string());
        assert_eq!(m.field_rgb(2), Some((255, 0, 0)));
    }

    #[test]
    fn move_down_clamps_to_last_field() {
        let mut m = make_modal();
        let max = m.field_count() - 1;
        for _ in 0..100 {
            m.move_down(20);
        }
        assert_eq!(m.field, max);
    }

    #[test]
    fn move_up_clamps_to_zero() {
        let mut m = make_modal();
        m.field = 3;
        for _ in 0..10 {
            m.move_up();
        }
        assert_eq!(m.field, 0);
    }

    #[test]
    fn next_tab_wraps_around() {
        let mut m = make_modal();
        m.tab = TAB_NAMES.len() - 1;
        m.next_tab();
        assert_eq!(m.tab, 0);
    }

    #[test]
    fn commit_file_unknown_as_plain() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 3;
        m.input_buf = "plain".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.filetype.unknown_as, "plain");
    }

    #[test]
    fn commit_file_unknown_as_invalid_returns_false() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 3;
        m.input_buf = "invalid".to_string();
        assert!(!m.commit_input());
    }

    #[test]
    fn toggle_read_only_flips() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 4;
        assert!(!m.read_only);
        m.toggle_current();
        assert!(m.read_only);
    }

    #[test]
    fn cycle_preset_forward_from_default() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0; // Preset
        assert_eq!(m.config.palette.preset, None);
        assert!(m.cycle_current(true));
        // "" → "catppuccin-mocha"
        assert_eq!(
            m.config.palette.preset,
            Some("catppuccin-mocha".to_string())
        );
    }

    #[test]
    fn cycle_preset_backward_from_default_wraps() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0;
        // "" is index 0; backward should wrap to last preset
        assert!(m.cycle_current(false));
        assert_eq!(
            m.config.palette.preset,
            Some(PRESET_OPTIONS[PRESET_OPTIONS.len() - 1].to_string())
        );
    }

    #[test]
    fn cycle_preset_to_empty_clears_option() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0;
        m.config.palette.preset = Some("catppuccin-mocha".to_string());
        // "catppuccin-mocha" is index 1; backward goes to index 0 = ""
        assert!(m.cycle_current(false));
        assert_eq!(m.config.palette.preset, None);
    }

    #[test]
    fn cycle_unknown_as_toggles_between_options() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 3; // Unknown files as
        assert_eq!(m.config.filetype.unknown_as, "markdown");
        assert!(m.cycle_current(true));
        assert_eq!(m.config.filetype.unknown_as, "plain");
        assert!(m.cycle_current(true));
        assert_eq!(m.config.filetype.unknown_as, "markdown");
    }

    #[test]
    fn cycle_blend_forward_advances_step() {
        let mut m = make_modal();
        m.tab = 0;
        m.expanded = true;
        // Delimiter blend is extended idx 16 → absolute field = APPEARANCE_EXPAND_IDX + 1 + 16
        m.field = APPEARANCE_EXPAND_IDX + 1 + 16;
        // default blend is 0.4 → next step is 0.5
        assert!(m.cycle_current(true));
        assert!((m.config.theme.delimiter_blend.unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn is_cycler_field_preset() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0;
        assert!(m.is_cycler_field());
    }

    #[test]
    fn is_cycler_field_false_for_hex() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2; // Text color (HexColor)
        assert!(!m.is_cycler_field());
    }

    #[test]
    fn rainbow_toggle_adds_expanded_suffix() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 1;
        m.config.palette.preset = Some("dracula".to_string());
        assert!(m.toggle_current());
        assert_eq!(
            m.config.palette.preset,
            Some("dracula-expanded".to_string())
        );
    }

    #[test]
    fn rainbow_toggle_removes_expanded_suffix() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 1;
        m.config.palette.preset = Some("dracula-expanded".to_string());
        assert!(m.toggle_current());
        assert_eq!(m.config.palette.preset, Some("dracula".to_string()));
    }

    #[test]
    fn rainbow_toggle_noop_when_no_preset() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 1;
        m.config.palette.preset = None;
        assert!(!m.toggle_current());
        assert_eq!(m.config.palette.preset, None);
    }

    #[test]
    fn preset_cycler_strips_expanded_for_display() {
        let mut m = make_modal();
        m.tab = 0;
        m.config.palette.preset = Some("nord-expanded".to_string());
        assert_eq!(m.field_value(0), "nord");
        // Rainbow toggle should read as on
        assert_eq!(m.field_value(1), "on");
    }

    #[test]
    fn commit_extra_md_extensions_splits_csv() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 4;
        m.input_buf = "txt, rst, wiki".to_string();
        assert!(m.commit_input());
        assert_eq!(
            m.config.filetype.extra_markdown_extensions,
            vec!["txt", "rst", "wiki"]
        );
    }

    // -------------------------------------------------------------------------
    // toggle_current — additional arms not covered above
    // -------------------------------------------------------------------------

    #[test]
    fn toggle_powerline_glyphs_flips() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 1; // powerline_glyphs (unwrap_or(true))
        assert!(m.toggle_current());
        assert_eq!(m.config.layout.powerline_glyphs, Some(false));
    }

    #[test]
    fn toggle_focus_mode_flips() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 3;
        assert!(!m.focus_mode);
        assert!(m.toggle_current());
        assert!(m.focus_mode);
    }

    #[test]
    fn toggle_highlighting_enabled_flips() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 0;
        let was = m.config.highlighting.enabled;
        assert!(m.toggle_current());
        assert_eq!(m.config.highlighting.enabled, !was);
    }

    #[test]
    fn toggle_use_palette_colors_flips() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 1;
        let was = m.config.highlighting.use_palette_colors;
        assert!(m.toggle_current());
        assert_eq!(m.config.highlighting.use_palette_colors, !was);
    }

    // -------------------------------------------------------------------------
    // start_editing / cancel_editing
    // -------------------------------------------------------------------------

    #[test]
    fn start_editing_sets_flag_and_populates_buf() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 5; // min_cols → "60" by default
        m.start_editing();
        assert!(m.editing);
        assert_eq!(m.input_buf, "60");
    }

    #[test]
    fn cancel_editing_clears_flag_and_buf() {
        let mut m = make_modal();
        m.editing = true;
        m.input_buf = "partial".to_string();
        m.cancel_editing();
        assert!(!m.editing);
        assert!(m.input_buf.is_empty());
    }

    // -------------------------------------------------------------------------
    // move_down / move_up scroll tracking
    // -------------------------------------------------------------------------

    #[test]
    fn move_down_scroll_advances_when_field_leaves_window() {
        let mut m = make_modal();
        // visible_rows = 2: window is scroll..scroll+2
        // After 1st down: field=1, still in window (1 < 0+2). scroll stays 0.
        m.move_down(2);
        assert_eq!(m.field, 1);
        assert_eq!(m.scroll, 0);
        // After 2nd down: field=2 >= 0+2 → scroll = 2+1-2 = 1.
        m.move_down(2);
        assert_eq!(m.field, 2);
        assert_eq!(m.scroll, 1);
    }

    #[test]
    fn move_up_scroll_retreats_when_field_leaves_window() {
        let mut m = make_modal();
        m.field = 3;
        m.scroll = 2;
        // field → 2: 2 < 2? No. scroll stays 2.
        m.move_up();
        assert_eq!(m.field, 2);
        assert_eq!(m.scroll, 2);
        // field → 1: 1 < 2? Yes. scroll becomes 1.
        m.move_up();
        assert_eq!(m.field, 1);
        assert_eq!(m.scroll, 1);
    }

    // -------------------------------------------------------------------------
    // prev_tab
    // -------------------------------------------------------------------------

    #[test]
    fn prev_tab_decrements_tab() {
        let mut m = make_modal();
        m.tab = 1;
        m.prev_tab();
        assert_eq!(m.tab, 0);
    }

    #[test]
    fn prev_tab_wraps_from_zero() {
        let mut m = make_modal();
        m.tab = 0;
        m.prev_tab();
        assert_eq!(m.tab, TAB_NAMES.len() - 1);
    }

    #[test]
    fn prev_tab_resets_field_scroll_and_editing() {
        let mut m = make_modal();
        m.tab = 2;
        m.field = 3;
        m.scroll = 2;
        m.editing = true;
        m.input_buf = "abc".to_string();
        m.prev_tab();
        assert_eq!(m.field, 0);
        assert_eq!(m.scroll, 0);
        assert!(!m.editing);
        assert!(m.input_buf.is_empty());
    }

    // -------------------------------------------------------------------------
    // input_rgb
    // -------------------------------------------------------------------------

    #[test]
    fn input_rgb_valid_hex_returns_tuple() {
        let mut m = make_modal();
        m.input_buf = "#abcdef".to_string();
        assert_eq!(m.input_rgb(), Some((0xab, 0xcd, 0xef)));
    }

    #[test]
    fn input_rgb_invalid_returns_none() {
        let mut m = make_modal();
        m.input_buf = "notacolor".to_string();
        assert_eq!(m.input_rgb(), None);
    }

    // -------------------------------------------------------------------------
    // handle_settings_key helpers
    // -------------------------------------------------------------------------

    fn key(code: KeyCode) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_mod(code: KeyCode, mods: KeyModifiers) -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(code, mods)
    }

    // -------------------------------------------------------------------------
    // handle_settings_key — editing mode
    // -------------------------------------------------------------------------

    #[test]
    fn hsk_esc_while_editing_cancels() {
        let mut m = make_modal();
        m.editing = true;
        m.input_buf = "abc".to_string();
        handle_settings_key(&mut m, key(KeyCode::Esc), 10);
        assert!(!m.editing);
        assert!(m.input_buf.is_empty());
    }

    #[test]
    fn hsk_enter_while_editing_commits_valid_hex() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2; // text color (HexColor)
        m.editing = true;
        m.input_buf = "#123456".to_string();
        let outcome = handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(matches!(outcome, SettingsOutcome::Committed));
        assert!(!m.editing);
        assert_eq!(m.config.palette.text, Some("#123456".to_string()));
    }

    #[test]
    fn hsk_backspace_while_editing_pops_char() {
        let mut m = make_modal();
        m.editing = true;
        m.input_buf = "abc".to_string();
        handle_settings_key(&mut m, key(KeyCode::Backspace), 10);
        assert_eq!(m.input_buf, "ab");
    }

    #[test]
    fn hsk_char_while_editing_pushes() {
        let mut m = make_modal();
        m.editing = true;
        handle_settings_key(&mut m, key(KeyCode::Char('x')), 10);
        assert_eq!(m.input_buf, "x");
    }

    #[test]
    fn hsk_ctrl_char_while_editing_ignored() {
        let mut m = make_modal();
        m.editing = true;
        m.input_buf = "ab".to_string();
        handle_settings_key(
            &mut m,
            key_mod(KeyCode::Char('c'), KeyModifiers::CONTROL),
            10,
        );
        assert_eq!(m.input_buf, "ab");
    }

    // -------------------------------------------------------------------------
    // handle_settings_key — nav mode
    // -------------------------------------------------------------------------

    #[test]
    fn hsk_esc_nav_returns_close() {
        let mut m = make_modal();
        assert!(matches!(
            handle_settings_key(&mut m, key(KeyCode::Esc), 10),
            SettingsOutcome::Close
        ));
    }

    #[test]
    fn hsk_f2_nav_returns_close() {
        let mut m = make_modal();
        assert!(matches!(
            handle_settings_key(&mut m, key(KeyCode::F(2)), 10),
            SettingsOutcome::Close
        ));
    }

    #[test]
    fn hsk_tab_advances_tab() {
        let mut m = make_modal();
        handle_settings_key(&mut m, key(KeyCode::Tab), 10);
        assert_eq!(m.tab, 1);
    }

    #[test]
    fn hsk_shift_tab_retreats_tab() {
        let mut m = make_modal();
        m.tab = 1;
        handle_settings_key(&mut m, key_mod(KeyCode::Tab, KeyModifiers::SHIFT), 10);
        assert_eq!(m.tab, 0);
    }

    #[test]
    fn hsk_right_on_non_cycler_advances_tab() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 0; // line_numbers (Toggle, not cycler)
        handle_settings_key(&mut m, key(KeyCode::Right), 10);
        assert_eq!(m.tab, 2);
    }

    #[test]
    fn hsk_left_on_non_cycler_retreats_tab() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 0;
        handle_settings_key(&mut m, key(KeyCode::Left), 10);
        assert_eq!(m.tab, 0);
    }

    #[test]
    fn hsk_right_on_cycler_cycles_and_commits() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0; // Preset cycler
        m.config.palette.preset = Some("dracula".to_string());
        let outcome = handle_settings_key(&mut m, key(KeyCode::Right), 10);
        assert!(matches!(outcome, SettingsOutcome::Committed));
        assert_eq!(m.tab, 0); // tab must NOT have advanced
    }

    #[test]
    fn hsk_down_moves_field() {
        let mut m = make_modal();
        handle_settings_key(&mut m, key(KeyCode::Down), 10);
        assert_eq!(m.field, 1);
    }

    #[test]
    fn hsk_up_moves_field() {
        let mut m = make_modal();
        m.field = 2;
        handle_settings_key(&mut m, key(KeyCode::Up), 10);
        assert_eq!(m.field, 1);
    }

    #[test]
    fn hsk_enter_on_expand_row_toggles_expanded() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = APPEARANCE_EXPAND_IDX;
        handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(m.expanded);
        handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(!m.expanded);
    }

    #[test]
    fn hsk_enter_on_non_expand_row_does_not_toggle_expanded() {
        // Kills `&& → ||` at line 509: with ||, any field on tab=0 triggers expand.
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0; // Preset cycler — NOT the expand row
        handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(!m.expanded);
    }

    #[test]
    fn hsk_enter_on_toggle_field_no_effect_does_not_start_editing() {
        // Kills `delete match arm Toggle | Cycler` at line 534: rainbow toggle with no
        // preset set returns false from toggle_current, reaching the field_def match.
        // Without the Toggle arm, start_editing would be called instead.
        let mut m = make_modal();
        m.tab = 0;
        m.field = 1; // rainbow headings Toggle; no preset → toggle_current returns false
        handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(!m.editing);
    }

    #[test]
    fn hsk_enter_on_toggle_field_with_effect_returns_committed() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 0; // line_numbers Toggle
        let outcome = handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(matches!(outcome, SettingsOutcome::Committed));
        assert!(!m.editing);
    }

    #[test]
    fn hsk_enter_on_hex_field_starts_editing() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2; // text color (HexColor)
        handle_settings_key(&mut m, key(KeyCode::Enter), 10);
        assert!(m.editing);
    }

    // -------------------------------------------------------------------------
    // Value getters — appearance_core, extended, editor, file
    // -------------------------------------------------------------------------

    #[test]
    fn appearance_core_value_color_fields_nonempty() {
        // Kills delete-match-arm mutations for fields 3-7 (accent–warning).
        let m = make_modal();
        for f in 3..=7usize {
            let v = m.field_value(f);
            assert!(
                !v.is_empty(),
                "appearance_core field {f} must return non-empty value"
            );
        }
    }

    #[test]
    fn appearance_extended_value_all_fields_nonempty() {
        // Kills delete-match-arm mutations for all 27 extended fields.
        let m = make_modal(); // tab=0; field_value works without expanded=true
        for idx in 0..APPEARANCE_EXTENDED.len() {
            let f = APPEARANCE_EXPAND_IDX + 1 + idx;
            let v = m.field_value(f);
            assert!(
                !v.is_empty(),
                "appearance_extended idx {idx} must return non-empty value"
            );
        }
    }

    #[test]
    fn editor_value_all_fields() {
        // Kills delete-match-arm for arms 0-4 (bools) and 6-7 (max_cols, tab_width).
        // Arm 5 (min_cols → "60") is already covered by field_value_editor_tab_returns_nonempty.
        let mut m = make_modal();
        m.tab = 1;
        // Arms 0-4: any non-empty bool string is enough; exact values from code defaults.
        assert_eq!(m.field_value(0), "off", "line_numbers default");
        assert_eq!(m.field_value(1), "on", "powerline_glyphs default");
        assert_eq!(m.field_value(2), "off", "typewriter_mode");
        assert_eq!(m.field_value(3), "off", "focus_mode");
        assert_eq!(m.field_value(4), "off", "read_only");
        assert_eq!(
            m.field_value(6),
            "none (≤50% of terminal)",
            "max_cols default"
        );
        assert_eq!(m.field_value(7), "4", "tab_width default");
    }

    #[test]
    fn file_value_all_fields() {
        // Kills delete-match-arm for arms 0, 1, 2, 4 (arm 3 caught by cycle test).
        let mut m = make_modal();
        m.tab = 2;
        m.config.highlighting.enabled = true;
        m.config.highlighting.use_palette_colors = true;
        m.config.highlighting.syntect_theme = "InspiredGitHub".to_string();
        m.config.filetype.extra_markdown_extensions = vec!["txt".to_string()];
        assert_eq!(m.field_value(0), "on", "highlighting.enabled");
        assert_eq!(m.field_value(1), "on", "use_palette_colors");
        assert_eq!(m.field_value(2), "InspiredGitHub", "syntect_theme");
        assert_eq!(m.field_value(4), "txt", "extra_md_extensions");
    }

    // -------------------------------------------------------------------------
    // Commit helpers — appearance_core fields 3-7, extended all, editor arm 5
    // -------------------------------------------------------------------------

    #[test]
    fn commit_appearance_core_fields_3_to_7() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 3;
        m.input_buf = "#aabbcc".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.accent, Some("#aabbcc".to_string()));
        m.field = 4;
        m.input_buf = "#aabbcc".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.muted, Some("#aabbcc".to_string()));
        m.field = 5;
        m.input_buf = "#aabbcc".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.code, Some("#aabbcc".to_string()));
        m.field = 6;
        m.input_buf = "#aabbcc".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.bg, Some("#aabbcc".to_string()));
        m.field = 7;
        m.input_buf = "#aabbcc".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.palette.warning, Some("#aabbcc".to_string()));
    }

    #[test]
    fn commit_appearance_extended_all_hex_fields() {
        // Kills delete-match-arm for extended indices 0-15 and 17-26.
        let mut m = make_modal();
        m.tab = 0;
        m.expanded = true;
        for idx in (0..=15usize).chain(17..=26) {
            m.field = APPEARANCE_EXPAND_IDX + 1 + idx;
            m.input_buf = "#fedcba".to_string();
            assert!(m.commit_input(), "commit extended idx {idx} should succeed");
        }
    }

    #[test]
    fn commit_appearance_extended_blend_field() {
        let mut m = make_modal();
        m.tab = 0;
        m.expanded = true;
        m.field = APPEARANCE_EXPAND_IDX + 1 + 16;
        m.input_buf = "0.5".to_string();
        assert!(m.commit_input());
        assert!((m.config.theme.delimiter_blend.unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn commit_editor_min_cols_valid() {
        // Kills TIMEOUT `delete match arm 5` in commit_editor.
        let mut m = make_modal();
        m.tab = 1;
        m.field = 5;
        m.input_buf = "80".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.layout.min_cols, Some(80));
    }

    #[test]
    fn commit_file_syntect_theme() {
        // Kills TIMEOUT `delete match arm 2` in commit_file.
        let mut m = make_modal();
        m.tab = 2;
        m.field = 2;
        m.input_buf = "base16-ocean.dark".to_string();
        assert!(m.commit_input());
        assert_eq!(m.config.highlighting.syntect_theme, "base16-ocean.dark");
    }

    // -------------------------------------------------------------------------
    // hit_test
    // -------------------------------------------------------------------------

    // editor_area: 120 wide × 40 tall at (0, 0)
    // bx = (120 - 56) / 2 = 32,  by = (40 - 18) / 2 = 11
    // tab row: y=12
    //   "Appearance" x=34..43, " · " x=44..46, "Editor" x=47..52, " · " x=53..55, "File" x=56..59
    // field rows: y=14..25  (by+3 .. by+3+12)
    fn test_area() -> Rect {
        Rect { x: 0, y: 0, width: 120, height: 40 }
    }

    #[test]
    fn hit_test_outside_returns_outside() {
        let m = make_modal();
        assert_eq!(m.hit_test(0, 0, test_area()), ModalHit::Outside);
    }

    #[test]
    fn hit_test_top_border_returns_inert() {
        // by = 11 — top border is inside the modal rectangle but non-interactive.
        let m = make_modal();
        assert_eq!(m.hit_test(11, 40, test_area()), ModalHit::Inert);
    }

    #[test]
    fn hit_test_separator_row_returns_inert() {
        // by+2 = 13 — separator between tab bar and fields: inside modal, non-interactive.
        let m = make_modal();
        assert_eq!(m.hit_test(13, 40, test_area()), ModalHit::Inert);
    }

    #[test]
    fn hit_test_field_row_0_returns_field_0() {
        let m = make_modal();
        assert_eq!(m.hit_test(14, 40, test_area()), ModalHit::Field(0));
    }

    #[test]
    fn hit_test_field_row_3_returns_field_3() {
        let m = make_modal();
        assert_eq!(m.hit_test(17, 40, test_area()), ModalHit::Field(3));
    }

    #[test]
    fn hit_test_field_row_with_scroll_offset() {
        // scroll=3: visual row 0 (y=14) maps to absolute field 3.
        let mut m = make_modal();
        m.scroll = 3;
        assert_eq!(m.hit_test(14, 40, test_area()), ModalHit::Field(3));
    }

    #[test]
    fn hit_test_tab_appearance() {
        let m = make_modal();
        // "Appearance" starts at bx+2=34; click col 36 is inside it.
        assert_eq!(m.hit_test(12, 36, test_area()), ModalHit::Tab(0));
    }

    #[test]
    fn hit_test_tab_editor() {
        let m = make_modal();
        // "Editor" starts at bx+2+13=47; click col 48 is inside it.
        assert_eq!(m.hit_test(12, 48, test_area()), ModalHit::Tab(1));
    }

    #[test]
    fn hit_test_tab_file() {
        let m = make_modal();
        // "File" starts at bx+2+22=56; click col 57 is inside it.
        assert_eq!(m.hit_test(12, 57, test_area()), ModalHit::Tab(2));
    }

    #[test]
    fn hit_test_tab_separator_returns_inert() {
        let m = make_modal();
        // Separator " · " between Appearance and Editor: col 44 (bx+12) — inside modal, non-interactive.
        assert_eq!(m.hit_test(12, 44, test_area()), ModalHit::Inert);
    }

    // -------------------------------------------------------------------------
    // jump_to_field
    // -------------------------------------------------------------------------

    #[test]
    fn jump_to_field_sets_field() {
        let mut m = make_modal();
        m.jump_to_field(3);
        assert_eq!(m.field, 3);
    }

    #[test]
    fn jump_to_field_cancels_active_edit() {
        let mut m = make_modal();
        m.editing = true;
        m.input_buf = "abc".to_string();
        m.jump_to_field(2);
        assert!(!m.editing);
        assert!(m.input_buf.is_empty());
    }

    #[test]
    fn jump_to_field_scrolls_forward_when_out_of_window() {
        // VISIBLE_FIELDS=12; jump to field 14 from scroll=0 → scroll = 14+1-12 = 3.
        let mut m = make_modal();
        m.tab = 0;
        m.expanded = true; // expose enough fields
        m.jump_to_field(14);
        assert_eq!(m.field, 14);
        assert_eq!(m.scroll, 3);
    }

    #[test]
    fn jump_to_field_scrolls_backward_when_above_window() {
        let mut m = make_modal();
        m.scroll = 5;
        m.field = 5;
        m.jump_to_field(2);
        assert_eq!(m.field, 2);
        assert_eq!(m.scroll, 2);
    }

    #[test]
    fn jump_to_field_out_of_range_is_noop() {
        let mut m = make_modal();
        let original_field = m.field;
        m.jump_to_field(9999);
        assert_eq!(m.field, original_field);
    }

    // -------------------------------------------------------------------------
    // switch_to_tab
    // -------------------------------------------------------------------------

    #[test]
    fn switch_to_tab_changes_tab() {
        let mut m = make_modal();
        m.switch_to_tab(1);
        assert_eq!(m.tab, 1);
    }

    #[test]
    fn switch_to_tab_resets_field_and_scroll() {
        let mut m = make_modal();
        m.field = 4;
        m.scroll = 2;
        m.editing = true;
        m.input_buf = "x".to_string();
        m.switch_to_tab(2);
        assert_eq!(m.tab, 2);
        assert_eq!(m.field, 0);
        assert_eq!(m.scroll, 0);
        assert!(!m.editing);
        assert!(m.input_buf.is_empty());
    }

    #[test]
    fn switch_to_tab_noop_on_same_tab() {
        let mut m = make_modal();
        m.field = 3;
        m.scroll = 1;
        m.switch_to_tab(0); // already on tab 0
        assert_eq!(m.field, 3); // unchanged
        assert_eq!(m.scroll, 1); // unchanged
    }

    #[test]
    fn switch_to_tab_out_of_range_is_noop() {
        let mut m = make_modal();
        m.switch_to_tab(99);
        assert_eq!(m.tab, 0); // unchanged
    }

    // -------------------------------------------------------------------------
    // hit_test — Inert / Outside boundary cases
    // -------------------------------------------------------------------------

    #[test]
    fn hit_test_truly_outside_returns_outside() {
        // col=0 is left of bx=32 → truly outside the modal rectangle.
        let m = make_modal();
        assert_eq!(m.hit_test(14, 0, test_area()), ModalHit::Outside);
    }

    #[test]
    fn hit_test_hint_row_returns_inert() {
        // by+16 = 27 — the navigation hint row below the second separator.
        let m = make_modal();
        assert_eq!(m.hit_test(27, 40, test_area()), ModalHit::Inert);
    }

    #[test]
    fn hit_test_bottom_border_returns_inert() {
        // by+17 = 28 — bottom border row.
        let m = make_modal();
        assert_eq!(m.hit_test(28, 40, test_area()), ModalHit::Inert);
    }

    // -------------------------------------------------------------------------
    // activate_clicked_field
    // -------------------------------------------------------------------------

    #[test]
    fn activate_clicked_field_toggle_flips_and_commits() {
        let mut m = make_modal();
        m.tab = 1;
        m.field = 0; // line_numbers Toggle
        let outcome = m.activate_clicked_field();
        assert!(matches!(outcome, SettingsOutcome::Committed));
        assert!(m.config.layout.line_numbers.unwrap_or(false));
    }

    #[test]
    fn activate_clicked_field_cycler_advances_and_commits() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = 0; // Preset cycler — starts at "" (index 0)
        let outcome = m.activate_clicked_field();
        assert!(matches!(outcome, SettingsOutcome::Committed));
        assert_eq!(
            m.config.palette.preset,
            Some("catppuccin-mocha".to_string())
        );
    }

    #[test]
    fn activate_clicked_field_expand_row_toggles_expanded() {
        let mut m = make_modal();
        m.tab = 0;
        m.field = APPEARANCE_EXPAND_IDX;
        let outcome = m.activate_clicked_field();
        assert!(matches!(outcome, SettingsOutcome::Continue));
        assert!(m.expanded);
    }

    #[test]
    fn activate_clicked_field_hex_field_returns_continue_no_edit() {
        // Text/color fields should not start editing on a bare click.
        let mut m = make_modal();
        m.tab = 0;
        m.field = 2; // Text color (HexColor)
        let outcome = m.activate_clicked_field();
        assert!(matches!(outcome, SettingsOutcome::Continue));
        assert!(!m.editing);
    }
}
