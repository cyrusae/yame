# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Workflow (mandatory)

**Chainlink issue tracking is enforced by a pre-tool hook — you will be blocked if you skip it.**

Before writing any code:
1. `chainlink quick "title" -p <priority> -l <label>` — create an issue
2. `chainlink session work <id>` — mark focus

After finishing:
- `chainlink session end --notes "..."` — always close the session with handoff notes

## Commands

```sh
cargo build                      # build
cargo run -- <file.md>           # run
cargo test                       # all tests (~820 tests)
cargo test <name>                # single test by name substring
cargo clippy -- -D warnings      # lint (no warnings allowed)
cargo fmt --check                # format check
cargo mutants                    # mutation testing (slow; run selectively)
```

Run `cargo test` after every change. Fix warnings — do not suppress them.

## Architecture

yame is a Rust TUI app built on **ratatui** + **tui-textarea** + **crossterm**. It is both a binary (`src/main.rs`) and a library (`src/lib.rs`) so integration tests can use internal modules.

### Data flow

1. **`src/main.rs`** — entry point; sets up crossterm alternate screen, loads config, creates `App`, runs event loop
2. **`src/config.rs`** — deserializes `~/.config/yame/config.toml`; resolves palette presets and per-token theme overrides into a `Theme` struct; all color tokens live here
3. **`src/app.rs`** — `App` struct: owns the `TextArea`, `DecorationMap`, `SearchState`, `HighlightCache`, `StatusLine`, `Theme`, and file metadata; `FileMode` enum decides decoration vs. plain-highlight vs. plain-text rendering
4. **`src/input.rs`** — `event_loop` (terminal I/O, `#[mutants::skip]`); all keypress handlers; decoration rebuild is triggered here after edits
5. **`src/decoration/`** — Markdown → `DecorationMap` pipeline:
   - `mod.rs` — `build_decoration_map`: runs pulldown-cmark, emits `StyledSpan`s per Markdown event
   - `spans.rs` — `StyledSpan`, `SpanParams`, `DecorationMap` internals; byte↔char index helpers
   - `emit.rs` — span helpers used by `build_decoration_map` (modifier overlay, content-around-existing)
   - `words.rs` — word count, link split logic
   - `frontmatter.rs` — YAML/TOML frontmatter detection
   - `highlight.rs` — syntect highlight spans for fenced code blocks
   - `builder/` — (sub-module of decoration)
6. **`src/renderer/`** — ratatui `Widget` impl that applies `DecorationMap` to the textarea buffer:
   - `mod.rs` — main render loop: soft-wraps lines, paints backgrounds, underlines headings, overlays selections and search highlights; `wrap_line` (column-aware soft wrap), `left_gutter_width`
   - `status.rs` — status bar and info line rendering
   - `search_bar.rs` — search/replace bar and help modals
   - `settings_modal.rs` — live settings modal
   - `utils.rs` — `shorten_path`, `format_thousands`, `split_into_spans`
7. **`src/layout.rs`** — `compute_layout`: maps terminal area to editor column width, centering, gutter regions
8. **`src/search.rs`** — `SearchState`: regex/literal search, match list, find/replace
9. **`src/highlighting.rs`** — `HighlightCache`: syntect engine wrapper; optional palette-derived theme
10. **`src/commands.rs`** — stateless cursor/scroll commands (clamp_scroll, visual move)
11. **`src/settings.rs`** + **`src/status.rs`** — settings modal state and status-line state

### Key types

| Type | File | Role |
|---|---|---|
| `App` | `src/app.rs` | Central state container |
| `Theme` | `src/config.rs` | All resolved color tokens |
| `FileMode` | `src/app.rs` | `Markdown` / `PlainHighlight(lang)` / `PlainText` |
| `DecorationMap` | `src/decoration/spans.rs` | `Vec<StyledSpan>` keyed by line; consumed by renderer |
| `StyledSpan` | `src/decoration/spans.rs` | char-range + style for one inline decoration |
| `EditorLayout` | `src/layout.rs` | Column widths, gutter, offsets for one terminal frame |

### Renderer internals worth knowing

- `GUTTER = 1` — the single left-margin cell used when line numbers are off
- `left_gutter_width(total_lines, show_line_numbers)` — returns 1 when line numbers off, `digits + 2` when on
- Heading background fills from col 0 when line numbers are off (full row width)
- Heading underline uses `ul_offset = 0` when line numbers off, `left_gutter` when on — keeps them aligned

### Mutation testing

`mutants.toml` (symlinked from `.cargo/mutants.toml`) lists all structurally unkillable mutants with explanations. When adding code, add `#[mutants::skip]` to I/O-only functions; for genuinely unkillable logic mutations, add a new `exclude_re` entry with a column anchor and explanation.

### Tests

- Unit tests are inline (`#[cfg(test)]` blocks) in most modules
- Integration tests live in `tests/integration.rs` with fixtures in `tests/fixtures/`
- `src/tests.rs` (included from `main.rs`) covers CLI-adjacent paths
- Test helpers: `make_theme()` / `build_map()` patterns repeated across test modules
