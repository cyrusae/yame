# `decoration/mod.rs` Refactor Plan
*Issue #181 — post-v0.2.0*

## Motivation

`src/decoration/mod.rs` is 5,326 lines: ~1,600 production + ~3,700 tests.  The
production code itself is not catastrophically large, but the entire codebase
for one logical subsystem living in a single file makes it hard to navigate,
slow to review, and awkward to mutation-test by subcomponent.  The mutation
test suite we completed for v0.2.0 is the ideal safety net to do this refactor
without regression risk.

---

## Current layout

```
src/decoration/
├── mod.rs      5326 lines  ← everything
├── spans.rs    (byte↔char helpers, SpanParams)
└── words.rs    (word count)
```

Key sections in `mod.rs`:

| Lines       | Content |
|-------------|---------|
| 1–91        | Span-emission helpers (`add_modifier_to_existing`, `emit_content_around_existing`) |
| 93–172      | Bold+italic helper (`emit_bold_italic_spans`) |
| 174–193     | Adjacency predicates (`is_strong_outer_adjacent_to_emphasis`) |
| 194–231     | **Types**: `StyledSpan`, `DecorationMap` |
| 232–270     | `block_highlights_to_decoration_map` |
| 272–356     | Frontmatter detection + helpers |
| 357–451     | `apply_frontmatter_spans` |
| 452–631     | Highlight/search helpers |
| 632–727     | Parser options + fenced helpers |
| 728–1596    | `build_decoration_map` (~870 lines, 1 giant match) |
| 1598–5326   | `#[cfg(test)]` block (~3,700 lines) |

---

## Target layout

```
src/decoration/
├── mod.rs          ~150 lines  — imports, re-exports, thin public API
├── types.rs         ~60 lines  — StyledSpan, DecorationMap
├── emit.rs         ~200 lines  — add_modifier_to_existing, emit_content_around_existing,
│                                  emit_bold_italic_spans, is_strong_outer_adjacent_to_emphasis
├── frontmatter.rs  ~200 lines  — detect_frontmatter + helpers + apply_frontmatter_spans
├── highlight.rs    ~190 lines  — highlight_re, in_content_zone, fill_highlight_gaps,
│                                  apply_highlight_spans
├── block_hl.rs      ~50 lines  — block_highlights_to_decoration_map
├── builder/
│   ├── mod.rs      ~150 lines  — BuildState struct, build_decoration_map, event dispatch loop
│   ├── headings.rs ~120 lines  — Event::Start/End Heading handlers
│   ├── inline.rs   ~230 lines  — Strong, Emphasis, inline Code handlers
│   ├── fenced.rs   ~200 lines  — CodeBlock handlers + fence_line_end helpers
│   ├── blocks.rs   ~230 lines  — BlockQuote, List, Item, TaskListMarker handlers
│   ├── tables.rs   ~120 lines  — Table, TableHead handlers
│   └── misc.rs     ~120 lines  — Strikethrough, Rule, Text, Link handlers
├── spans.rs        (unchanged)
└── words.rs        (unchanged)
```

Tests stay **co-located** with the code they test: move each `// ---- X. ----`
section into the `#[cfg(test)]` block of the file that owns that handler.
The `mod.rs` test block keeps only integration-level and cross-cutting tests.

---

## The key enabler: `BuildState`

`build_decoration_map` is one giant function because all its handlers share
mutable local state.  The refactor is only tractable if we lift that state into
a struct first.

### State audit

All local variables in `build_decoration_map` that cross event boundaries:

```rust
// src/decoration/builder/mod.rs

pub(crate) struct BuildState<'a> {
    // --- inputs (immutable) ---
    pub text:            &'a str,
    pub theme:           &'a Theme,
    pub italic_support:  bool,
    pub highlight_cache: Option<&'a HighlightCache>,
    pub line_starts:     Vec<usize>,   // precomputed from text

    // --- outputs (accumulate as events fire) ---
    pub map:             DecorationMap,
    pub word_count:      usize,

    // --- per-event-sequence state ---
    pub in_ordered_list: bool,
    pub in_strong:       Option<std::ops::Range<usize>>,
    pub in_emphasis:     Option<std::ops::Range<usize>>,
    pub in_table_head:   Option<std::ops::Range<usize>>,
    pub code_block_lines: HashSet<usize>,
}
```

`build_decoration_map` becomes a thin orchestrator:

```rust
pub fn build_decoration_map(
    text: &str,
    theme: &Theme,
    italic_support: bool,
    highlight_cache: Option<&HighlightCache>,
) -> (DecorationMap, usize) {
    let mut s = BuildState::new(text, theme, italic_support, highlight_cache);

    let parser = Parser::new_ext(text, parser_options()).into_offset_iter();
    for (event, range) in parser {
        match event {
            Event::Start(Tag::Heading { level, .. })   => headings::on_start(&mut s, level, range),
            Event::Start(Tag::Strong)                   => inline::on_strong_start(&mut s, range),
            Event::End(TagEnd::Strong)                  => inline::on_strong_end(&mut s, range),
            Event::Start(Tag::Emphasis)                 => inline::on_emphasis_start(&mut s, range),
            Event::End(TagEnd::Emphasis)                => inline::on_emphasis_end(&mut s, range),
            Event::Code(ref val)                        => inline::on_code(&mut s, val, range),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                fenced::on_start(&mut s, lang, range)
            }
            Event::Start(Tag::BlockQuote(_))            => blocks::on_blockquote_start(&mut s, range),
            Event::Start(Tag::List(kind))               => blocks::on_list_start(&mut s, kind),
            Event::End(TagEnd::List(_))                 => blocks::on_list_end(&mut s),
            Event::Start(Tag::Item)                     => blocks::on_item_start(&mut s, range),
            Event::TaskListMarker(checked)              => blocks::on_task_marker(&mut s, checked, range),
            Event::Start(Tag::Table(_))                 => tables::on_table_start(&mut s, range),
            Event::Start(Tag::TableHead)                => tables::on_table_head_start(&mut s, range),
            Event::End(TagEnd::TableHead)               => tables::on_table_head_end(&mut s, range),
            Event::Start(Tag::Strikethrough)            => misc::on_strikethrough(&mut s, range),
            Event::Rule                                 => misc::on_rule(&mut s, range),
            Event::Start(Tag::Link { .. })              => misc::on_link(&mut s, range),
            Event::Text(ref val)                        => misc::on_text(&mut s, val, range),
            _ => {}
        }
    }

    // Post-pass: apply ==highlight== and frontmatter spans.
    apply_highlight_spans(&mut s.map, text, theme, &s.code_block_lines);
    if let Some(fm_end) = detect_frontmatter(text) {
        apply_frontmatter_spans(&mut s.map, text, theme, fm_end, &s.line_starts);
    }

    (s.map, s.word_count)
}
```

Each handler module only imports `BuildState` and whatever stdlib/crate types it
needs — no cross-module coupling beyond `BuildState`.

---

## Phases

### Phase 1 — Extract pure, stateless modules (low risk)

Move code with zero cross-module dependencies first.  Each step is a pure
mechanical move: copy to new file, add `mod X;` + re-exports to `mod.rs`,
confirm `cargo test` still passes.

| Step | Move | Into |
|------|------|------|
| 1a | `StyledSpan`, `DecorationMap` | `types.rs` |
| 1b | `add_modifier_to_existing`, `emit_content_around_existing`, `emit_bold_italic_spans`, `is_strong_outer_adjacent_to_emphasis` | `emit.rs` |
| 1c | `detect_frontmatter`, `frontmatter_line_end`, `frontmatter_zero_start_span`, `apply_frontmatter_spans` | `frontmatter.rs` |
| 1d | `highlight_re`, `in_content_zone`, `fill_highlight_gaps`, `apply_highlight_spans` | `highlight.rs` |
| 1e | `block_highlights_to_decoration_map` | `block_hl.rs` |

Move co-located test sections with each step.

Checkpoint: `cargo test` must pass after every single step.

---

### Phase 2 — Introduce `BuildState` (moderate risk)

**2a** Define `BuildState` struct in `builder/mod.rs` with `new()` and
`finish()` methods.  **Do not yet move any handler logic** — just convert
`build_decoration_map` to construct a `BuildState`, run the existing match in
place, and at the end call `finish()` to return the pair.  Run tests.

**2b** Extract handler modules one event family at a time, in this order
(simplest → most complex):

1. `misc.rs` — Rule, Text (trivial; no inter-event state)
2. `headings.rs` — Heading Start/End (uses only local variables, no cross-event state)
3. `tables.rs` — Table + TableHead (uses `in_table_head` state only)
4. `inline.rs` — Strong, Emphasis, Code (uses `in_strong`, `in_emphasis`)
5. `fenced.rs` — CodeBlock (uses `code_block_lines`; most complex due to syntect path)
6. `blocks.rs` — BlockQuote, List, Item, TaskListMarker (uses `in_ordered_list`)
7. `misc.rs` additions — Strikethrough, Link

After each extraction: `cargo test`.

**2c** Run `cargo mutants` on the new layout to confirm coverage didn't regress.
Update `mutants.toml` if any line-anchored rules need adjusting (use `\d+`
anchors as established in v0.2.0).

---

### Phase 3 — Test hygiene (low risk, optional)

Move each `// ---- X. ----` test section from the monolithic `#[cfg(test)]` into
the `#[cfg(test)]` block of the file that now owns that code.  The `mod.rs` test
block shrinks to integration-level tests only (smoke tests, fixture tests,
cross-cutting byte-mapping tests).

This is mechanical and can be done handler-by-handler after Phase 2 lands.

---

## Risk notes

| Risk | Mitigation |
|------|------------|
| Rust visibility: handlers call `use crate::decoration::spans::*` etc. | Use `pub(super)` or `pub(crate)` on `BuildState` fields; handler modules are submodules of `builder`, so `super::` resolves to `builder`. |
| `#[mutants::skip]` annotations move with their functions | Move the attribute with the function body; no extra work. |
| `mutants.toml` `exclude_re` patterns reference paths like `decoration/mod.rs` | After Phase 2, patterns that reference `decoration/mod.rs` path fragments will need updating to `decoration/builder/mod.rs` or the specific handler file.  Do this as part of step 2c. |
| Test breakage from import path changes | Each step ends with `cargo test`; catch immediately. |
| Large diff makes review hard | Enforce one PR per phase, ideally one PR per step within Phase 2. |

---

## Definition of done

- [ ] `src/decoration/mod.rs` is ≤ 200 production lines (re-exports + thin orchestrator)
- [ ] No single file in `src/decoration/**` exceeds 500 production lines
- [ ] `cargo test` passes
- [ ] `cargo clippy` produces no warnings
- [ ] `cargo mutants` coverage does not regress from v0.2.0 baseline
- [ ] `mutants.toml` patterns updated if any paths changed
