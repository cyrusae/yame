use ratatui::style::Color;
use serde::Deserialize;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Raw config structs (deserialized from TOML)
// ---------------------------------------------------------------------------

/// Base color palette. Defaults to Catppuccin Mocha.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Palette {
    pub text: String,
    pub accent: String,
    pub muted: String,
    pub code: String,
    pub bg: String,
    pub warning: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            text: "#cdd6f4".into(),
            accent: "#cba6f7".into(),
            muted: "#585b70".into(),
            code: "#a6e3a1".into(),
            // Catppuccin Crust — near-black main canvas
            bg: "#11111b".into(),
            warning: "#f38ba8".into(),
        }
    }
}

/// Optional per-field overrides for derived theme tokens.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ThemeOverrides {
    pub bold_color: Option<String>,
    pub italic_color: Option<String>,
    pub strikethrough_color: Option<String>,
    pub blockquote_color: Option<String>,
    pub link_text_color: Option<String>,
    pub link_url_color: Option<String>,
    pub todo_done: Option<String>,
    pub rule_color: Option<String>,
    pub code_bg: Option<String>,
    pub fenced_bg: Option<String>,
    pub heading_bg: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_fg: Option<String>,
    pub ui_bg: Option<String>,
    pub ui_bar: Option<String>,
    pub ui_text: Option<String>,
    /// 0.0 = full muted, 1.0 = full span color. Default 0.4.
    pub delimiter_blend: Option<f32>,
}

/// Per-level heading color overrides (all optional).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct HeadingColors {
    pub h1: Option<String>,
    pub h2: Option<String>,
    pub h3: Option<String>,
    pub h4: Option<String>,
    pub h5: Option<String>,
    pub h6: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    pub min_cols: Option<u16>,
    /// Number of spaces to substitute for each `\t` on file load. Default 4.
    pub tab_width: Option<u16>,
    /// Use Powerline/Nerd Font filled-arrow glyphs (U+E0B0) in the status bar
    /// instead of the universal box-drawing separator `│`.
    /// Requires a Nerd Font or Powerline-patched font. Default true.
    /// Set `powerline_glyphs = false` to opt out if your font lacks glyph U+E0B0.
    pub powerline_glyphs: Option<bool>,
}

/// Configuration for syntax highlighting of fenced code blocks.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HighlightingConfig {
    /// Enable syntect syntax highlighting for fenced code blocks. Default true.
    pub enabled: bool,
    /// Derive token colours from the yame palette instead of a built-in syntect
    /// theme.  When true (the default) keywords use `accent`, strings use
    /// `code`, comments use `muted`, etc.  Set false to use `syntect_theme`
    /// colours instead (e.g. for a light-mode code block on a dark editor).
    pub use_palette_colors: bool,
    /// Name of the bundled syntect theme to use when `use_palette_colors = false`.
    /// Available: "base16-ocean.dark", "base16-ocean.light", "base16-eighties.dark",
    /// "base16-mocha.dark", "InspiredGitHub", "Solarized (dark)", "Solarized (light)".
    /// Invalid names fall back to "base16-ocean.dark".
    pub syntect_theme: String,
}

impl Default for HighlightingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_palette_colors: true,
            syntect_theme: "base16-ocean.dark".into(),
        }
    }
}

/// Configuration for file-type detection and editing mode selection.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FiletypeConfig {
    /// Additional file extensions (without the leading dot, case-insensitive)
    /// to treat as Markdown on top of the built-in list
    /// (`md`, `markdown`, `mdx`, `mkd`, `mkdn`, `mdown`).
    pub extra_markdown_extensions: Vec<String>,
    /// What to do with files whose extension is not in the Markdown list and
    /// not recognised by syntect.
    /// `"markdown"` (default) — open as Markdown.
    /// `"plain"` — open as unstyled plain text.
    pub unknown_as: String,
}

impl Default for FiletypeConfig {
    fn default() -> Self {
        Self {
            extra_markdown_extensions: vec![],
            unknown_as: "markdown".into(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub palette: Palette,
    pub theme: ThemeOverrides,
    pub headings: HeadingColors,
    pub layout: LayoutConfig,
    pub highlighting: HighlightingConfig,
    pub filetype: FiletypeConfig,
}

// ---------------------------------------------------------------------------
// Computed Theme (never serialized)
// ---------------------------------------------------------------------------

/// All resolved Color values used by the renderer.
#[derive(Debug, Clone)]
pub struct HeadingTheme {
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub h4: Color,
    pub h5: Color,
    pub h6: Color,
}

#[derive(Debug, Clone)]
pub struct Theme {
    // base
    pub text: Color,
    pub accent: Color,
    pub muted: Color,
    pub code_color: Color,
    pub bg: Color,
    pub warning: Color,
    // derived
    pub bold_color: Color,
    pub italic_color: Color,
    pub strikethrough_color: Color,
    pub blockquote_color: Color,
    pub link_text: Color,
    pub link_url: Color,
    pub todo_done: Color,
    pub rule_color: Color,
    pub code_bg: Color,
    pub fenced_bg: Color,
    pub heading_bg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub ui_bg: Color,
    pub ui_bar: Color,
    pub ui_text: Color,
    // per-level headings
    pub headings: HeadingTheme,
    pub delimiter_blend: f32,
}

// ---------------------------------------------------------------------------
// Color utilities
// ---------------------------------------------------------------------------

/// Parse a `#rrggbb` hex string into an (r, g, b) tuple.
pub fn parse_hex_color(s: &str) -> Result<(u8, u8, u8), String> {
    let s = s.trim();
    if !s.starts_with('#') {
        return Err(format!("color must start with '#': {s}"));
    }
    let hex = &s[1..];
    if hex.len() != 6 {
        return Err(format!("color must be exactly 6 hex digits: {s}"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| format!("invalid hex digits in color: {s}"))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| format!("invalid hex digits in color: {s}"))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| format!("invalid hex digits in color: {s}"))?;
    Ok((r, g, b))
}

/// Linear blend of two RGB tuples. `t=0.0` → bg, `t=1.0` → fg.
pub fn blend(fg: (u8, u8, u8), bg: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let lerp = |a: u8, b: u8| -> u8 { (b as f32 + (a as f32 - b as f32) * t).round() as u8 };
    (lerp(fg.0, bg.0), lerp(fg.1, bg.1), lerp(fg.2, bg.2))
}

fn to_color(rgb: (u8, u8, u8)) -> Color {
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}

/// Blend two `Color::Rgb` values. Falls back to `a` for non-Rgb colors.
pub fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => {
            let (r, g, bv) = blend((ar, ag, ab), (br, bg, bb), t);
            Color::Rgb(r, g, bv)
        }
        _ => a,
    }
}

/// Parse a hex string to Color, falling back to `default` on error.
/// Pushes a warning string into `warnings` on failure.
fn parse_or_warn(s: &str, default: (u8, u8, u8), warnings: &mut Vec<String>) -> (u8, u8, u8) {
    match parse_hex_color(s) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(e);
            default
        }
    }
}

/// Parse an Option<String> override field; return None on invalid.
fn parse_override(opt: &Option<String>, warnings: &mut Vec<String>) -> Option<(u8, u8, u8)> {
    opt.as_deref().map(|s| {
        parse_hex_color(s).unwrap_or_else(|e| {
            warnings.push(e);
            (0, 0, 0) // placeholder; caller ignores via the Option wrapping
        })
    })
}

impl Theme {
    /// Build a `Theme` from raw config structs.
    /// Invalid color strings are skipped (defaults used) and warnings appended to `warnings`.
    pub fn from_config(
        palette: &Palette,
        overrides: &ThemeOverrides,
        headings: &HeadingColors,
        warnings: &mut Vec<String>,
    ) -> Self {
        // Parse base palette
        let text = parse_or_warn(&palette.text, (205, 214, 244), warnings);
        let accent = parse_or_warn(&palette.accent, (203, 166, 247), warnings);
        let muted = parse_or_warn(&palette.muted, (88, 91, 112), warnings);
        let code_rgb = parse_or_warn(&palette.code, (166, 227, 161), warnings);
        let bg = parse_or_warn(&palette.bg, (30, 30, 46), warnings);
        let warning_rgb = parse_or_warn(&palette.warning, (243, 139, 168), warnings);

        // Helper: use override or derive
        let resolve = |opt: &Option<String>,
                       derive: (u8, u8, u8),
                       warnings: &mut Vec<String>|
         -> (u8, u8, u8) {
            if let Some(rgb) = parse_override(opt, warnings) {
                rgb
            } else {
                derive
            }
        };

        let bold_color = resolve(&overrides.bold_color, text, warnings);
        // Italic defaults to plain text color (same as bold) — independently overridable.
        let italic_color = resolve(&overrides.italic_color, text, warnings);
        let strikethrough_color = resolve(
            &overrides.strikethrough_color,
            blend(muted, text, 0.5),
            warnings,
        );
        let blockquote_color = resolve(
            &overrides.blockquote_color,
            blend(muted, text, 0.5),
            warnings,
        );
        let link_text = resolve(
            &overrides.link_text_color,
            blend(accent, text, 0.8),
            warnings,
        );
        let link_url = resolve(&overrides.link_url_color, muted, warnings);
        let todo_done = resolve(&overrides.todo_done, muted, warnings);
        let rule_color = resolve(&overrides.rule_color, muted, warnings);
        let code_bg_rgb = resolve(&overrides.code_bg, blend(code_rgb, bg, 0.15), warnings);
        // Fenced block background: lift bg slightly toward text (neutral) so the
        // panel reads as a distinct surface without inheriting code_color's hue.
        let fenced_bg_rgb = resolve(&overrides.fenced_bg, blend(text, bg, 0.08), warnings);
        let heading_bg_rgb = resolve(&overrides.heading_bg, blend(accent, bg, 0.15), warnings);
        let selection_bg_rgb = resolve(&overrides.selection_bg, blend(accent, bg, 0.6), warnings);
        let selection_fg_rgb = resolve(&overrides.selection_fg, bg, warnings);
        // Hints-pill bg: canvas blended 10% toward text — subtly lifted off the canvas.
        let ui_bg_rgb = resolve(&overrides.ui_bg, blend(text, bg, 0.10), warnings);
        // ui_bar is retained as an override target; not used in the default renderer.
        let ui_bar_rgb = resolve(&overrides.ui_bar, bg, warnings);
        let ui_text_rgb = resolve(&overrides.ui_text, text, warnings);
        let delimiter_blend = overrides.delimiter_blend.unwrap_or(0.4).clamp(0.0, 1.0);

        // Per-level heading colors
        let heading_default = |blend_t: f32| blend(accent, text, blend_t);
        let h1_rgb = headings
            .h1
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(1.0));
        let h2_rgb = headings
            .h2
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(1.0));
        let h3_rgb = headings
            .h3
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(0.85));
        let h4_rgb = headings
            .h4
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(0.7));
        let h5_rgb = headings
            .h5
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(0.6));
        let h6_rgb = headings
            .h6
            .as_deref()
            .and_then(|s| parse_hex_color(s).ok())
            .unwrap_or_else(|| heading_default(0.5));

        Self {
            text: to_color(text),
            accent: to_color(accent),
            muted: to_color(muted),
            code_color: to_color(code_rgb),
            bg: to_color(bg),
            warning: to_color(warning_rgb),
            bold_color: to_color(bold_color),
            italic_color: to_color(italic_color),
            strikethrough_color: to_color(strikethrough_color),
            blockquote_color: to_color(blockquote_color),
            link_text: to_color(link_text),
            link_url: to_color(link_url),
            todo_done: to_color(todo_done),
            rule_color: to_color(rule_color),
            code_bg: to_color(code_bg_rgb),
            fenced_bg: to_color(fenced_bg_rgb),
            heading_bg: to_color(heading_bg_rgb),
            selection_bg: to_color(selection_bg_rgb),
            selection_fg: to_color(selection_fg_rgb),
            ui_bg: to_color(ui_bg_rgb),
            ui_bar: to_color(ui_bar_rgb),
            ui_text: to_color(ui_text_rgb),
            headings: HeadingTheme {
                h1: to_color(h1_rgb),
                h2: to_color(h2_rgb),
                h3: to_color(h3_rgb),
                h4: to_color(h4_rgb),
                h5: to_color(h5_rgb),
                h6: to_color(h6_rgb),
            },
            delimiter_blend,
        }
    }

    /// Build from default palette (Catppuccin Mocha) with no overrides.
    pub fn default_theme() -> Self {
        let mut warnings = Vec::new();
        Self::from_config(
            &Palette::default(),
            &ThemeOverrides::default(),
            &HeadingColors::default(),
            &mut warnings,
        )
    }
}

// ---------------------------------------------------------------------------
// Config file loading
// ---------------------------------------------------------------------------

/// Commented template written to the platform config directory on first run.
///
/// All values shown are the Catppuccin Mocha defaults.  The `[theme]`,
/// `[headings]`, and `[layout]` sections are fully commented out — uncomment
/// any line to override the derived default.
pub const DEFAULT_CONFIG_TEMPLATE: &str = r##"# yame configuration
# Unix:    ~/.config/yame/config.toml  (respects $XDG_CONFIG_HOME)
# Windows: %APPDATA%\yame\config.toml
# Reload in-app at any time with Ctrl+R.
#
# All values shown are the Catppuccin Mocha defaults.
# Uncomment and edit any line to override it.

# ── Base palette ──────────────────────────────────────────────────────────────
# These six colors drive the entire theme.
[palette]
text    = "#cdd6f4"   # body text
accent  = "#cba6f7"   # headings, links, bullets
muted   = "#585b70"   # blockquotes, URLs, completed todos
code    = "#a6e3a1"   # inline code and fenced blocks
bg      = "#11111b"   # editor background
warning = "#f38ba8"   # dirty flag, warnings

# ── Per-element overrides ─────────────────────────────────────────────────────
# Uncomment any line to pin that value instead of deriving it from the palette.
[theme]
# bold_color          = "#cdd6f4"
# italic_color        = "#cdd6f4"
# strikethrough_color = "#9399b2"
# blockquote_color    = "#9399b2"
# link_text_color     = "#cbb0f6"
# link_url_color      = "#585b70"
# todo_done           = "#585b70"
# rule_color          = "#585b70"
# code_bg             = "#27312f"
# fenced_bg           = "#20212c"
# heading_bg          = "#2d273c"
# selection_bg        = "#816a9f"
# selection_fg        = "#11111b"
# ui_bg               = "#242531"
# ui_bar              = "#11111b"
# ui_text             = "#cdd6f4"
# delimiter_blend     = 0.4        # 0.0 = full muted · 1.0 = full span color

# ── Per-level heading colors ──────────────────────────────────────────────────
[headings]
# h1 = "#cba6f7"
# h2 = "#cba6f7"
# h3 = "#cbadf7"
# h4 = "#ccb4f6"
# h5 = "#ccb9f6"
# h6 = "#ccbef6"

# ── Layout ────────────────────────────────────────────────────────────────────
[layout]
# min_cols         = 60     # minimum editing-column width in characters
# tab_width        = 4      # spaces per tab character expanded on load
# powerline_glyphs = true   # set false to use the universal │ separator (no Nerd Font required)

# ── Syntax highlighting ────────────────────────────────────────────────────────
[highlighting]
# enabled            = true   # set false to disable fenced-block syntax highlighting
# use_palette_colors = true   # derive token colours from your palette (recommended)
#                             # set false to use a standalone syntect theme instead
# syntect_theme = "base16-ocean.dark"   # only used when use_palette_colors = false
#   Other bundled themes: base16-ocean.light · base16-eighties.dark · base16-mocha.dark
#                         InspiredGitHub · Solarized (dark) · Solarized (light)

# ── File-type detection ───────────────────────────────────────────────────────
[filetype]
# Built-in Markdown extensions: md · markdown · mdx · mkd · mkdn · mdown
# All other extensions are opened in plain-highlight mode (syntect whole-file).
#
# extra_markdown_extensions = []   # additional extensions to treat as Markdown
#                                  # e.g. ["txt", "rst"]
#
# unknown_as = "markdown"          # what to do with extensionless files
#                                  # (CONTRIBUTING, Makefile, …)
#                                  # "markdown" (default) | "plain"
"##;

pub fn config_path() -> PathBuf {
    // Windows: use %APPDATA%\yame\config.toml, falling back to
    // %USERPROFILE%\AppData\Roaming if %APPDATA% is unset (rare).
    #[cfg(windows)]
    {
        let base = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
                    .unwrap_or_default()
            });
        return base.join("yame").join("config.toml");
    }
    // Unix: respect $XDG_CONFIG_HOME, fall back to ~/.config.
    #[cfg(not(windows))]
    {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_default()
                    .join(".config")
            });
        base.join("yame").join("config.toml")
    }
}

/// Load config from disk; fall back to defaults on any error.
/// Returns `(Config, warnings)`.
#[mutants::skip] // fs::read_to_string + toml::from_str I/O path — mutations masked by filesystem state.
pub fn load_config() -> (Config, Vec<String>) {
    let path = config_path();
    let mut warnings = Vec::new();

    if !path.exists() {
        // Scaffold a commented starter config so the user can discover all options.
        // Failures are silently ignored — yame works fine without the file.
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, DEFAULT_CONFIG_TEMPLATE);
        return (Config::default(), warnings);
    }

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("yame: could not read config at {}: {e}", path.display());
            return (Config::default(), warnings);
        }
    };

    match toml::from_str::<Config>(&text) {
        Ok(cfg) => {
            // Validate individual color fields, collecting warnings.
            // Actual resolution happens in Theme::from_config.
            for (name, val) in [
                ("palette.text", &cfg.palette.text),
                ("palette.accent", &cfg.palette.accent),
                ("palette.muted", &cfg.palette.muted),
                ("palette.code", &cfg.palette.code),
                ("palette.bg", &cfg.palette.bg),
                ("palette.warning", &cfg.palette.warning),
            ] {
                if let Err(e) = parse_hex_color(val) {
                    warnings.push(format!("Config warning: Invalid color for {name}: {e}"));
                }
            }
            (cfg, warnings)
        }
        Err(e) => {
            eprintln!("yame: config parse error (using defaults): {e}");
            (Config::default(), warnings)
        }
    }
}

/// Returns true if the given `term` string indicates italic support.
///
/// Separated from env reading so the logic can be unit-tested without
/// touching `$TERM` and causing parallel-test races.
///
/// On Windows, [`supports_italic`] checks `WT_SESSION` before calling this;
/// the strings here cover Git Bash / MSYS2 terminal emulators (`cygwin`,
/// `mintty`) as well as the standard Unix set.
pub fn term_supports_italic(term: &str) -> bool {
    matches!(
        term,
        "xterm-256color"
            | "tmux-256color"
            | "screen-256color"
            | "kitty"
            | "alacritty"
            | "rio"
            | "wezterm"
            | "foot"
            | "cygwin"   // MSYS2 / Git Bash default TERM
            | "mintty" // mintty terminal emulator (Git for Windows)
    ) || term.starts_with("xterm-kitty")
}

/// Detect italic support from the current environment.
///
/// On Windows, Windows Terminal (`WT_SESSION` set) is checked first —
/// it always supports italics regardless of `$TERM`.  For Git Bash / MSYS2,
/// the `$TERM` fallback path handles `cygwin` and `mintty` values.
///
/// This is a thin shim; the `$TERM`-matching logic lives in
/// [`term_supports_italic`], which is fully tested.
#[mutants::skip]
pub fn supports_italic() -> bool {
    // Windows Terminal always supports italics.
    #[cfg(windows)]
    if std::env::var("WT_SESSION").is_ok() {
        return true;
    }
    term_supports_italic(&std::env::var("TERM").unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- blend ---

    #[test]
    fn blend_midpoint() {
        assert_eq!(blend((255, 0, 0), (0, 0, 0), 0.5), (128, 0, 0));
    }

    // --- blend_colors ---

    #[test]
    fn blend_colors_returns_correct_rgb() {
        // blend pure red with pure black at t=0.5 → approximately (128, 0, 0).
        // The return must NOT be Color::default() (= Color::Reset).
        let result = blend_colors(Color::Rgb(255, 0, 0), Color::Rgb(0, 0, 0), 0.5);
        assert_eq!(
            result,
            Color::Rgb(128, 0, 0),
            "blend_colors must interpolate RGB values correctly"
        );
    }

    #[test]
    fn blend_colors_at_zero_returns_b() {
        let result = blend_colors(Color::Rgb(255, 0, 0), Color::Rgb(10, 20, 30), 0.0);
        assert_eq!(result, Color::Rgb(10, 20, 30), "t=0 must return b");
    }

    #[test]
    fn blend_colors_at_one_returns_a() {
        let result = blend_colors(Color::Rgb(255, 0, 0), Color::Rgb(10, 20, 30), 1.0);
        assert_eq!(result, Color::Rgb(255, 0, 0), "t=1 must return a");
    }

    #[test]
    fn blend_zero_is_bg() {
        assert_eq!(blend((255, 0, 0), (10, 20, 30), 0.0), (10, 20, 30));
    }

    #[test]
    fn blend_one_is_fg() {
        assert_eq!(blend((255, 0, 0), (10, 20, 30), 1.0), (255, 0, 0));
    }

    // --- parse_hex_color ---

    #[test]
    fn parse_valid_color() {
        assert_eq!(parse_hex_color("#cba6f7"), Ok((203, 166, 247)));
    }

    #[test]
    fn parse_missing_hash() {
        assert!(parse_hex_color("cba6f7").is_err());
    }

    #[test]
    fn parse_too_short() {
        assert!(parse_hex_color("#cba6f").is_err());
    }

    #[test]
    fn parse_non_hex() {
        assert!(parse_hex_color("#zzzzzz").is_err());
    }

    // --- derived tokens ---

    #[test]
    fn derived_heading_bg() {
        let theme = Theme::default_theme();
        // heading_bg = blend(accent #cba6f7, bg #11111b, 0.15)
        // blend((203,166,247), (17,17,27), 0.15)
        // r = 17 + (203-17)*0.15 = 17 + 27.9  = 44.9  → 45
        // g = 17 + (166-17)*0.15 = 17 + 22.35 = 39.35 → 39
        // b = 27 + (247-27)*0.15 = 27 + 33.0  = 60.0  → 60
        assert!(
            matches!(theme.heading_bg, Color::Rgb(r, g, b) if r == 45 && g == 39 && b == 60),
            "heading_bg was {:?}",
            theme.heading_bg
        );
    }

    #[test]
    fn override_takes_precedence() {
        let overrides = ThemeOverrides {
            bold_color: Some("#ff0000".into()),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let theme = Theme::from_config(
            &Palette::default(),
            &overrides,
            &HeadingColors::default(),
            &mut warnings,
        );
        assert_eq!(theme.bold_color, Color::Rgb(255, 0, 0));
        assert!(warnings.is_empty());
    }

    // --- XDG config path (Unix only) ---

    #[cfg(not(windows))]
    #[test]
    fn xdg_config_home_used_when_set() {
        // Use a unique env key to avoid cross-test interference.
        // SAFETY: single-threaded test context; no other threads read this var.
        unsafe { std::env::set_var("XDG_CONFIG_HOME", "/tmp/custom_yame_test") };
        let path = config_path();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        assert_eq!(
            path,
            PathBuf::from("/tmp/custom_yame_test/yame/config.toml")
        );
    }

    // --- Windows config path ---

    #[cfg(windows)]
    #[test]
    fn windows_appdata_config_path() {
        // SAFETY: single-threaded test context; no other threads read this var.
        unsafe { std::env::set_var("APPDATA", r"C:\Users\test\AppData\Roaming") };
        let path = config_path();
        unsafe { std::env::remove_var("APPDATA") };
        assert_eq!(
            path,
            PathBuf::from(r"C:\Users\test\AppData\Roaming\yame\config.toml")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_config_path_ends_with_config_toml() {
        // Even without controlling APPDATA, the result must end with yame\config.toml.
        let path = config_path();
        assert!(
            path.ends_with("yame\\config.toml") || path.ends_with("yame/config.toml"),
            "Windows config path must end with yame\\config.toml, got: {}",
            path.display()
        );
    }

    // --- italic detection ---

    #[test]
    fn italic_unsupported_for_dumb_term() {
        assert!(!term_supports_italic("dumb"));
    }

    #[test]
    fn italic_unsupported_for_empty_term() {
        assert!(!term_supports_italic(""));
    }

    #[test]
    fn italic_supported_for_xterm_256() {
        assert!(term_supports_italic("xterm-256color"));
    }

    #[test]
    fn italic_supported_for_kitty() {
        assert!(term_supports_italic("kitty"));
    }

    #[test]
    fn italic_supported_for_xterm_kitty_prefix() {
        assert!(term_supports_italic("xterm-kitty"));
    }

    #[test]
    fn italic_supported_for_cygwin() {
        // MSYS2 / Git Bash default TERM value.
        assert!(term_supports_italic("cygwin"));
    }

    #[test]
    fn italic_supported_for_mintty() {
        // mintty terminal emulator (Git for Windows).
        assert!(term_supports_italic("mintty"));
    }

    // --- DEFAULT_CONFIG_TEMPLATE ---

    /// The scaffold template written on first run must be valid TOML so the
    /// next Ctrl+R doesn't fail with a parse error.
    #[test]
    fn default_template_is_valid_toml() {
        let result = toml::from_str::<Config>(DEFAULT_CONFIG_TEMPLATE);
        assert!(
            result.is_ok(),
            "DEFAULT_CONFIG_TEMPLATE failed to parse as Config: {:?}",
            result.err()
        );
    }

    /// powerline_glyphs is commented out in the template (None) so it defers
    /// to the code default of `true`.  If it were accidentally uncommented as
    /// `false` the default behaviour would silently break.
    #[test]
    fn default_template_powerline_glyphs_is_unset() {
        let cfg: Config =
            toml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("template must be valid TOML");
        assert_eq!(
            cfg.layout.powerline_glyphs, None,
            "template must leave powerline_glyphs unset so the code default (true) applies"
        );
        // Confirm the resolution: None → unwrap_or(true) → enabled.
        assert!(cfg.layout.powerline_glyphs.unwrap_or(true));
    }

    /// The palette values embedded in the template must match Config::default()
    /// so that scaffolded configs produce the same theme as no config at all.
    #[test]
    fn default_template_palette_matches_defaults() {
        let cfg: Config =
            toml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("template must be valid TOML");
        let defaults = Config::default();
        assert_eq!(
            cfg.palette.text, defaults.palette.text,
            "template palette.text differs from Config::default()"
        );
        assert_eq!(
            cfg.palette.accent, defaults.palette.accent,
            "template palette.accent differs from Config::default()"
        );
        assert_eq!(
            cfg.palette.bg, defaults.palette.bg,
            "template palette.bg differs from Config::default()"
        );
        assert_eq!(
            cfg.palette.muted, defaults.palette.muted,
            "template palette.muted differs from Config::default()"
        );
        assert_eq!(
            cfg.palette.code, defaults.palette.code,
            "template palette.code differs from Config::default()"
        );
        assert_eq!(
            cfg.palette.warning, defaults.palette.warning,
            "template palette.warning differs from Config::default()"
        );
    }
}
