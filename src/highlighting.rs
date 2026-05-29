//! Syntax highlighting for fenced code blocks via syntect.
//!
//! `HighlightCache` wraps a `SyntaxSet` + resolved `Theme` and memoises results
//! keyed on `(language_tag, hash_of_block_content)`.  The cache is stored in
//! a `RefCell` so that `build_decoration_map` can take `Option<&HighlightCache>`
//! (immutable) while still mutating the cache on first use.
//!
//! # Theme resolution (highest priority first)
//! 1. Palette-derived theme — built from the yame colour palette when
//!    `use_palette_colors = true` (the default).  Keywords use `accent`,
//!    strings use `code_color`, comments use `muted`, etc.
//! 2. Named syntect built-in theme — used when `use_palette_colors = false`
//!    (e.g. `"base16-ocean.dark"`).  Falls back to `"base16-ocean.dark"` on
//!    an unrecognised name.
//!
//! # Fallback behaviour
//! - Highlighting disabled (`enabled = false`) → `None` for every block.
//! - Unknown language tag → `None` (caller falls back to fenced_bg-only).

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

use ratatui::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color as SC, ScopeSelectors, StyleModifier, Theme as SyntectTheme, ThemeItem, ThemeSet,
    ThemeSettings,
};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// One highlighted span within a single line of a code block.
/// All indices are char-counts (not bytes) from the start of that line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HlSpan {
    pub char_start: usize,
    pub char_end: usize,
    pub fg: Color,
}

/// Per-line highlight spans for an entire fenced code block.
/// Outer `Vec` index = line offset within the block content (0-based, fence
/// delimiters excluded).  Each inner `Vec` is the list of coloured spans for
/// that line, in left-to-right order.
pub type BlockHighlights = Vec<Vec<HlSpan>>;

// ---------------------------------------------------------------------------
// Palette-derived theme builder
// ---------------------------------------------------------------------------

/// Convert a `ratatui` `Color::Rgb` value to a `syntect` `Color`.
/// Non-Rgb values fall back to opaque white (won't happen with our palette).
fn to_sc(c: Color) -> SC {
    match c {
        Color::Rgb(r, g, b) => SC { r, g, b, a: 0xFF },
        _ => SC { r: 205, g: 214, b: 244, a: 0xFF },
    }
}

/// Build a syntect `Theme` whose token colours are derived from the yame palette.
///
/// Token → palette mapping:
///
/// | Token scope                        | Colour source                          |
/// |------------------------------------|----------------------------------------|
/// | `keyword`, `storage.*`             | `accent`                               |
/// | `string`, `constant.character`     | `code_color`                           |
/// | `comment`                          | `muted`                                |
/// | `constant.numeric`                 | blend(`accent`, `code_color`, 0.5)     |
/// | `constant.language` (true/false)   | `accent`                               |
/// | `entity.name.type`, `support.type` | blend(`accent`, `text`, 0.75)          |
/// | `entity.name.function`             | blend(`accent`, `text`, 0.85)          |
/// | `invalid`                          | `warning`                              |
/// | everything else                    | `text` (via `ThemeSettings.foreground`)|
pub fn build_palette_theme(yame: &crate::config::Theme) -> SyntectTheme {
    use crate::config::blend_colors;

    let accent = to_sc(yame.accent);
    let code_color = to_sc(yame.code_color);
    let muted = to_sc(yame.muted);
    let text = to_sc(yame.text);
    let bg = to_sc(yame.fenced_bg);
    let warning = to_sc(yame.warning);
    // Numbers: midpoint between accent and code_color — a blue-green that reads
    // as "literal value" without borrowing the warning/error hue.
    let number_color = to_sc(blend_colors(yame.accent, yame.code_color, 0.5));
    // Types: accent blended toward text — softer than keywords, still in-family.
    let type_color = to_sc(blend_colors(yame.accent, yame.text, 0.75));
    // Functions: even softer accent blend.
    let fn_color = to_sc(blend_colors(yame.accent, yame.text, 0.85));
    // Operators: code_color blended toward accent (30%) → soft cyan/teal.
    // Distinct from strings (pure code_color), keywords (pure accent), and the
    // blue-white `text` used for plain identifiers — so `x + y` reads as three
    // different visual tokens.
    let op_color = to_sc(blend_colors(yame.code_color, yame.accent, 0.3));

    // Scope selectors follow the TextMate convention used by syntect grammars.
    // More-specific selectors override less-specific ones (syntect scoring).
    // Listing both "keyword" and "keyword.control" is redundant but explicit.
    let rules: &[(&str, SC)] = &[
        // Keywords and storage modifiers → accent
        ("keyword", accent),
        ("keyword.control", accent),
        ("storage.type", accent),
        ("storage.modifier", accent),
        // Operators → op_color (cyan/teal: distinct from keywords and strings)
        ("keyword.operator", op_color),
        // Strings → code_color
        ("string", code_color),
        ("string.quoted", code_color),
        ("constant.character", code_color),
        // Comments → muted
        ("comment", muted),
        ("comment.line", muted),
        ("comment.block", muted),
        // Numeric literals → number_color (blue-green midpoint)
        ("constant.numeric", number_color),
        // Language constants (true, false, nil, null) → accent
        ("constant.language", accent),
        // Types and built-in types → type_color
        ("entity.name.type", type_color),
        ("entity.name.class", type_color),
        ("support.type", type_color),
        ("support.class", type_color),
        // Function names → fn_color
        ("entity.name.function", fn_color),
        ("support.function", fn_color),
        ("variable.function", fn_color),
        // Invalid / error tokens → warning
        ("invalid", warning),
    ];

    let scopes: Vec<ThemeItem> = rules
        .iter()
        .filter_map(|(scope_str, fg)| {
            ScopeSelectors::from_str(scope_str).ok().map(|scope| ThemeItem {
                scope,
                style: StyleModifier {
                    foreground: Some(*fg),
                    background: None,
                    font_style: None,
                },
            })
        })
        .collect();

    SyntectTheme {
        name: Some("yame-palette".into()),
        author: Some("yame".into()),
        settings: ThemeSettings {
            foreground: Some(text),
            background: Some(bg),
            ..ThemeSettings::default()
        },
        scopes,
    }
}

// ---------------------------------------------------------------------------
// HighlightCache
// ---------------------------------------------------------------------------

pub struct HighlightCache {
    /// Bundled syntect syntaxes (Rust, Python, JS, etc.)
    syntax_set: SyntaxSet,
    /// Bundled syntect themes — only used when `palette_theme` is `None`.
    theme_set: ThemeSet,
    /// Palette-derived theme, pre-built at startup when `use_palette_colors = true`.
    /// When `Some`, this takes priority over `theme_name`.
    palette_theme: Option<SyntectTheme>,
    /// Named built-in theme fallback (used when `palette_theme` is `None`).
    theme_name: String,
    /// Memoisation: `(lang_lower, content_hash)` → per-line coloured spans.
    cache: RefCell<HashMap<(String, u64), BlockHighlights>>,
    /// Whether syntax highlighting is enabled.
    pub enabled: bool,
}

impl HighlightCache {
    const FALLBACK_THEME: &'static str = "base16-ocean.dark";

    /// Create a new cache.
    ///
    /// `palette_theme` — pass `Some(build_palette_theme(&yame_theme))` to use
    /// palette-derived colours (recommended), or `None` to fall through to the
    /// named `theme_name` built-in.
    pub fn new(
        enabled: bool,
        theme_name: String,
        palette_theme: Option<SyntectTheme>,
    ) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            palette_theme,
            theme_name,
            cache: RefCell::new(HashMap::new()),
            enabled,
        }
    }

    /// Highlight `content` as `lang`, returning per-line coloured spans.
    ///
    /// Returns `None` when highlighting is disabled, the language is empty or
    /// unknown, or no theme can be resolved.
    pub fn highlight_block(&self, lang: &str, content: &str) -> Option<BlockHighlights> {
        if !self.enabled || lang.is_empty() {
            return None;
        }

        let lang_lower = lang.to_lowercase();
        let hash = hash_str(content);
        let key = (lang_lower.clone(), hash);

        // Cache hit.
        if let Some(cached) = self.cache.borrow().get(&key) {
            return Some(cached.clone());
        }

        // Resolve syntax.
        let syntax = self
            .syntax_set
            .find_syntax_by_token(&lang_lower)
            .or_else(|| self.syntax_set.find_syntax_by_extension(&lang_lower))?;

        // Resolve theme: palette first, then named built-in, then hard fallback.
        let theme: &SyntectTheme = if let Some(pt) = &self.palette_theme {
            pt
        } else {
            self.theme_set
                .themes
                .get(&self.theme_name)
                .or_else(|| self.theme_set.themes.get(Self::FALLBACK_THEME))?
        };

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut block_hl: BlockHighlights = Vec::new();

        for line in LinesWithEndings::from(content) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .ok()?;

            let mut line_spans: Vec<HlSpan> = Vec::new();
            let mut char_col = 0usize;

            for (style, piece) in &ranges {
                let char_count = piece.chars().filter(|&c| c != '\n').count();
                if char_count > 0 {
                    line_spans.push(HlSpan {
                        char_start: char_col,
                        char_end: char_col + char_count,
                        fg: Color::Rgb(
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        ),
                    });
                    char_col += char_count;
                }
            }

            block_hl.push(line_spans);
        }

        self.cache.borrow_mut().insert(key, block_hl.clone());
        Some(block_hl)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;

    /// Cache using the default built-in theme (no palette).
    fn make_cache() -> HighlightCache {
        HighlightCache::new(true, "base16-ocean.dark".into(), None)
    }

    /// Cache using the palette-derived theme built from the default yame theme.
    fn make_palette_cache() -> HighlightCache {
        let theme = Theme::default_theme();
        let pt = build_palette_theme(&theme);
        HighlightCache::new(true, "base16-ocean.dark".into(), Some(pt))
    }

    // ---- basic cache behaviour ----

    #[test]
    fn disabled_cache_returns_none() {
        let cache = HighlightCache::new(false, "base16-ocean.dark".into(), None);
        assert!(cache.highlight_block("rust", "let x = 1;").is_none());
    }

    #[test]
    fn empty_lang_returns_none() {
        let cache = make_cache();
        assert!(cache.highlight_block("", "fn main() {}").is_none());
    }

    #[test]
    fn unknown_lang_returns_none() {
        let cache = make_cache();
        assert!(cache.highlight_block("notareallangnobodywoulduse", "hello").is_none());
    }

    #[test]
    fn rust_block_returns_spans() {
        let cache = make_cache();
        let content = "let x: u32 = 42;\n";
        let hl = cache.highlight_block("rust", content).expect("rust should be recognised");
        assert_eq!(hl.len(), 1, "one content line → one entry");
        assert!(!hl[0].is_empty(), "rust line must produce spans");
    }

    #[test]
    fn yaml_block_returns_spans() {
        let cache = make_cache();
        let content = "section:\n  key: value\n";
        let hl = cache.highlight_block("yaml", content).expect("yaml should be recognised");
        assert_eq!(hl.len(), 2, "two content lines → two entries");
    }

    #[test]
    fn cache_hit_returns_same_result() {
        let cache = make_cache();
        let content = "fn foo() {}\n";
        let first = cache.highlight_block("rust", content).unwrap();
        let second = cache.highlight_block("rust", content).unwrap();
        assert_eq!(first, second, "cache hit must return identical spans");
    }

    #[test]
    fn multiline_block_line_count_matches() {
        let cache = make_cache();
        let content = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let hl = cache.highlight_block("rust", content).unwrap();
        assert_eq!(hl.len(), 3, "three-line block must produce three line-span lists");
    }

    #[test]
    fn span_char_ranges_are_non_overlapping_and_ordered() {
        let cache = make_cache();
        let hl = cache.highlight_block("rust", "let x: i32 = 42;\n").unwrap();
        for w in hl[0].windows(2) {
            assert!(
                w[0].char_end <= w[1].char_start,
                "spans must be non-overlapping and ordered: {:?}",
                w
            );
        }
    }

    #[test]
    fn unknown_theme_falls_back_gracefully() {
        let cache = HighlightCache::new(true, "no-such-theme-ever".into(), None);
        let _ = cache.highlight_block("rust", "let x = 1;\n");
    }

    #[test]
    fn json_block_produces_spans() {
        let cache = make_cache();
        assert!(cache.highlight_block("json", "{\"key\": 42}\n").is_some());
    }

    #[test]
    fn python_block_produces_spans() {
        let cache = make_cache();
        let hl = cache.highlight_block("python", "def foo():\n    pass\n").unwrap();
        assert_eq!(hl.len(), 2);
    }

    #[test]
    fn case_insensitive_lang_lookup() {
        let cache = make_cache();
        let content = "fn main() {}\n";
        assert_eq!(
            cache.highlight_block("rust", content).is_some(),
            cache.highlight_block("Rust", content).is_some(),
            "language lookup must be case-insensitive"
        );
    }

    #[test]
    fn fg_colors_are_rgb() {
        let cache = make_cache();
        let hl = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        for span in &hl[0] {
            assert!(
                matches!(span.fg, Color::Rgb(_, _, _)),
                "all fg values must be Color::Rgb: {:?}",
                span.fg
            );
        }
    }

    // ---- palette theme builder ----

    #[test]
    fn build_palette_theme_produces_nonempty_scopes() {
        let theme = Theme::default_theme();
        let pt = build_palette_theme(&theme);
        assert!(!pt.scopes.is_empty(), "palette theme must have scope rules");
    }

    #[test]
    fn build_palette_theme_name_is_yame_palette() {
        let theme = Theme::default_theme();
        let pt = build_palette_theme(&theme);
        assert_eq!(pt.name.as_deref(), Some("yame-palette"));
    }

    #[test]
    fn palette_cache_rust_returns_spans() {
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        assert!(!hl[0].is_empty(), "palette cache must produce spans for rust");
    }

    #[test]
    fn palette_cache_fg_colors_are_rgb() {
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        for span in &hl[0] {
            assert!(matches!(span.fg, Color::Rgb(_, _, _)));
        }
    }

    #[test]
    fn palette_cache_keyword_uses_accent_color() {
        // "let" in Rust is a keyword → should render with accent color.
        let yame = Theme::default_theme();
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "let x = 1;\n").unwrap();

        let accent = match yame.accent {
            Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
            _ => panic!("accent must be Rgb"),
        };

        // At least one span must use accent (the `let` keyword).
        assert!(
            hl[0].iter().any(|s| s.fg == accent),
            "a keyword span must use the accent colour; got: {:?}",
            hl[0]
        );
    }

    #[test]
    fn palette_cache_string_uses_code_color() {
        // A quoted string literal should render with code_color.
        let yame = Theme::default_theme();
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "let s = \"hello\";\n").unwrap();

        let code_color = match yame.code_color {
            Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
            _ => panic!("code_color must be Rgb"),
        };

        assert!(
            hl[0].iter().any(|s| s.fg == code_color),
            "a string span must use code_color; got: {:?}",
            hl[0]
        );
    }

    #[test]
    fn palette_cache_comment_uses_muted_color() {
        let yame = Theme::default_theme();
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "// a comment\n").unwrap();

        let muted = match yame.muted {
            Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
            _ => panic!("muted must be Rgb"),
        };

        assert!(
            hl[0].iter().any(|s| s.fg == muted),
            "a comment span must use muted colour; got: {:?}",
            hl[0]
        );
    }

    #[test]
    fn palette_and_builtin_produce_different_keyword_colors() {
        // Palette keywords use accent (#cba6f7); base16-ocean.dark uses its own
        // purple (#b48ead).  They must differ so we know the theme is actually wired.
        let content = "let x = 1;\n";
        let builtin_cache = make_cache();
        let palette_cache = make_palette_cache();

        let builtin_hl = builtin_cache.highlight_block("rust", content).unwrap();
        let palette_hl = palette_cache.highlight_block("rust", content).unwrap();

        // Collect all unique fg colors from each.
        let builtin_colors: std::collections::HashSet<_> =
            builtin_hl[0].iter().map(|s| s.fg).collect();
        let palette_colors: std::collections::HashSet<_> =
            palette_hl[0].iter().map(|s| s.fg).collect();

        assert_ne!(
            builtin_colors, palette_colors,
            "palette theme and built-in theme must produce different colours"
        );
    }

    #[test]
    fn palette_cache_operator_uses_op_color() {
        // `+` in Rust has scope keyword.operator → should render with op_color,
        // which is blend(code_color, accent, 0.3) — distinct from both strings
        // and plain identifiers.
        use crate::config::blend_colors;
        let yame = Theme::default_theme();
        let cache = make_palette_cache();
        // A line with a clear arithmetic operator.
        let hl = cache.highlight_block("rust", "let z = x + y;\n").unwrap();

        let op_color = match blend_colors(yame.code_color, yame.accent, 0.3) {
            Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
            _ => panic!("op_color must be Rgb"),
        };

        assert!(
            hl[0].iter().any(|s| s.fg == op_color),
            "an operator span must use op_color; got: {:?}",
            hl[0]
        );
    }

    #[test]
    fn op_color_is_distinct_from_string_and_keyword_colors() {
        // Validates that the computed op_color doesn't collapse to code_color or
        // accent — i.e. blend(code_color, accent, 0.3) ≠ either endpoint.
        use crate::config::blend_colors;
        let yame = Theme::default_theme();
        let op_color = blend_colors(yame.code_color, yame.accent, 0.3);
        assert_ne!(op_color, yame.code_color, "op_color must differ from code_color (strings)");
        assert_ne!(op_color, yame.accent, "op_color must differ from accent (keywords)");
        assert_ne!(op_color, yame.text, "op_color must differ from text (identifiers)");
    }
}
