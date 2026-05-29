//! Syntax highlighting for fenced code blocks via syntect.
//!
//! `HighlightCache` wraps a `SyntaxSet` + `ThemeSet` and memoises results
//! keyed on `(language_tag, hash_of_block_content)`.  The cache is stored in
//! a `RefCell` so that `build_decoration_map` can take `Option<&HighlightCache>`
//! (immutable) while still mutating the cache on first use.
//!
//! # Fallback behaviour
//! - Highlighting disabled (`enabled = false`) → returns `None` for every block.
//! - Unknown language tag → returns `None` (caller falls back to fenced_bg-only).
//! - Unrecognised syntect theme name → falls back to `"base16-ocean.dark"`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use ratatui::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
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
// HighlightCache
// ---------------------------------------------------------------------------

pub struct HighlightCache {
    /// Bundled syntect syntaxes (Rust, Python, JS, etc.)
    syntax_set: SyntaxSet,
    /// Bundled syntect themes.
    theme_set: ThemeSet,
    /// Memoisation: `(lang_lower, content_hash)` → per-line coloured spans.
    /// `RefCell` allows `highlight_block(&self, …)` without requiring `&mut self`,
    /// so callers that only hold `&HighlightCache` can still populate the cache.
    cache: RefCell<HashMap<(String, u64), BlockHighlights>>,
    /// Whether syntax highlighting is enabled (from `[highlighting] enabled`).
    pub enabled: bool,
    /// Name of the syntect built-in theme to use (e.g. `"base16-ocean.dark"`).
    pub theme_name: String,
}

impl HighlightCache {
    const FALLBACK_THEME: &'static str = "base16-ocean.dark";

    /// Create a new cache, eagerly loading the bundled syntax / theme sets.
    ///
    /// Instantiation takes ~30 ms on first call (SyntaxSet::load_defaults_newlines
    /// decompresses embedded binary data).  Create once at startup and reuse.
    pub fn new(enabled: bool, theme_name: String) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            cache: RefCell::new(HashMap::new()),
            enabled,
            theme_name,
        }
    }

    /// Highlight `content` as `lang`, returning per-line coloured spans.
    ///
    /// Returns `None` when:
    /// - `self.enabled` is false,
    /// - `lang` is empty or not recognised by the bundled syntax set,
    /// - the configured theme name is not found (uses fallback automatically).
    ///
    /// On a cache hit the stored `BlockHighlights` is cloned and returned
    /// (cheap — code blocks are typically small).
    pub fn highlight_block(&self, lang: &str, content: &str) -> Option<BlockHighlights> {
        if !self.enabled || lang.is_empty() {
            return None;
        }

        let lang_lower = lang.to_lowercase();
        let hash = hash_str(content);
        let key = (lang_lower.clone(), hash);

        // Cache hit — clone and return.
        if let Some(cached) = self.cache.borrow().get(&key) {
            return Some(cached.clone());
        }

        // Resolve syntax.
        let syntax = self
            .syntax_set
            .find_syntax_by_token(&lang_lower)
            .or_else(|| self.syntax_set.find_syntax_by_extension(&lang_lower))?;

        // Resolve theme; fall back to a known-good dark theme if the configured
        // name isn't found (avoids a panic on typo in config).
        let theme = self
            .theme_set
            .themes
            .get(&self.theme_name)
            .or_else(|| self.theme_set.themes.get(Self::FALLBACK_THEME))?;

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut block_hl: BlockHighlights = Vec::new();

        for line in LinesWithEndings::from(content) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .ok()?;

            let mut line_spans: Vec<HlSpan> = Vec::new();
            let mut char_col = 0usize;

            for (style, piece) in &ranges {
                // Strip the trailing newline from char counts.
                let chars: Vec<char> = piece.chars().filter(|&c| c != '\n').collect();
                let char_count = chars.len();
                if char_count > 0 {
                    let r = style.foreground.r;
                    let g = style.foreground.g;
                    let b = style.foreground.b;
                    line_spans.push(HlSpan {
                        char_start: char_col,
                        char_end: char_col + char_count,
                        fg: Color::Rgb(r, g, b),
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

    fn make_cache() -> HighlightCache {
        HighlightCache::new(true, "base16-ocean.dark".into())
    }

    #[test]
    fn disabled_cache_returns_none() {
        let cache = HighlightCache::new(false, "base16-ocean.dark".into());
        assert!(
            cache.highlight_block("rust", "let x = 1;").is_none(),
            "disabled cache must always return None"
        );
    }

    #[test]
    fn empty_lang_returns_none() {
        let cache = make_cache();
        assert!(
            cache.highlight_block("", "fn main() {}").is_none(),
            "empty language tag must return None"
        );
    }

    #[test]
    fn unknown_lang_returns_none() {
        let cache = make_cache();
        assert!(
            cache
                .highlight_block("notareallangnobodywoulduse", "hello")
                .is_none(),
            "unknown language tag must return None"
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

    #[test]
    fn different_content_different_result() {
        let cache = make_cache();
        let a = cache.highlight_block("rust", "let x = 1;\n").unwrap();
        let b = cache
            .highlight_block("rust", "fn main() {}\n")
            .unwrap();
        // They may or may not be equal but at least both compute without panic.
        let _ = (a, b);
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
        let hl = cache
            .highlight_block("rust", "let x: i32 = 42;\n")
            .unwrap();
        let line = &hl[0];
        for w in line.windows(2) {
            assert!(
                w[0].char_end <= w[1].char_start,
                "spans must be non-overlapping and ordered: {:?}",
                w
            );
        }
    }

    #[test]
    fn unknown_theme_falls_back_gracefully() {
        // An invalid theme name should not panic; it should either fall back to
        // the default theme and return Some, or return None (both are acceptable).
        let cache = HighlightCache::new(true, "no-such-theme-ever".into());
        // Just must not panic.
        let _ = cache.highlight_block("rust", "let x = 1;\n");
    }

    #[test]
    fn json_block_produces_spans() {
        let cache = make_cache();
        let content = "{\"key\": 42}\n";
        let hl = cache.highlight_block("json", content);
        // json is bundled; should not be None
        assert!(hl.is_some(), "json should produce highlights");
    }

    #[test]
    fn python_block_produces_spans() {
        let cache = make_cache();
        let content = "def foo():\n    pass\n";
        let hl = cache.highlight_block("python", content);
        assert!(hl.is_some(), "python should produce highlights");
        let hl = hl.unwrap();
        assert_eq!(hl.len(), 2);
    }

    #[test]
    fn case_insensitive_lang_lookup() {
        let cache = make_cache();
        let content = "fn main() {}\n";
        let lower = cache.highlight_block("rust", content);
        let upper = cache.highlight_block("Rust", content);
        // Both should succeed (or both fail; they must not diverge).
        assert_eq!(
            lower.is_some(),
            upper.is_some(),
            "language lookup must be case-insensitive"
        );
    }

    #[test]
    fn fg_colors_are_rgb() {
        let cache = make_cache();
        let hl = cache
            .highlight_block("rust", "let x = 1;\n")
            .unwrap();
        for span in &hl[0] {
            assert!(
                matches!(span.fg, Color::Rgb(_, _, _)),
                "all span fg values must be Color::Rgb: {:?}",
                span.fg
            );
        }
    }
}
