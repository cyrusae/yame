use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Raw config structs (deserialized from TOML)
// ---------------------------------------------------------------------------

/// Base color palette.
///
/// All fields are optional.  When a field is absent, its value is taken from
/// the named `preset` (or from the built-in Catppuccin Mocha defaults when no
/// preset is specified).  An explicit field always wins over the preset.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Palette {
    /// Named preset (case-insensitive).  Sets all six base colors; individual
    /// field overrides still win.  Unrecognised names fall back to Catppuccin
    /// Mocha with a config warning.
    ///
    /// Available: `catppuccin-mocha` · `catppuccin-latte` ·
    /// `catppuccin-frappe` · `catppuccin-macchiato` · `dracula` · `nord` ·
    /// `gruvbox-dark` · `solarized-dark` · `solarized-light` · `tokyo-night`
    pub preset: Option<String>,
    pub text:    Option<String>,
    pub accent:  Option<String>,
    pub muted:   Option<String>,
    pub code:    Option<String>,
    pub bg:      Option<String>,
    pub warning: Option<String>,
}

/// Optional per-field overrides for derived theme tokens.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
    /// Background colour for `==highlighted==` text.
    /// Default: 55 % blend of `code` toward `bg` — a visible green tint.
    pub highlight_bg: Option<String>,
    /// Foreground colour for `==highlighted==` text.
    /// Default: body `text` colour.
    pub highlight_fg: Option<String>,
    /// Foreground colour for YAML/TOML frontmatter keys.
    /// Default: `code` colour (green) so keys are visually distinct from accent headings.
    pub frontmatter_key: Option<String>,
    /// Background tint for the entire frontmatter block (delimiters + content).
    /// Default: `code` blended 15 % toward `bg` — a subtle green panel distinct
    /// from the accent-tinted heading background.
    pub frontmatter_bg: Option<String>,
}

/// Per-level heading color overrides (all optional).
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HeadingColors {
    pub h1: Option<String>,
    pub h2: Option<String>,
    pub h3: Option<String>,
    pub h4: Option<String>,
    pub h5: Option<String>,
    pub h6: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Minimum editing-column width in terminal columns.  The column expands to
    /// fill the terminal (up to 50 % of width) but never narrows below this.
    /// Default 60.
    pub min_cols: Option<u16>,
    /// Maximum editing-column width in terminal columns.  The column will not
    /// exceed this value even on wide terminals, keeping line length comfortable
    /// for prose writing.  Must be ≥ `min_cols`; if smaller, `min_cols` wins.
    /// Default: unlimited (the column fills up to 50 % of the terminal width).
    ///
    /// Example: `max_cols = 88` keeps every line within 88 characters.
    pub max_cols: Option<u16>,
    /// Number of spaces to substitute for each `\t` on file load. Default 4.
    pub tab_width: Option<u16>,
    /// Use Powerline/Nerd Font filled-arrow glyphs (U+E0B0) in the status bar
    /// instead of the universal box-drawing separator `│`.
    /// Requires a Nerd Font or Powerline-patched font. Default true.
    /// Set `powerline_glyphs = false` to opt out if your font lacks glyph U+E0B0.
    pub powerline_glyphs: Option<bool>,
    /// Show line numbers in the left gutter. Default false.
    /// Numbers are right-aligned and muted; the cursor line uses accent color.
    /// The gutter width grows automatically as the document grows past 9, 99,
    /// 999 … lines so the content column shifts by one cell at each threshold.
    pub line_numbers: Option<bool>,
}

/// Configuration for syntax highlighting of fenced code blocks.
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
    // line numbers
    pub line_num_active: Color,   // cursor line — muted
    pub line_num_inactive: Color, // all other rows — muted blended toward bg
    // search match highlights
    pub search_match_bg: Color,   // all non-current matches — dim tint
    pub search_current_bg: Color, // the currently selected match — brighter tint
    // ==highlight== spans
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    // frontmatter
    pub frontmatter_key: Color,
    pub frontmatter_bg: Color,
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

// ---------------------------------------------------------------------------
// Named palette presets
// ---------------------------------------------------------------------------

/// The six base colors that constitute a named palette preset.
pub struct PaletteBase {
    pub text:    &'static str,
    pub accent:  &'static str,
    pub muted:   &'static str,
    pub code:    &'static str,
    pub bg:      &'static str,
    pub warning: &'static str,
}

// Catppuccin — https://github.com/catppuccin/catppuccin
// Roles: text=Text · accent=Mauve · muted=Surface2 · code=Green · bg=Crust · warning=Red
const CATPPUCCIN_MOCHA: PaletteBase = PaletteBase {
    text: "#cdd6f4", accent: "#cba6f7", muted: "#585b70",
    code: "#a6e3a1", bg: "#11111b",    warning: "#f38ba8",
};
const CATPPUCCIN_MACCHIATO: PaletteBase = PaletteBase {
    text: "#cad3f5", accent: "#c6a0f6", muted: "#5b6078",
    code: "#a6da95", bg: "#181926",     warning: "#ed8796",
};
const CATPPUCCIN_FRAPPE: PaletteBase = PaletteBase {
    text: "#c6d0f5", accent: "#ca9ee6", muted: "#626880",
    code: "#a6d189", bg: "#232634",     warning: "#e78284",
};
const CATPPUCCIN_LATTE: PaletteBase = PaletteBase {
    text: "#4c4f69", accent: "#8839ef", muted: "#acb0be",
    code: "#40a02b", bg: "#dce0e8",     warning: "#d20f39",
};

// Dracula — https://draculatheme.com/contribute
// Roles: text=Foreground · accent=Purple · muted=Comment · code=Green · bg=Background · warning=Red
const DRACULA: PaletteBase = PaletteBase {
    text: "#f8f8f2", accent: "#bd93f9", muted: "#6272a4",
    code: "#50fa7b", bg: "#282a36",     warning: "#ff5555",
};

// Nord — https://www.nordtheme.com/docs/colors-and-palettes
// Roles: text=Snow3 · accent=Frost2 · muted=PolarNight4 · code=AuroraGreen · bg=PolarNight1 · warning=AuroraRed
const NORD: PaletteBase = PaletteBase {
    text: "#eceff4", accent: "#88c0d0", muted: "#4c566a",
    code: "#a3be8c", bg: "#2e3440",     warning: "#bf616a",
};

// Gruvbox Dark — https://github.com/morhetz/gruvbox
// Roles: text=fg1 · accent=purple · muted=gray · code=bright_green · bg=bg0_h · warning=bright_red
const GRUVBOX_DARK: PaletteBase = PaletteBase {
    text: "#ebdbb2", accent: "#d3869b", muted: "#928374",
    code: "#b8bb26", bg: "#1d2021",     warning: "#fb4934",
};

// Solarized — https://ethanschoonover.com/solarized/
// Roles: text=base0 · accent=blue · muted=base01 · code=green · bg=base03 · warning=red
const SOLARIZED_DARK: PaletteBase = PaletteBase {
    text: "#839496", accent: "#268bd2", muted: "#586e75",
    code: "#859900", bg: "#002b36",     warning: "#dc322f",
};
const SOLARIZED_LIGHT: PaletteBase = PaletteBase {
    text: "#657b83", accent: "#268bd2", muted: "#93a1a1",
    code: "#859900", bg: "#fdf6e3",     warning: "#dc322f",
};

// Tokyo Night — https://github.com/folke/tokyonight.nvim
// Roles: text=fg · accent=purple · muted=comment · code=green · bg=bg · warning=red
const TOKYO_NIGHT: PaletteBase = PaletteBase {
    text: "#a9b1d6", accent: "#bb9af7", muted: "#565f89",
    code: "#9ece6a", bg: "#1a1b26",     warning: "#f7768e",
};

// Rosé Pine — https://rosepinetheme.com/palette/
// Roles: text=Text · accent=Iris · muted=Muted · code=Foam · bg=Base · warning=Love
const ROSE_PINE: PaletteBase = PaletteBase {
    text: "#e0def4", accent: "#c4a7e7", muted: "#6e6a86",
    code: "#9ccfd8", bg: "#191724",     warning: "#eb6f92",
};
const ROSE_PINE_MOON: PaletteBase = PaletteBase {
    text: "#e0def4", accent: "#c4a7e7", muted: "#6e6a86",
    code: "#9ccfd8", bg: "#232136",     warning: "#eb6f92",
};
const ROSE_PINE_DAWN: PaletteBase = PaletteBase {
    text: "#575279", accent: "#907aa9", muted: "#9893a5",
    code: "#56949f", bg: "#faf4ed",     warning: "#b4637a",
};

// GitHub — https://primer.style/foundations/color
// Roles: text=fg.default · accent=accent.fg · muted=fg.muted · code=success.fg · bg=canvas.default · warning=danger.fg
const GITHUB_LIGHT: PaletteBase = PaletteBase {
    text: "#24292f", accent: "#0969da", muted: "#57606a",
    code: "#116329", bg: "#ffffff",     warning: "#cf222e",
};

// ---------------------------------------------------------------------------
// Preset expansions — rainbow headings + per-theme accent tints
// ---------------------------------------------------------------------------

/// Optional per-preset extra derived colors.
///
/// An *expanded* preset (e.g. `preset = "catppuccin-mocha-expanded"`) uses
/// these values as the default for heading colors and italic color.
/// User `[headings]` and `[theme]` overrides always win over the expansion.
pub struct PaletteExpansion {
    // Per-level heading colors (h1 → h6, warm-to-cool or theme-native order).
    pub h1: &'static str,
    pub h2: &'static str,
    pub h3: &'static str,
    pub h4: &'static str,
    pub h5: &'static str,
    pub h6: &'static str,
    /// Italic text color (e.g. theme's Pink or Sky token).
    pub italic_color: &'static str,
}

// Catppuccin expansions — rainbow order: Rosewater → Flamingo → Pink → Mauve → Red → Maroon
const CATPPUCCIN_MOCHA_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#f5e0dc", h2: "#f2cdcd", h3: "#f5c2e7",
    h4: "#cba6f7", h5: "#f38ba8", h6: "#eba0ac",
    italic_color: "#f5e0dc", // Rosewater — warm warmth for emphasis
};
const CATPPUCCIN_MACCHIATO_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#f4dbd6", h2: "#f0c6c6", h3: "#f5bde6",
    h4: "#c6a0f6", h5: "#ed8796", h6: "#ee99a0",
    italic_color: "#f4dbd6",
};
const CATPPUCCIN_FRAPPE_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#f2d5cf", h2: "#eebebe", h3: "#f4b8e4",
    h4: "#ca9ee6", h5: "#e78284", h6: "#ea999c",
    italic_color: "#f2d5cf",
};
const CATPPUCCIN_LATTE_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#dc8a78", h2: "#dd7878", h3: "#ea76cb",
    h4: "#8839ef", h5: "#d20f39", h6: "#e64553",
    italic_color: "#dc8a78",
};

// Dracula expansion — rainbow from Dracula's full palette
const DRACULA_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#ff79c6", h2: "#ffb86c", h3: "#f1fa8c",
    h4: "#50fa7b", h5: "#8be9fd", h6: "#bd93f9",
    italic_color: "#ff79c6", // Pink — traditional Dracula italic
};

// Nord expansion — Aurora palette (Red → Orange → Yellow → Green → Frost → Purple)
const NORD_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#bf616a", h2: "#d08770", h3: "#ebcb8b",
    h4: "#a3be8c", h5: "#88c0d0", h6: "#b48ead",
    italic_color: "#81a1c1", // Frost1 — slightly softer than accent Frost2
};

// Gruvbox Dark expansion — bright palette (Red → Orange → Yellow → Green → Aqua → Blue)
const GRUVBOX_DARK_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#fb4934", h2: "#fe8019", h3: "#fabd2f",
    h4: "#b8bb26", h5: "#8ec07c", h6: "#83a598",
    italic_color: "#fe8019", // bright orange
};

// Solarized expansions — full Solarized accent palette
const SOLARIZED_DARK_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#dc322f", h2: "#cb4b16", h3: "#b58900",
    h4: "#859900", h5: "#2aa198", h6: "#268bd2",
    italic_color: "#2aa198", // cyan — Solarized's traditional italic/string color
};
const SOLARIZED_LIGHT_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#dc322f", h2: "#cb4b16", h3: "#b58900",
    h4: "#859900", h5: "#2aa198", h6: "#268bd2",
    italic_color: "#2aa198",
};

// Tokyo Night expansion — full accent palette
const TOKYO_NIGHT_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#f7768e", h2: "#ff9e64", h3: "#e0af68",
    h4: "#9ece6a", h5: "#73daca", h6: "#7dcfff",
    italic_color: "#7dcfff", // sky blue
};

// Rosé Pine expansions — Love · Gold · Rose · Iris · Pine · Foam
const ROSE_PINE_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#eb6f92", h2: "#f6c177", h3: "#ebbcba",
    h4: "#c4a7e7", h5: "#31748f", h6: "#9ccfd8",
    italic_color: "#ebbcba", // Rose — warm rosy emphasis
};
const ROSE_PINE_MOON_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#eb6f92", h2: "#f6c177", h3: "#ebbcba",
    h4: "#c4a7e7", h5: "#3e8fb0", h6: "#9ccfd8",
    italic_color: "#ebbcba",
};
const ROSE_PINE_DAWN_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#b4637a", h2: "#ea9d34", h3: "#d7827a",
    h4: "#907aa9", h5: "#286983", h6: "#56949f",
    italic_color: "#d7827a", // Rose (Dawn variant)
};

// GitHub Light expansion — GitHub's semantic color palette
const GITHUB_LIGHT_EXP: PaletteExpansion = PaletteExpansion {
    h1: "#cf222e", h2: "#953800", h3: "#9a6700",
    h4: "#116329", h5: "#0550ae", h6: "#8250df",
    italic_color: "#953800", // orange — GitHub attribute/annotation color
};

// ---------------------------------------------------------------------------
// Preset lookup helpers
// ---------------------------------------------------------------------------

/// Strip a trailing `-expanded` suffix (case-insensitive), returning the base
/// name slice and whether the suffix was present.
fn strip_expanded_suffix(name: &str) -> (&str, bool) {
    // Work in lowercase for comparison but return the original slice length.
    let lower = name.to_ascii_lowercase();
    if lower.ends_with("-expanded") {
        (&name[..name.len() - "-expanded".len()], true)
    } else {
        (name, false)
    }
}

/// Look up a named palette preset (case-insensitive).
///
/// The `-expanded` suffix is stripped before lookup so that both
/// `"catppuccin-mocha"` and `"catppuccin-mocha-expanded"` resolve to the
/// same six base colors.
///
/// Returns `None` for unrecognised names so callers can emit a config warning.
pub fn palette_preset(name: &str) -> Option<&'static PaletteBase> {
    let (base, _) = strip_expanded_suffix(name);
    Some(match base.to_lowercase().as_str() {
        "catppuccin-mocha"     | "mocha"          => &CATPPUCCIN_MOCHA,
        "catppuccin-macchiato" | "macchiato"       => &CATPPUCCIN_MACCHIATO,
        "catppuccin-frappe"    | "frappe"
        | "catppuccin-frappé"  | "frappé"          => &CATPPUCCIN_FRAPPE,
        "catppuccin-latte"     | "latte"           => &CATPPUCCIN_LATTE,
        "dracula"                                  => &DRACULA,
        "nord"                                     => &NORD,
        "gruvbox-dark"         | "gruvbox"         => &GRUVBOX_DARK,
        "solarized-dark"       | "solarized"       => &SOLARIZED_DARK,
        "solarized-light"                          => &SOLARIZED_LIGHT,
        "tokyo-night"          | "tokyonight"      => &TOKYO_NIGHT,
        "rose-pine"            | "rosepine"        => &ROSE_PINE,
        "rose-pine-moon"       | "rosepine-moon"   => &ROSE_PINE_MOON,
        "rose-pine-dawn"       | "rosepine-dawn"   => &ROSE_PINE_DAWN,
        "github-light"         | "github"          => &GITHUB_LIGHT,
        _                                          => return None,
    })
}

/// Look up the *expansion* for an expanded preset variant (case-insensitive).
///
/// An expansion provides rainbow heading colors and a theme-native italic tint.
/// Returns `None` when the name has no `-expanded` suffix **or** when the base
/// preset is not recognised.
pub fn palette_expansion(name: &str) -> Option<&'static PaletteExpansion> {
    let (base, is_expanded) = strip_expanded_suffix(name);
    if !is_expanded {
        return None;
    }
    Some(match base.to_lowercase().as_str() {
        "catppuccin-mocha"     | "mocha"          => &CATPPUCCIN_MOCHA_EXP,
        "catppuccin-macchiato" | "macchiato"       => &CATPPUCCIN_MACCHIATO_EXP,
        "catppuccin-frappe"    | "frappe"
        | "catppuccin-frappé"  | "frappé"          => &CATPPUCCIN_FRAPPE_EXP,
        "catppuccin-latte"     | "latte"           => &CATPPUCCIN_LATTE_EXP,
        "dracula"                                  => &DRACULA_EXP,
        "nord"                                     => &NORD_EXP,
        "gruvbox-dark"         | "gruvbox"         => &GRUVBOX_DARK_EXP,
        "solarized-dark"       | "solarized"       => &SOLARIZED_DARK_EXP,
        "solarized-light"                          => &SOLARIZED_LIGHT_EXP,
        "tokyo-night"          | "tokyonight"      => &TOKYO_NIGHT_EXP,
        "rose-pine"            | "rosepine"        => &ROSE_PINE_EXP,
        "rose-pine-moon"       | "rosepine-moon"   => &ROSE_PINE_MOON_EXP,
        "rose-pine-dawn"       | "rosepine-dawn"   => &ROSE_PINE_DAWN_EXP,
        "github-light"         | "github"          => &GITHUB_LIGHT_EXP,
        _                                          => return None,
    })
}

// ---------------------------------------------------------------------------

impl Theme {
    /// Build a `Theme` from raw config structs.
    /// Invalid color strings are skipped (defaults used) and warnings appended to `warnings`.
    pub fn from_config(
        palette: &Palette,
        overrides: &ThemeOverrides,
        headings: &HeadingColors,
        warnings: &mut Vec<String>,
    ) -> Self {
        // Resolve named preset (falls back to Catppuccin Mocha when absent or unknown).
        let base = palette
            .preset
            .as_deref()
            .and_then(|n| {
                let p = palette_preset(n);
                if p.is_none() {
                    warnings.push(format!(
                        "Config warning: unknown palette preset {n:?}; using Catppuccin Mocha"
                    ));
                }
                p
            })
            .unwrap_or(&CATPPUCCIN_MOCHA);

        // Resolve optional expansion (set by an `-expanded` preset suffix).
        // Provides rainbow heading colors and a per-theme italic tint.
        let expansion = palette.preset.as_deref().and_then(palette_expansion);

        // Pre-parse the preset's six base colors (hardcoded strings — always valid).
        let base_text    = parse_hex_color(base.text).unwrap_or((205, 214, 244));
        let base_accent  = parse_hex_color(base.accent).unwrap_or((203, 166, 247));
        let base_muted   = parse_hex_color(base.muted).unwrap_or((88, 91, 112));
        let base_code    = parse_hex_color(base.code).unwrap_or((166, 227, 161));
        let base_bg      = parse_hex_color(base.bg).unwrap_or((17, 17, 27));
        let base_warning = parse_hex_color(base.warning).unwrap_or((243, 139, 168));

        // Per-field palette overrides win over the preset; absent fields use preset.
        let text       = parse_or_warn(palette.text.as_deref().unwrap_or(base.text),    base_text,    warnings);
        let accent     = parse_or_warn(palette.accent.as_deref().unwrap_or(base.accent), base_accent, warnings);
        let muted      = parse_or_warn(palette.muted.as_deref().unwrap_or(base.muted),  base_muted,  warnings);
        let code_rgb   = parse_or_warn(palette.code.as_deref().unwrap_or(base.code),    base_code,   warnings);
        let bg         = parse_or_warn(palette.bg.as_deref().unwrap_or(base.bg),        base_bg,     warnings);
        let warning_rgb = parse_or_warn(palette.warning.as_deref().unwrap_or(base.warning), base_warning, warnings);

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
        // Italic: expansion provides a per-theme tint (e.g. Rosewater for Catppuccin,
        // Pink for Dracula); plain text color when no expansion is active.
        // [theme] italic_color always wins over the expansion default.
        let italic_default = expansion
            .and_then(|e| parse_hex_color(e.italic_color).ok())
            .unwrap_or(text);
        let italic_color = resolve(&overrides.italic_color, italic_default, warnings);
        let strikethrough_color = resolve(
            &overrides.strikethrough_color,
            blend(muted, text, 0.5),
            warnings,
        );
        let blockquote_color = resolve(
            &overrides.blockquote_color,
            blend(accent, muted, 0.5),
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
        // Line-number colors: deliberately subdued so they don't compete with content.
        // Active (cursor) row uses plain muted; inactive rows blend muted 60% toward bg.
        let line_num_active_rgb = muted;
        let line_num_inactive_rgb = blend(muted, bg, 0.6);
        // Search-match highlight colors derived from code_color (green) so they are
        // visually distinct from selections (which use accent/lavender).
        let search_match_bg_rgb = blend(code_rgb, bg, 0.35);
        let search_current_bg_rgb = blend(code_rgb, bg, 0.70);
        // Highlight (==text==): green tint background, normal text foreground.
        // Users can override to e.g. a warm yellow for a traditional marker feel.
        // Default: accent blended 35% toward bg — visibly tinted without reaching
        // selection-level saturation.  Uses the accent hue so highlights stay
        // tonally consistent with the rest of the theme.
        // Override with [theme] highlight_bg.
        let highlight_bg_rgb = resolve(&overrides.highlight_bg, blend(accent, bg, 0.35), warnings);
        let highlight_fg_rgb = resolve(&overrides.highlight_fg, text, warnings);
        // Frontmatter key color: code green by default — visually distinct from
        // accent headings and makes keys easy to scan against plain-text values.
        let frontmatter_key_rgb = resolve(&overrides.frontmatter_key, code_rgb, warnings);
        // Frontmatter background: code blended 15% toward bg, same recipe as
        // heading_bg but using the code hue so the block reads as a distinct
        // "data" surface rather than a heading.
        let frontmatter_bg_rgb = resolve(
            &overrides.frontmatter_bg,
            blend(code_rgb, bg, 0.15),
            warnings,
        );

        // Per-level heading colors.
        // Resolution order (last wins): blend default → expansion → [headings] override.
        let heading_default = |blend_t: f32| blend(accent, text, blend_t);
        // Helper: parse one expansion heading field, used in the or_else chain below.
        let exp_h = |s: &'static str| parse_hex_color(s).ok();
        let h1_rgb = headings.h1.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h1)))
            .unwrap_or_else(|| heading_default(1.0));
        let h2_rgb = headings.h2.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h2)))
            .unwrap_or_else(|| heading_default(1.0));
        let h3_rgb = headings.h3.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h3)))
            .unwrap_or_else(|| heading_default(0.85));
        let h4_rgb = headings.h4.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h4)))
            .unwrap_or_else(|| heading_default(0.7));
        let h5_rgb = headings.h5.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h5)))
            .unwrap_or_else(|| heading_default(0.6));
        let h6_rgb = headings.h6.as_deref().and_then(|s| parse_hex_color(s).ok())
            .or_else(|| expansion.and_then(|e| exp_h(e.h6)))
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
            line_num_active: to_color(line_num_active_rgb),
            line_num_inactive: to_color(line_num_inactive_rgb),
            search_match_bg: to_color(search_match_bg_rgb),
            search_current_bg: to_color(search_current_bg_rgb),
            highlight_bg: to_color(highlight_bg_rgb),
            highlight_fg: to_color(highlight_fg_rgb),
            frontmatter_key: to_color(frontmatter_key_rgb),
            frontmatter_bg: to_color(frontmatter_bg_rgb),
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
/// All palette fields are commented out; the built-in Catppuccin Mocha defaults
/// apply.  The `[theme]`, `[headings]`, and `[layout]` sections are also fully
/// commented out — uncomment any line to override the derived default.
pub const DEFAULT_CONFIG_TEMPLATE: &str = r##"# yame configuration
# Unix:    ~/.config/yame/config.toml  (respects $XDG_CONFIG_HOME)
# Windows: %APPDATA%\yame\config.toml
# Reload in-app at any time with Ctrl+R.
#
# Defaults are Catppuccin Mocha.  Uncomment any line to override it.

# ── Base palette ──────────────────────────────────────────────────────────────
# Switch to a named preset, or override individual colors, or both
# (individual fields always win over the preset).
#
# Available presets:
#   catppuccin-mocha (default) · catppuccin-latte · catppuccin-frappe · catppuccin-macchiato
#   dracula · nord · gruvbox-dark · solarized-dark · solarized-light · tokyo-night
#   rose-pine · rose-pine-moon · rose-pine-dawn · github-light
#
# Append -expanded for rainbow headings + per-theme italic tint:
#   e.g. "catppuccin-mocha-expanded", "dracula-expanded", "rose-pine-expanded"
[palette]
# preset  = "catppuccin-mocha"   # switch to any preset above; add -expanded for rainbow headings
# text    = "#cdd6f4"   # body text
# accent  = "#cba6f7"   # headings, links, bullets
# muted   = "#585b70"   # blockquotes, URLs, completed todos
# code    = "#a6e3a1"   # inline code and fenced blocks
# bg      = "#11111b"   # editor background
# warning = "#f38ba8"   # dirty flag, warnings

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
# highlight_bg        = "#524568"  # ==text== background (accent at 35% — try "#816a9f" for bolder)
# highlight_fg        = "#cdd6f4"  # ==text== foreground
# frontmatter_key     = "#a6e3a1"  # YAML/TOML frontmatter key color (default: code green)
# frontmatter_bg      = "#1e2620"  # YAML/TOML frontmatter block background (default: code at 15%)

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
# max_cols         = 88     # maximum editing-column width (prose wrap margin); default: unlimited
# tab_width        = 4      # spaces per tab character expanded on load
# powerline_glyphs = true   # set false to use the universal │ separator (no Nerd Font required)
# line_numbers     = false  # set true to show line numbers in the left gutter

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
                ("palette.text",    &cfg.palette.text),
                ("palette.accent",  &cfg.palette.accent),
                ("palette.muted",   &cfg.palette.muted),
                ("palette.code",    &cfg.palette.code),
                ("palette.bg",      &cfg.palette.bg),
                ("palette.warning", &cfg.palette.warning),
            ] {
                if let Some(v) = val
                    && let Err(e) = parse_hex_color(v)
                {
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

/// Serialize `config` to TOML and write it to the platform config path.
///
/// Creates parent directories if they do not exist.  Called by the settings
/// modal when the user commits a field change.
#[mutants::skip] // fs::write I/O — mutations masked by filesystem state.
pub fn write_config(config: &Config) -> std::io::Result<()> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&path, text)
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

    /// The scaffold template must produce the same computed theme as no config at
    /// all (Catppuccin Mocha).  All palette fields are commented out in the
    /// template, so the preset/default resolution path is exercised end-to-end.
    #[test]
    fn default_template_produces_mocha_theme() {
        let cfg: Config =
            toml::from_str(DEFAULT_CONFIG_TEMPLATE).expect("template must be valid TOML");
        // Template leaves all palette fields unset — preset = None, fields = None.
        assert!(cfg.palette.preset.is_none(), "template must not set a preset");
        assert!(cfg.palette.text.is_none(),   "template must not set palette.text");

        let mut warnings = Vec::new();
        let theme = Theme::from_config(&cfg.palette, &cfg.theme, &cfg.headings, &mut warnings);
        assert!(warnings.is_empty(), "template must not produce config warnings: {warnings:?}");

        // Compare against the well-known Mocha values.
        assert_eq!(theme.bg,     Color::Rgb(17,  17,  27),  "bg must be Mocha crust #11111b");
        assert_eq!(theme.accent, Color::Rgb(203, 166, 247), "accent must be Mocha mauve #cba6f7");
        assert_eq!(theme.text,   Color::Rgb(205, 214, 244), "text must be Mocha text #cdd6f4");
        // heading_bg is derived; if base colors are correct the derivation follows.
        assert_eq!(theme.heading_bg, Theme::default_theme().heading_bg,
            "heading_bg must match default_theme()");
    }

    // --- palette_preset ---

    #[test]
    fn palette_preset_mocha_aliases_resolve() {
        assert!(palette_preset("catppuccin-mocha").is_some());
        assert!(palette_preset("mocha").is_some());
        assert!(palette_preset("Mocha").is_some()); // case-insensitive
    }

    #[test]
    fn palette_preset_all_known_names_resolve() {
        let known = [
            "catppuccin-mocha", "catppuccin-latte", "catppuccin-frappe",
            "catppuccin-macchiato", "dracula", "nord", "gruvbox-dark",
            "solarized-dark", "solarized-light", "tokyo-night",
            "rose-pine", "rose-pine-moon", "rose-pine-dawn", "github-light",
            // short aliases
            "mocha", "latte", "frappe", "macchiato", "gruvbox", "solarized",
            "tokyonight", "rosepine", "github",
            // -expanded variants resolve the same base colors
            "catppuccin-mocha-expanded", "dracula-expanded", "rose-pine-expanded",
        ];
        for name in known {
            assert!(palette_preset(name).is_some(), "preset {name:?} must resolve");
        }
    }

    #[test]
    fn palette_preset_unknown_returns_none() {
        assert!(palette_preset("").is_none());
        assert!(palette_preset("everforest").is_none());
        assert!(palette_preset("one-dark").is_none());
    }

    #[test]
    fn palette_preset_field_override_wins_over_preset() {
        // Setting preset = "dracula" but overriding accent with red:
        // the computed accent must be red, not Dracula purple.
        let palette = Palette {
            preset: Some("dracula".into()),
            accent: Some("#ff0000".into()),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let theme = Theme::from_config(&palette, &ThemeOverrides::default(), &HeadingColors::default(), &mut warnings);
        assert_eq!(theme.accent, Color::Rgb(255, 0, 0),
            "explicit accent must override the preset value");
        assert!(warnings.is_empty());
    }

    #[test]
    fn palette_preset_unknown_emits_warning_and_uses_mocha() {
        let palette = Palette {
            preset: Some("no-such-theme".into()),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let theme = Theme::from_config(&palette, &ThemeOverrides::default(), &HeadingColors::default(), &mut warnings);
        assert!(!warnings.is_empty(), "unknown preset must push a warning");
        // Falls back to Mocha bg
        assert_eq!(theme.bg, Color::Rgb(17, 17, 27), "unknown preset must fall back to Mocha bg");
    }

    #[test]
    fn palette_preset_latte_bg_is_light() {
        let p = palette_preset("catppuccin-latte").expect("latte must resolve");
        // Latte bg (Crust) is a very light gray — R,G,B all well above 200.
        let (r, g, b) = parse_hex_color(p.bg).expect("preset bg must be valid hex");
        assert!(r > 200 && g > 200 && b > 200, "Latte bg must be light, got ({r},{g},{b})");
    }

    #[test]
    fn palette_preset_rose_pine_dawn_bg_is_light() {
        let p = palette_preset("rose-pine-dawn").expect("rose-pine-dawn must resolve");
        let (r, g, b) = parse_hex_color(p.bg).expect("preset bg must be valid hex");
        assert!(r > 200 && g > 200 && b > 200, "Rose Pine Dawn bg must be light, got ({r},{g},{b})");
    }

    #[test]
    fn palette_preset_github_light_bg_is_white() {
        let p = palette_preset("github-light").expect("github-light must resolve");
        assert_eq!(p.bg, "#ffffff");
    }

    // --- palette_expansion ---

    #[test]
    fn palette_expansion_requires_expanded_suffix() {
        // Without -expanded: no expansion returned.
        assert!(palette_expansion("catppuccin-mocha").is_none());
        assert!(palette_expansion("dracula").is_none());
        assert!(palette_expansion("rose-pine").is_none());
    }

    #[test]
    fn palette_expansion_all_expanded_variants_resolve() {
        let expanded = [
            "catppuccin-mocha-expanded", "catppuccin-latte-expanded",
            "catppuccin-frappe-expanded", "catppuccin-macchiato-expanded",
            "dracula-expanded", "nord-expanded", "gruvbox-dark-expanded",
            "solarized-dark-expanded", "solarized-light-expanded",
            "tokyo-night-expanded", "rose-pine-expanded",
            "rose-pine-moon-expanded", "rose-pine-dawn-expanded",
            "github-light-expanded",
            // short alias + expanded
            "mocha-expanded", "latte-expanded", "dracula-expanded",
            "tokyonight-expanded", "rosepine-expanded", "github-expanded",
        ];
        for name in expanded {
            assert!(palette_expansion(name).is_some(), "expansion {name:?} must resolve");
        }
    }

    #[test]
    fn palette_expansion_unknown_base_returns_none() {
        assert!(palette_expansion("everforest-expanded").is_none());
        assert!(palette_expansion("-expanded").is_none());
    }

    #[test]
    fn palette_expansion_mocha_headings_are_rainbow() {
        // Mocha expansion h1=Rosewater, h6=Maroon — they must be different colors.
        let exp = palette_expansion("catppuccin-mocha-expanded").expect("must resolve");
        assert_ne!(exp.h1, exp.h6, "rainbow headings h1 and h6 must differ");
        // h1 is Rosewater — pinkish, high R value
        let (r, _, _) = parse_hex_color(exp.h1).expect("h1 must be valid hex");
        assert!(r > 200, "Mocha h1 (Rosewater) must have high R, got {r}");
    }

    #[test]
    fn palette_expansion_mocha_italic_is_not_text() {
        // Expansion italic should differ from the plain Mocha text color.
        let exp = palette_expansion("catppuccin-mocha-expanded").expect("must resolve");
        assert_ne!(exp.italic_color, CATPPUCCIN_MOCHA.text,
            "expanded italic_color must differ from base text");
    }

    #[test]
    fn expanded_preset_headings_differ_from_non_expanded() {
        // Non-expanded: all headings default to accent/text blend (same color for h1+h2).
        let plain = Palette { preset: Some("catppuccin-mocha".into()), ..Default::default() };
        let expanded = Palette { preset: Some("catppuccin-mocha-expanded".into()), ..Default::default() };
        let mut w = Vec::new();
        let t_plain    = Theme::from_config(&plain,    &ThemeOverrides::default(), &HeadingColors::default(), &mut w);
        let t_expanded = Theme::from_config(&expanded, &ThemeOverrides::default(), &HeadingColors::default(), &mut w);
        assert!(w.is_empty());
        assert_ne!(t_plain.headings.h1, t_expanded.headings.h1,
            "expanded h1 must differ from plain h1");
        assert_ne!(t_plain.headings.h6, t_expanded.headings.h6,
            "expanded h6 must differ from plain h6");
    }

    #[test]
    fn expanded_preset_headings_user_override_still_wins() {
        // [headings] h1 override must beat the expansion's rainbow value.
        let palette = Palette { preset: Some("dracula-expanded".into()), ..Default::default() };
        let headings = HeadingColors { h1: Some("#aabbcc".into()), ..Default::default() };
        let mut w = Vec::new();
        let theme = Theme::from_config(&palette, &ThemeOverrides::default(), &headings, &mut w);
        assert_eq!(theme.headings.h1, Color::Rgb(0xaa, 0xbb, 0xcc),
            "user [headings].h1 must override expansion h1");
    }

    #[test]
    fn expanded_preset_italic_user_override_still_wins() {
        let palette = Palette { preset: Some("catppuccin-mocha-expanded".into()), ..Default::default() };
        let overrides = ThemeOverrides { italic_color: Some("#123456".into()), ..Default::default() };
        let mut w = Vec::new();
        let theme = Theme::from_config(&palette, &overrides, &HeadingColors::default(), &mut w);
        assert_eq!(theme.italic_color, Color::Rgb(0x12, 0x34, 0x56),
            "[theme] italic_color must override expansion italic");
    }
}
