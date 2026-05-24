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
            bg: "#1e1e2e".into(),
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
    pub blockquote_color: Option<String>,
    pub link_text_color: Option<String>,
    pub link_url_color: Option<String>,
    pub todo_done: Option<String>,
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
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub palette: Palette,
    pub theme: ThemeOverrides,
    pub headings: HeadingColors,
    pub layout: LayoutConfig,
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
    pub blockquote_color: Color,
    pub link_text: Color,
    pub link_url: Color,
    pub todo_done: Color,
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
        let italic_color = resolve(&overrides.italic_color, blend(accent, text, 0.7), warnings);
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
        let code_bg_rgb = resolve(&overrides.code_bg, blend(code_rgb, bg, 0.15), warnings);
        let fenced_bg_rgb = resolve(&overrides.fenced_bg, blend(code_rgb, bg, 0.08), warnings);
        let heading_bg_rgb = resolve(&overrides.heading_bg, blend(accent, bg, 0.15), warnings);
        let selection_bg_rgb = resolve(&overrides.selection_bg, blend(accent, bg, 0.6), warnings);
        let selection_fg_rgb = resolve(&overrides.selection_fg, bg, warnings);
        let ui_bg_rgb = resolve(&overrides.ui_bg, blend(muted, bg, 0.3), warnings);
        let ui_bar_rgb = resolve(&overrides.ui_bar, blend(muted, bg, 0.5), warnings);
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
            blockquote_color: to_color(blockquote_color),
            link_text: to_color(link_text),
            link_url: to_color(link_url),
            todo_done: to_color(todo_done),
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

pub fn config_path() -> PathBuf {
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

/// Load config from disk; fall back to defaults on any error.
/// Returns `(Config, warnings)`.
#[mutants::skip] // fs::read_to_string + toml::from_str I/O path — mutations masked by filesystem state.
pub fn load_config() -> (Config, Vec<String>) {
    let path = config_path();
    let mut warnings = Vec::new();

    if !path.exists() {
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

/// Detect italic support from $TERM.
pub fn supports_italic() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    matches!(
        term.as_str(),
        "xterm-256color"
            | "tmux-256color"
            | "screen-256color"
            | "kitty"
            | "alacritty"
            | "rio"
            | "wezterm"
            | "foot"
    ) || term.starts_with("xterm-kitty")
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
        // heading_bg = blend(accent #cba6f7, bg #1e1e2e, 0.15)
        // blend((203,166,247), (30,30,46), 0.15)
        // r = 30 + (203-30)*0.15 = 30 + 25.95 = 55.95 → 56
        // g = 30 + (166-30)*0.15 = 30 + 20.4  = 50.4  → 50
        // b = 46 + (247-46)*0.15 = 46 + 30.15 = 76.15 → 76
        assert!(
            matches!(theme.heading_bg, Color::Rgb(r, g, b) if r == 56 && g == 50 && b == 76),
            "heading_bg was {:?}",
            theme.heading_bg
        );
    }

    #[test]
    fn override_takes_precedence() {
        let mut overrides = ThemeOverrides::default();
        overrides.bold_color = Some("#ff0000".into());
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

    // --- XDG config path ---

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

    // --- italic detection ---

    #[test]
    fn italic_unsupported_for_dumb_term() {
        let old = std::env::var("TERM").ok();
        // SAFETY: single-threaded test context.
        unsafe { std::env::set_var("TERM", "dumb") };
        let result = supports_italic();
        unsafe {
            match old {
                Some(v) => std::env::set_var("TERM", v),
                None => std::env::remove_var("TERM"),
            }
        }
        assert!(!result);
    }

    #[test]
    fn italic_supported_for_xterm_256() {
        let old = std::env::var("TERM").ok();
        // SAFETY: single-threaded test context.
        unsafe { std::env::set_var("TERM", "xterm-256color") };
        let result = supports_italic();
        unsafe {
            match old {
                Some(v) => std::env::set_var("TERM", v),
                None => std::env::remove_var("TERM"),
            }
        }
        assert!(result);
    }
}
