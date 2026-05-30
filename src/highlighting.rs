//! Syntax highlighting for fenced code blocks via syntect.
//!
//! `HighlightCache` wraps a lazily-initialised `SyntaxSet` + resolved `Theme`
//! and memoises results keyed on `(language_tag, hash_of_block_content)`.
//!
//! ## Lazy initialisation
//! `SyntaxSet` and `ThemeSet` are expensive to deserialise from their bundled
//! binary assets.  Both are held in `OnceLock` fields and initialised on the
//! first call to `highlight_block` — so files with no fenced code blocks pay
//! zero startup cost.  On the default production path (`use_palette_colors =
//! true`) the `ThemeSet` is never initialised at all.
//!
//! The highlight memo-cache is stored in a `RefCell` so that
//! `build_decoration_map` can take `Option<&HighlightCache>` (immutable) while
//! still mutating the cache on first use.
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
use std::sync::OnceLock;

use ratatui::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    Color as SC, ScopeSelectors, StyleModifier, Theme as SyntectTheme, ThemeItem, ThemeSet,
    ThemeSettings,
};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use two_face::syntax as tf_syntax;

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
        _ => SC {
            r: 205,
            g: 214,
            b: 244,
            a: 0xFF,
        },
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
            ScopeSelectors::from_str(scope_str)
                .ok()
                .map(|scope| ThemeItem {
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

/// Maximum number of `(lang, content_hash)` entries kept in the memo-cache.
///
/// Each entry is a `Vec<Vec<HlSpan>>` — roughly proportional to the number of
/// tokens in the block.  256 entries is a very generous bound: real documents
/// rarely have more than a handful of distinct code blocks, so this cap is
/// effectively never hit in normal use.  When it is hit (e.g. a document with
/// hundreds of distinct snippets, or a very long editing session), one arbitrary
/// entry is evicted before the new one is inserted, keeping the map bounded.
const MAX_CACHE_ENTRIES: usize = 256;

pub struct HighlightCache {
    /// Sublime Text 3 default syntaxes — lazily deserialised on first use.
    syntax_set: OnceLock<SyntaxSet>,
    /// Extra syntaxes from the bat/two-face collection (TOML, TypeScript,
    /// Kotlin, Dockerfile, …) — lazily deserialised on first use.
    /// Kept separate from `syntax_set` because syntect binds each
    /// `SyntaxReference` to the `SyntaxSet` it was parsed from; the same set
    /// must be passed to `HighlightLines::highlight_line`.
    extra_syntax_set: OnceLock<SyntaxSet>,
    /// Bundled syntect themes — lazily deserialised, and only when
    /// `palette_theme` is `None` (i.e. never on the default production path).
    theme_set: OnceLock<ThemeSet>,
    /// Palette-derived theme, pre-built at startup when `use_palette_colors = true`.
    /// When `Some`, this takes priority over `theme_name` and `theme_set` is
    /// never initialised.
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
    /// Construction is now O(1): `SyntaxSet` and `ThemeSet` are not deserialised
    /// here — they are initialised lazily on the first call to `highlight_block`.
    ///
    /// `palette_theme` — pass `Some(build_palette_theme(&yame_theme))` to use
    /// palette-derived colours (recommended), or `None` to fall through to the
    /// named `theme_name` built-in.
    pub fn new(enabled: bool, theme_name: String, palette_theme: Option<SyntectTheme>) -> Self {
        Self {
            syntax_set: OnceLock::new(),
            extra_syntax_set: OnceLock::new(),
            theme_set: OnceLock::new(),
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
    ///
    /// Initialises `SyntaxSet` (and `ThemeSet` when needed) on first call.
    pub fn highlight_block(&self, lang: &str, content: &str) -> Option<BlockHighlights> {
        if !self.enabled || lang.is_empty() {
            return None;
        }

        let lang_lower = lang.to_lowercase();
        let hash = hash_str(content);
        let key = (lang_lower.clone(), hash);

        // Cache hit — no syntax/theme work needed.
        if let Some(cached) = self.cache.borrow().get(&key) {
            return Some(cached.clone());
        }

        // Lazily initialise syntax sets on first cache miss.
        // Try the syntect defaults first; fall back to the two-face extras.
        // The active set must be threaded into highlight_line because syntect
        // binds SyntaxReferences to the SyntaxSet they were parsed from.
        let default_set = self
            .syntax_set
            .get_or_init(SyntaxSet::load_defaults_newlines);
        let (syntax, active_set) = if let Some(s) = default_set
            .find_syntax_by_token(&lang_lower)
            .or_else(|| default_set.find_syntax_by_extension(&lang_lower))
        {
            (s, default_set)
        } else {
            let extra_set = self.extra_syntax_set.get_or_init(tf_syntax::extra_newlines);
            let s = extra_set
                .find_syntax_by_token(&lang_lower)
                .or_else(|| extra_set.find_syntax_by_extension(&lang_lower))?;
            (s, extra_set)
        };

        // Resolve theme: palette first (ThemeSet never touched), then named
        // built-in (ThemeSet lazily loaded on first non-palette use).
        let theme: &SyntectTheme = if let Some(pt) = &self.palette_theme {
            pt
        } else {
            let theme_set = self.theme_set.get_or_init(ThemeSet::load_defaults);
            theme_set
                .themes
                .get(&self.theme_name)
                .or_else(|| theme_set.themes.get(Self::FALLBACK_THEME))?
        };

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut block_hl: BlockHighlights = Vec::new();

        for line in LinesWithEndings::from(content) {
            let ranges = highlighter.highlight_line(line, active_set).ok()?;

            let mut line_spans: Vec<HlSpan> = Vec::new();
            let mut char_col = 0usize;

            for (style, piece) in &ranges {
                let char_count = piece.chars().filter(|&c| c != '\n').count();
                if char_count > 0 {
                    line_spans.push(HlSpan {
                        char_start: char_col,
                        char_end: char_col + char_count,
                        fg: Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b),
                    });
                    char_col += char_count;
                }
            }

            block_hl.push(line_spans);
        }

        let mut cache = self.cache.borrow_mut();
        // Evict one arbitrary entry before inserting when the cap is reached.
        // HashMap iteration order is non-deterministic, so "arbitrary" is fine —
        // we have no usage-frequency data and all entries are equally cheap to
        // recompute.  This keeps peak memory O(MAX_CACHE_ENTRIES) without
        // pulling in an LRU crate.
        //
        // The key is cloned out of the immutable borrow before `remove` takes the
        // mutable one; the `bool::then` short-circuits when below the cap.
        let evict_key = (cache.len() >= MAX_CACHE_ENTRIES)
            .then(|| cache.keys().next().cloned())
            .flatten();
        if let Some(k) = evict_key {
            cache.remove(&k);
        }
        cache.insert(key, block_hl.clone());
        Some(block_hl)
    }

    /// Number of entries currently stored in the memo-cache.
    /// Exposed for unit tests; not used on the hot path.
    #[cfg(test)]
    fn cache_len(&self) -> usize {
        self.cache.borrow().len()
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

    /// Print every bundled syntax name and its file extensions — run with
    /// `cargo test enumerate_bundled_syntaxes -- --nocapture --ignored`.
    #[test]
    #[ignore]
    fn enumerate_bundled_syntaxes() {
        let ss = SyntaxSet::load_defaults_newlines();
        let mut names: Vec<String> = ss
            .syntaxes()
            .iter()
            .map(|s| format!("{:30} exts: {:?}", s.name, s.file_extensions))
            .collect();
        names.sort();
        for n in &names {
            println!("{n}");
        }
    }

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
        assert!(
            cache
                .highlight_block("notareallangnobodywoulduse", "hello")
                .is_none()
        );
    }

    #[test]
    fn rust_block_returns_spans() {
        let cache = make_cache();
        let content = "let x: u32 = 42;\n";
        let hl = cache
            .highlight_block("rust", content)
            .expect("rust should be recognised");
        assert_eq!(hl.len(), 1, "one content line → one entry");
        assert!(!hl[0].is_empty(), "rust line must produce spans");
    }

    #[test]
    fn yaml_block_returns_spans() {
        let cache = make_cache();
        let content = "section:\n  key: value\n";
        let hl = cache
            .highlight_block("yaml", content)
            .expect("yaml should be recognised");
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

    // Kills: highlighting.rs `replace hash_str -> u64 with 0` and `with 1`.
    // If hash_str always returns a constant, two different content blocks with
    // the same language key map to the same cache entry.  The second lookup
    // returns the first block's spans (wrong).
    #[test]
    fn cache_miss_different_content_same_language_gives_different_spans() {
        let cache = make_cache();
        let hl_let = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        let hl_fn = cache
            .highlight_block("rust", "fn foo() -> bool { true }\n")
            .unwrap();
        // These two Rust snippets have different token structures and must
        // produce different span lists.  If hash_str → constant, the second
        // call hits the entry from the first call and returns hl_let wrongly.
        assert_ne!(
            hl_let, hl_fn,
            "different Rust content must produce different span lists (hash must be content-dependent)"
        );
    }

    // Kills: highlighting.rs `replace > with >= in highlight_block` (char_count guard).
    // The syntect tokenizer emits a trailing `\n` piece at the end of each line.
    // With `>= 0` (always true for usize) that zero-char piece generates a span
    // with char_start == char_end, violating the non-zero-width invariant.
    #[test]
    fn highlight_block_produces_no_zero_width_spans() {
        let cache = make_cache();
        let hl = cache.highlight_block("rust", "let x: i32 = 42;\n").unwrap();
        for (line_idx, spans) in hl.iter().enumerate() {
            for span in spans {
                assert!(
                    span.char_start < span.char_end,
                    "span on line {line_idx} must be non-zero-width, got {span:?}"
                );
            }
        }
    }

    #[test]
    fn multiline_block_line_count_matches() {
        let cache = make_cache();
        let content = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let hl = cache.highlight_block("rust", content).unwrap();
        assert_eq!(
            hl.len(),
            3,
            "three-line block must produce three line-span lists"
        );
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
        let hl = cache
            .highlight_block("python", "def foo():\n    pass\n")
            .unwrap();
        assert_eq!(hl.len(), 2);
    }

    // ---- default-syntaxes coverage guard ----
    // syntect 5.x bundles the Sublime Text 3 *default* package set — the classic
    // Sublime packages circa ~2015.  Well-supported: Rust, Python, JS, Go, Bash,
    // Ruby, Java, C/C++, SQL, YAML, JSON, HTML, CSS, etc.
    //
    // NOT included (post-2015 / never in ST3 defaults):
    //   TOML, TypeScript, Kotlin, Swift, Dart, SCSS, JSX/TSX, …
    //
    // The `two-face` crate (bat's asset collection) can supply those gaps; see
    // issue #135 if we decide to pull it in.

    #[test]
    fn toml_block_produces_spans() {
        // TOML is not in the syntect default bundle but is supplied by two-face.
        let cache = make_cache();
        let hl = cache
            .highlight_block("toml", "[package]\nname = \"yame\"\nversion = \"0.1.0\"\n")
            .expect("toml must be recognised via the two-face extra syntax set");
        assert_eq!(hl.len(), 3, "three TOML lines → three span-lists");
        assert!(!hl[0].is_empty(), "section header line must produce spans");
    }

    #[test]
    fn typescript_block_produces_spans() {
        // TypeScript is not in the syntect default bundle but is supplied by two-face.
        let cache = make_cache();
        assert!(
            cache
                .highlight_block("typescript", "const x: number = 1;\n")
                .is_some(),
            "typescript must be recognised via the two-face extra syntax set"
        );
    }

    #[test]
    fn bash_block_produces_spans() {
        let cache = make_cache();
        assert!(
            cache
                .highlight_block("bash", "#!/bin/bash\necho hello\n")
                .is_some(),
            "bash must be recognised by the default-syntaxes bundle"
        );
    }

    #[test]
    fn sh_tag_produces_spans() {
        // Both "bash" and "sh" are common fence tags; both should resolve.
        let cache = make_cache();
        assert!(
            cache.highlight_block("sh", "echo hello\n").is_some(),
            "sh tag must resolve to a shell syntax"
        );
    }

    #[test]
    fn go_block_produces_spans() {
        let cache = make_cache();
        assert!(
            cache
                .highlight_block("go", "package main\nfunc main() {}\n")
                .is_some(),
            "go must be recognised by the default-syntaxes bundle"
        );
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

    // Kills: highlighting.rs `delete field foreground from ThemeSettings`
    //        and `delete field background from ThemeSettings` in build_palette_theme.
    // Without these fields the syntect theme uses None for fg/bg, which means
    // unmatched text falls back to syntect defaults instead of the yame palette.
    #[test]
    fn build_palette_theme_sets_foreground_and_background() {
        let theme = Theme::default_theme();
        let pt = build_palette_theme(&theme);
        assert!(
            pt.settings.foreground.is_some(),
            "palette theme must have foreground set in ThemeSettings"
        );
        assert!(
            pt.settings.background.is_some(),
            "palette theme must have background set in ThemeSettings"
        );
    }

    #[test]
    fn palette_cache_rust_returns_spans() {
        let cache = make_palette_cache();
        let hl = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        assert!(
            !hl[0].is_empty(),
            "palette cache must produce spans for rust"
        );
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
        let hl = cache
            .highlight_block("rust", "let s = \"hello\";\n")
            .unwrap();

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

    // ---- cache eviction / boundedness ----

    /// Regression for FEEDBACK-2 §2.3: the memo-cache must not grow without
    /// bound.  Inserting MAX_CACHE_ENTRIES + 1 distinct blocks must leave the
    /// cache at or below the cap — i.e. at least one eviction must have fired.
    #[test]
    fn cache_is_bounded_by_max_entries() {
        let cache = make_cache();
        // Each iteration uses unique content so every call is a cache miss,
        // producing a distinct (lang, hash) key.
        for i in 0..=MAX_CACHE_ENTRIES {
            let content = format!("let x{i} = {i};\n");
            let _ = cache.highlight_block("rust", &content);
        }
        assert!(
            cache.cache_len() <= MAX_CACHE_ENTRIES,
            "cache must not exceed MAX_CACHE_ENTRIES ({MAX_CACHE_ENTRIES}) entries; \
             got {}",
            cache.cache_len()
        );
    }

    /// Inserting exactly MAX_CACHE_ENTRIES entries must not trigger eviction
    /// (the cap is inclusive: we store up to MAX_CACHE_ENTRIES entries before
    /// evicting on the *next* insert).
    #[test]
    fn cache_at_capacity_does_not_lose_entries_prematurely() {
        let cache = make_cache();
        // Fill to exactly the cap.
        for i in 0..MAX_CACHE_ENTRIES {
            let content = format!("let x{i} = {i};\n");
            let _ = cache.highlight_block("rust", &content);
        }
        assert_eq!(
            cache.cache_len(),
            MAX_CACHE_ENTRIES,
            "cache filled to exactly the cap must hold MAX_CACHE_ENTRIES entries"
        );
    }

    #[test]
    fn op_color_is_distinct_from_string_and_keyword_colors() {
        // Validates that the computed op_color doesn't collapse to code_color or
        // accent — i.e. blend(code_color, accent, 0.3) ≠ either endpoint.
        use crate::config::blend_colors;
        let yame = Theme::default_theme();
        let op_color = blend_colors(yame.code_color, yame.accent, 0.3);
        assert_ne!(
            op_color, yame.code_color,
            "op_color must differ from code_color (strings)"
        );
        assert_ne!(
            op_color, yame.accent,
            "op_color must differ from accent (keywords)"
        );
        assert_ne!(
            op_color, yame.text,
            "op_color must differ from text (identifiers)"
        );
    }
}
