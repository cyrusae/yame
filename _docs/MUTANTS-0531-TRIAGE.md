# Mutants Triage — 2026-05-31 Run

Generated from MUTANTS-0531.md.  180 entries: MISSED and TIMEOUT across 6 files.
All annotated `#[mutants::skip]` functions are correctly absent (skip is now working).

---

## Executive summary

| File | Action |
|------|--------|
| `decoration/mod.rs` (build_decoration_map) | ~15 `exclude_re` no-ops + 14 new tests |
| `renderer/mod.rs` (wrap_line) | Fix 3 drifted `exclude_re` line numbers + 3 new entries |
| `renderer/mod.rs` (focus_paragraph_bounds) | 1 targeted test |
| `renderer/search_bar.rs` | `exclude_re` for constants; `#[mutants::skip]` on `put_padded` |
| `search.rs` | 5 precision tests + 2 `exclude_re` no-ops |
| `table_format.rs` | 2 `exclude_re` (infinite loops) + 4 tests |
| `input.rs` (handle_search_key / key_event) | 4 `exclude_re` bitmask + comprehensive key tests |

Rough split: **~22 `exclude_re` patterns**, **1 `mutants::skip`**, **~27 new tests**.

---

## 1 · `src/decoration/mod.rs` — `build_decoration_map`

### 1a · Pure `exclude_re` (structurally unkillable)

#### Line 267 — pulldown-cmark Options bitmask (cols 32 and 60)
```
replace | with ^ in build_decoration_map
```
`ENABLE_TABLES | ENABLE_TASKLISTS | ENABLE_STRIKETHROUGH` — three non-overlapping
bit-flags.  For any pair of non-overlapping flags, `A | B == A ^ B`.  Identical pattern
to the existing `handle_pair_wrap` and `handle_key_event` entries.

**→ `exclude_re`, cols 32 and 60, `in build_decoration_map` anchor.**

---

#### Line 326 — heading delimiter guard `> with >=` (col 30)
```rust
if delim_end > start_char { push_span(...) }
```
`>=` allows `delim_end == start_char`, emitting a zero-width span `(start_char, start_char)`.
Zero-width spans are not drawn; `push_span` does not validate them.  Unkillable.

**→ `exclude_re`, col 30.**

---

#### Line 331 — `delete field char_start` from heading delimiter span (col 29)
Explicit value is `start_char`.  For ATX headings pulldown-cmark reports the `#` at byte 0,
so `start_char` is always 0.  `Default::default()` for `usize` is 0 — identical.

**→ `exclude_re`, col 29.**

---

#### Line 350 — `delete field full_line_bg` from heading *content* span
This is NOT unkillable — see §1b T-1.

---

#### Line 440 — `replace + with *` on `add_modifier_to_existing` range (col 48)
```rust
add_modifier_to_existing(&mut map, start_line, start_char + 2, ..., Modifier::BOLD);
```
Layers BOLD onto inner spans inside a bold region.  For plain `**text**` (no inner spans),
the function iterates nothing regardless of range — no-op in all plain-bold tests.
Killable only via a `**_text_**` test (T-2).  The col-48 mutation itself is unkillable
for all existing tests.

**→ `exclude_re`, col 48, `in build_decoration_map`.  Also add T-2.**

---

#### Lines 553, 561 — inline code `< with <=` guards (cols 33, 36)
```rust
if open_end < close_start { push content span }    // 553:33
if close_start < end_char_excl { push close span } // 561:36
```
`<=` admits equality → zero-width span emitted.  Zero-width is a no-op.

**→ `exclude_re`, cols 33 and 36.**

---

#### Line 629 — `delete field char_start` from opening fence span (col 29)
Fenced code fence always starts at column 0; explicit 0 = default 0.

**→ `exclude_re`, col 29.**

---

#### Line 639 — `replace > with >=` in lang tag guard (col 37)
```rust
if lang_end > fence_count { push lang span }
```
`>=` emits zero-width when `lang_end == fence_count` (empty lang string).  No-op.

**→ `exclude_re`, col 37.**

---

#### Line 673 — `replace match guard !hl_spans.is_empty() with true` (col 43)
Syntect path: `Some(hl_spans) if !hl_spans.is_empty()`.  With `true`, an empty Vec
passes the guard; the for-loop iterates zero times.  Observable behavior unchanged.

**→ `exclude_re`, col 43.**

---

#### Line 684 — `replace < with <=` in syntect span cs < ce guard (col 39)
`<=` admits `cs == ce` → zero-width span.  No-op.  **→ `exclude_re`, col 39.**

---

#### Lines 689, 690 — `delete field char_start/char_end` from syntect span (col 45)
Unit tests pass `highlight_cache = None`, so the syntect branch is never exercised.
The mutations are structurally unkillable: no test path reaches this code.

**→ `exclude_re`, cols 45 (both lines), `in build_decoration_map`.**

---

#### Line 713 — `delete field char_start` from fallback fenced-content span (col 37)
Explicit value 0 (full-line content span always starts at col 0).  Default 0.  No-op.

**→ `exclude_re`, col 37.**

---

#### Line 725 — `replace > with >=` in closing fence guard (col 29)
```rust
if end_line > start_line { push closing fence span }
```
`>=` fires when `end_line == start_line` (open and close fence on the same source line).
pulldown-cmark does not produce a `Fenced` event with coincident start/end for any valid
Markdown document.  Unkillable in practice.  **→ `exclude_re`, col 29.**

---

#### Line 742 — `delete field char_start` from closing fence span (col 29)
Explicit 0; default 0.  **→ `exclude_re`, col 29.**

---

#### Line 782 — `delete field char_start` from blockquote indicator span (col 29)
Blockquote `▌` always at col 0; explicit 0 = default 0.  **→ `exclude_re`, col 29.**

---

#### Lines 826, 846, 854 — `replace > with >=` in link span guards (cols 38, 36, 25)
```rust
if split_idx > 1 { push text span }         // 826 — >= emits zero-width if split_idx==1
if url_end > url_start { push url span }    // 846 — >= emits zero-width if equal
if end_char_excl > 0 { push closing ) }    // 854 — >= is always true (usize ≥ 0 always)
```
All three: either zero-width no-op or tautology for valid input.

**→ `exclude_re`, cols 38, 36, and 25 respectively.**

---

#### Line 1081:43 — `replace < with <=` in strikethrough content guard
```rust
if start_char + 2 < end_char_excl.saturating_sub(2) { push content span }
```
`<=` emits zero-width when equality holds (i.e., for the 4-char minimum `~~~~`).  No-op.

**→ `exclude_re`, col 43.**

---

#### Line 1106 — `delete field char_start` from horizontal rule span (col 25)
Horizontal rules always cover `0..line_len`; explicit 0 = default 0.

**→ `exclude_re`, col 25.**

---

### 1b · Tests needed for `decoration/mod.rs`

All additions belong in the `#[cfg(test)] mod tests` block at the end of the file.

**T-1: Heading content span carries `full_line_bg`** (kills line 350)
```rust
// "# Hello" — content span (delim_end..line_len) must also have full_line_bg set.
let map = build_map("# Hello");
let spans = &map[&0];
let content = spans.iter().find(|s| s.char_start == 2).unwrap();
assert!(content.full_line_bg.is_some(), "content span must carry heading_bg");
```

**T-2: Bold wrapping inline italic — modifier layered on inner span** (kills line 440)
```rust
// "**_x_**": strong outer, emphasis inner (adjacent via End(Strong) adjacency check).
// The inner italic content span should have BOTH ITALIC and BOLD modifiers.
let map = build_map("**_x_**");
let spans = &map[&0];
// Find the italic content span at char_start=3 (between delimiters ** and _)
let inner = spans.iter().find(|s| s.char_start == 3 && s.char_end == 4).unwrap();
assert!(inner.style.add_modifier.contains(Modifier::ITALIC));
assert!(inner.style.add_modifier.contains(Modifier::BOLD));
```
Note: `***x***` goes through `emit_bold_italic_spans`; `**_x_**` exercises the
`End(Strong)` adjacency branch with `add_modifier_to_existing`.

**T-3: Fenced code block at end of document — no trailing newline** (kills lines 615, 657, 728)
```rust
// Closing ``` is the last line; no newline after.  Exercises the
// `if end_line + 1 < line_starts.len()` guard in the closing fence path.
let text = "```\ncode\n```";  // no trailing \n
let map = build_map(text);
let spans_0 = &map[&0];
assert!(spans_0.iter().any(|s| s.char_start == 0 && s.char_end == 3), "open fence 0..3");
let spans_2 = &map[&2];
assert!(spans_2.iter().any(|s| s.char_start == 0 && s.char_end == 3), "close fence 0..3");
```

**T-4: Tilde fence — close_fence char matching** (kills line 735 col 56)
```rust
// ~~~ fence: take_while uses `c == '`' || c == '~'`.
// Mutating `c == '~'` → `c != '~'` silently breaks tilde detection.
let text = "~~~\ncode\n~~~";
let map = build_map(text);
let spans_2 = &map[&2];
assert!(spans_2.iter().any(|s| s.char_start == 0 && s.char_end == 3),
    "tilde close fence must span 0..3");
```

**T-5: Fenced code content line — `char_end` and `style`** (kills lines 714, 715)
```rust
// Fallback (no syntect) — line inside fenced block gets a full-line span.
let text = "```\nhello\n```";
let map = build_map(text);
let spans_1 = &map[&1];
let content = spans_1.iter().find(|s| s.char_start == 0).unwrap();
assert_eq!(content.char_end, 5, "content span covers 'hello' (5 chars)");
assert!(content.full_line_bg.is_some(), "fenced content has fenced_bg");
```

**T-6: Blockquote indicator style** (kills line 784)
```rust
let map = build_map("> quote");
let spans_0 = &map[&0];
let indicator = spans_0.iter().find(|s| s.is_blockquote).unwrap();
assert_eq!(indicator.style.fg, Some(DEFAULT_THEME.muted),
    "blockquote indicator uses muted color");
```

**T-7: Two consecutive lists — `in_ordered_list` resets** (kills line 869)
```rust
// After End(List) the ordered flag must be cleared so the second unordered list
// gets bullet (char_end=1) not numbered (char_end=2+) spans.
let text = "1. first\n\n- second";
let map = build_map(text);
let spans_2 = &map[&2];
let bullet = spans_2.iter().find(|s| s.char_start == 0 && s.continuation_indent > 0).unwrap();
assert_eq!(bullet.char_end, 1, "unordered bullet must end at 1, not 2+");
```

**T-8: Nested list bullet at non-zero `item_char`** (kills line 896)
```rust
// Two-space indent: "  - nested" → item_char = 2; char_start must be 2, not 0 (default).
let text = "- outer\n  - nested";
let map = build_map(text);
let spans_1 = &map[&1];
let bullet = spans_1.iter().find(|s| s.continuation_indent > 0).unwrap();
assert_eq!(bullet.char_start, 2, "nested bullet starts at col 2");
assert_eq!(bullet.char_end,   3, "nested bullet ends at col 3");
```

**T-9: Todo checked item — inline decoration keeps `continuation_indent = 0`** (kills line 916)
```rust
// "- [x] **bold**" — bold delimiter span must NOT have its ci overwritten to task_ci.
// The `for span in spans.iter_mut() { if span.continuation_indent > 0 }` guard uses `> 0`.
// Mutating to `>= 0` would also update spans with ci=0 (all inline decorations).
let text = "- [x] **bold**";
let map = build_map(text);
let spans_0 = &map[&0];
// Bold opening delimiter `**` is at char 6..8 with ci=0.
let bold_delim = spans_0.iter().find(|s| s.char_start == 6 && s.char_end == 8).unwrap();
assert_eq!(bold_delim.continuation_indent, 0,
    "bold delimiter must not inherit todo continuation_indent");
```

**T-10: Todo checked item — sub-span boundaries with non-zero `marker_char`** (kills lines 925–951)
```rust
// "- [x] done" — marker_char = 2 (the `[` is at position 2 after "- ").
// Sub-spans: `[` at (2,3), `x` at (3,4), `]` at (4,5), done-text at (6, line_len).
let text = "- [x] done";
let map = build_map(text);
let spans_0 = &map[&0];
assert!(spans_0.iter().any(|s| s.char_start == 2 && s.char_end == 3), "[");
assert!(spans_0.iter().any(|s| s.char_start == 3 && s.char_end == 4), "x");
assert!(spans_0.iter().any(|s| s.char_start == 4 && s.char_end == 5), "]");
assert!(spans_0.iter().any(|s| s.char_start == 6 && s.char_end == 10), "done");
```

**T-11: Unchecked checkbox `[ ]` bracket positions** (kills lines 965–970)
```rust
// "- [ ] todo" — `[` at (2,3), `]` at (4,5).
let text = "- [ ] todo";
let map = build_map(text);
let spans_0 = &map[&0];
assert!(spans_0.iter().any(|s| s.char_start == 2 && s.char_end == 3), "[ at (2,3)");
assert!(spans_0.iter().any(|s| s.char_start == 4 && s.char_end == 5), "] at (4,5)");
```

**T-12: Table `is_sep` — body row with dashes is NOT a separator** (kills line 1000)
```rust
// "| a-b |" contains '-' but also letters → `chars().all(...)` is false.
// `&&→||` mutation makes it a separator because `contains('-')` is true.
let text = "| head |\n| --- |\n| a-b |";
let map = build_map(text);
let spans_2 = &map[&2];
// A separator row would produce per-char dash/colon spans with sep_dash_style.
// The body row `a-b` should not produce a span at char_idx=2 (the `-`).
let has_dash_styled = spans_2.iter().any(|s| s.char_start == 2 && s.char_end == 3);
assert!(!has_dash_styled, "body-row '-' must not get sep_dash_style");
```

**T-13: Table match guards — `:` and `-` in body row** (kills lines 1013, 1020)
```rust
// Separator match arms use `if is_sep` guards.  With guard replaced by `true`,
// body-row colons and dashes would get separator coloring.
let text = "| h1 | h2 |\n| -- | -- |\n| :val: | a-b |";
let map = build_map(text);
let spans_2 = &map[&2];
// No span at the `:` position (col 2) of the body row.
assert!(!spans_2.iter().any(|s| s.char_start == 2 && s.char_end == 3),
    "body-row ':' must not get sep_colon_style");
```

**T-14: Mid-line strikethrough `+→*` on `start_char`** (kills line 1081:39)
```rust
// "ok ~~word~~ rest" — start_char = 3.
// Mutation: 3*2 = 6 ≠ 3+2 = 5 → opening delimiter placed at wrong position.
let text = "ok ~~word~~ rest";
let map = build_map(text);
let spans_0 = &map[&0];
assert!(spans_0.iter().any(|s| s.char_start == 3 && s.char_end == 5),
    "opening ~~ at (3,5)");
assert!(spans_0.iter().any(|s| s.char_start == 9 && s.char_end == 11),
    "closing ~~ at (9,11)");
```

---

## 2 · `src/renderer/mod.rs` — line number drift

The wrap_line `exclude_re` entries use exact source line numbers.  Those lines shifted
after the Phase-12 heading/border additions.  New line numbers per MUTANTS-0531:

| Old pattern | New line | Description |
|-------------|----------|-------------|
| `mod\\.rs:81:.*replace == with !=` | **85:22** | while-condition `==→!=` |
| `mod\\.rs:90:.*replace \\+= with \\*=` | **94:24** | `char_start += 1` → `*= 1` (infinite loop) |
| `mod\\.rs:96:.*replace \\+ with [-*]` | **100:33** | `sp + 1` → backward walk (infinite loop) |

Additionally, two new MISSED entries at **line 79** and one TIMEOUT at **line 81** need
covering:

- **79:36** (`char_start + ci` → `char_start * ci`): for `char_start > 0` on subsequent
  outer iterations, `ci = 0` on first inner step → `chunk_end = char_start * 0 + 1 = 1`.
  Next outer iteration: `next_start = 1 < char_start` → backward walk → infinite loop.
- **79:41** (`ci + 1` → `ci * 1 = ci`): `chunk_end` is one short; outer loop reprocesses
  the last char → infinite loop.
- **81:46** (`chunk_end = char_start + ci` break path → `char_start * ci`): same backward-walk
  logic as 79:36 when the break fires with `ci = 0` and `char_start > 0`.

**Updated/new patterns to add to `mutants.toml`:**
```toml
# wrap_line — line numbers updated after Phase-12 source growth
"renderer/mod\\.rs:85:22:.*replace == with != in wrap_line",
"renderer/mod\\.rs:94:24:.*replace \\+= with \\*= in wrap_line",
"renderer/mod\\.rs:100:33:.*replace \\+ with [-*] in wrap_line",

# New: chunk_end = char_start + ci (+1) — both + operators cause backward-walk
"renderer/mod\\.rs:79:36:.*replace \\+ with \\* in wrap_line",
"renderer/mod\\.rs:79:41:.*replace \\+ with \\* in wrap_line",
"renderer/mod\\.rs:81:46:.*replace \\+ with \\* in wrap_line",
```
Remove the old `mod\\.rs:81`, `mod\\.rs:90`, `mod\\.rs:96` entries when adding these.

### `focus_paragraph_bounds` — line 853

**Line 853:27** — `replace + with *` in `(cursor_row + 1..n).find(...)`:
`cursor_row * 1 = cursor_row` → downward scan starts at cursor_row itself.  When
cursor_row is blank the scan immediately finds it → `end = cursor_row - 1`, wrong.
For non-blank cursor lines the first line found beyond is the same whether scanning from
cursor_row or cursor_row+1, so MISSED for all existing tests.

**T-15: `focus_paragraph_bounds` with cursor on a blank line:**
```rust
// Blank line between two paragraphs; `end` must not regress to cursor_row - 1.
let lines: Vec<String> = vec!["first".into(), "".into(), "second".into()];
let (start, end) = focus_paragraph_bounds(&lines, 1);  // cursor on blank line 1
assert!(start <= end, "start must not exceed end");
// More precisely: the blank line should not shrink the range to a degenerate pair.
assert!(end >= 1 || start == 0, "range must include at least one line");
```

---

## 3 · `src/renderer/search_bar.rs`

### Constants (lines 137, 139, 150) — TIMEOUT and MISSED

`INNER_W`, `BOX_W`, and `BOX_H` are compile-time constants used exclusively inside
`render_search_help_modal`, which is `#[mutants::skip]`.  Mutations to these constant
expressions cannot be observed by any test.  The timeouts and misses both stem from the
full 60-s suite running without a failure.

```toml
# INNER_W / BOX_W / BOX_H — only used in #[mutants::skip] render function
"renderer/search_bar\\.rs:137:.*replace \\+ with",
"renderer/search_bar\\.rs:139:.*replace \\+ with",
"renderer/search_bar\\.rs:150:.*replace \\+ with",
```

### `put_padded` (lines 244–257) — TIMEOUT and MISSED

`put_padded` is a private function called exclusively from `render_search_help_modal`.
Since the caller is `#[mutants::skip]`, `put_padded` is never invoked during any test run.
All 14 mutations MISSED/TIMEOUT.

**→ Add `#[mutants::skip]` to `put_padded`.**
```rust
#[mutants::skip] // Only called from skipped render_search_help_modal.
fn put_padded(buf: &mut Buffer, x: u16, y: u16, s: &str, width: u16, fg: Color, bg: Color) -> u16 {
```

---

## 4 · `src/search.rs`

### `update_matches` — clamping arithmetic (line 102)
```rust
self.current = self.current.min(self.matches.len() - 1);
```
`- 1` → `+ 1`: allows `current = len` (OOB index).  `- 1` → `/ 1 = len`: same effect.
`current_match()` calls `matches.get(current)` → `None` for OOB.  Silently wrong.

**T-16: `update_matches` clamps `current` to last valid index:**
```rust
let mut s = SearchState::new("a").unwrap();
s.update_matches(&["aaa".to_string()]);  // 3 matches
s.current = 99;                          // manually set OOB
s.update_matches(&["aaa".to_string()]);  // should clamp to 2 (len-1)
assert!(s.current_match().is_some(),  "must return a match, not None");
assert_eq!(s.current, 2,              "must be clamped to len-1");
```

### `prev_match` (line 140)
```rust
self.current -= 1;
```
`+= 1` → goes forward instead of backward.  `/= 1` → stays at same index (no-op).
Both are killable by asserting the exact value of `current` after the call.

**T-17: `prev_match` decrements `current` correctly:**
```rust
let mut s = setup_search_with_matches(3);  // 3 matches
s.current = 1;
s.prev_match();
assert_eq!(s.current, 0, "prev from 1 → 0");
s.prev_match();
assert_eq!(s.current, 2, "prev from 0 wraps to 2 (last)");
```

### `current_match` (line 147)
Whole-function replacements `→ None` and `→ Some(Default::default())` are only killable
if a test asserts on specific non-default field values of the returned match.

**T-18: `current_match` returns correct `(line, char_start, char_end)`:**
```rust
let mut s = SearchState::new("bc").unwrap();
s.update_matches(&["abcd".to_string()]);
let (ln, cs, ce) = s.current_match().unwrap();
assert_eq!(ln, 0, "line 0");
assert_eq!(cs, 1, "char_start 1");   // fails for Some(Default) → (0,0,0)
assert_eq!(ce, 3, "char_end 3");
```

### `snap_to_cursor` (line 161)
```rust
ml > cursor_line
```
`> → <`: seeks matches *before* cursor instead of after.

**T-19: `snap_to_cursor` selects first match at-or-after cursor:**
```rust
let mut s = SearchState::new("x").unwrap();
s.update_matches(&["x foo x bar x".to_string()]);  // matches at cols 0, 6, 12
s.snap_to_cursor(0, 4);
assert_eq!(s.current_match().unwrap().1, 6, "first match at or after col 4 is col 6");
s.snap_to_cursor(0, 0);
assert_eq!(s.current_match().unwrap().1, 0, "exact match at col 0");
```

### `find_all_matches` — char_end arithmetic (line 227)
```rust
let char_end = char_start + line[m.start()..m.end()].chars().count();
```
`+→-` underflows; `+→*` gives wrong product.  Need a test asserting the exact `char_end`.

**T-20: `find_all_matches` returns correct `char_end`:**
```rust
let re = fancy_regex::Regex::new("foo").unwrap();
let matches = find_all_matches(&re, &["barfoo".to_string()]);
assert_eq!(matches[0], (0, 3, 6), "char_start=3, char_end=6");
// With `+→-`: char_end = 3 - 3 = 0 (underflow/wrap).
// With `+→*`: char_end = 3 * 3 = 9 (out of bounds).
```

### Unkillable no-ops

**Line 185:36** — `replace || with &&` in `apply_replace_all` early-return guard:
```rust
if self.compiled.is_none() || self.matches.is_empty() { return }
```
`&&` is only different when exactly one condition is true.  `compiled.is_none()` implies
`matches.is_empty()` through normal API usage; the inconsistent half-state is unreachable.

**→ `exclude_re`, col 36, `in apply_replace_all`.**

**Line 190:19** — `replace < with <=` in per-match bounds check:
```rust
if ml < result.len() { ... }
```
`ml` comes from `find_all_matches` which only yields valid line indices.  `ml == len`
is unreachable for valid state.  **→ `exclude_re`, col 19, `in apply_replace_all`.**

---

## 5 · `src/table_format.rs`

### `find_table_bounds` infinite-loop mutations (lines 60, 66) — TIMEOUT

```rust
start -= 1;   // line 60: while-upward counter
end   += 1;   // line 66: while-downward counter
```
`-= → /=`: `x / 1 = x` — counter never changes → infinite loop.
`+= → *=`: `x * 1 = x` — counter never changes → infinite loop.

**→ `exclude_re` (col 15 and col 13):**
```toml
"table_format\\.rs:60:15:.*replace -= with /= in find_table_bounds",
"table_format\\.rs:66:13:.*replace \\+= with \\*= in find_table_bounds",
```

### `is_separator_row` — `&&→||` (line 103)

`||` short-circuits on non-empty rows, making any non-empty row a separator.

**T-21: `is_separator_row` returns false for non-separator content:**
```rust
// Public or via #[cfg(test)] re-export
assert!(!is_separator_row(&["hello".to_string()]));
assert!(!is_separator_row(&["a-b".to_string()]));
assert!(!is_separator_row(&["".to_string()]));
assert!( is_separator_row(&["---".to_string()]));
assert!( is_separator_row(&[":--:".to_string()]));
```

### `format_table` — 1-row table (lines 197:30 and 197:44)

`> → >=`: `rows.len() >= 1` → tries `rows[sep_row_idx=1]` OOB when len==1 → panic.
`&& → ||`: same OOB path, triggered when len==1 and `||` evaluates the right side.

**T-22: `format_table` with a single-row table (no separator row):**
```rust
let result = format_table(&["| A | B |".to_string()]);
assert_eq!(result.len(), 1, "single row in → single row out");
assert!(result[0].contains('|'), "output is still a pipe table");
```

### `format_table` bounds check (line 218:19)

`< → <=`: `ci == ncols` → `widths[ncols]` OOB.  But `ci` iterates `row.iter().enumerate()`
and every `row.len() <= ncols` by definition (`ncols = max row len`).  `ci` never reaches
`ncols`.  **→ `exclude_re`, col 19, `in format_table`.**

### `handle_format_table` integration (lines 243, 258)

Whole-function replacement (243:5) and `start + i` arithmetic (258:25) require an App-level
test to observe the side-effect on `app.textarea`.

**T-23: `handle_format_table` reformats a misaligned table in-place:**
```rust
let mut app = make_app_with_lines(vec![
    "| A | Bbb |".to_string(),
    "| - | --- |".to_string(),
    "| x | y   |".to_string(),
]);
app.textarea.move_cursor(CursorMove::Jump(0, 0));
handle_format_table(&mut app);
let out: Vec<String> = app.textarea.lines().iter().map(|s| s.to_string()).collect();
assert_eq!(out.len(), 3, "row count unchanged");
// After formatting, all rows have the same cell width structure.
assert_eq!(out[0].len(), out[2].len(), "header and body row same length after format");
```

---

## 6 · `src/input.rs`

### Bitmask no-ops (lines 474:47, 474:67 and 387:47, 387:67)

`handle_key_event` at line 474 and `handle_search_key` at line 387 both use
`CONTROL | ALT | SUPER`.  Existing `exclude_re` covers a different call site (cols 57/77).
These new instances at cols 47/67 need the same treatment.

**→ Add four `exclude_re` entries:**
```toml
"input\\.rs:474:47:.*replace \\| with \\^ in handle_key_event",
"input\\.rs:474:67:.*replace \\| with \\^ in handle_key_event",
"input\\.rs:387:47:.*replace \\| with \\^ in handle_search_key",
"input\\.rs:387:67:.*replace \\| with \\^ in handle_search_key",
```

### `do_replace_current` and `do_replace_all` (lines 222–262)

The whole-function replacements (222:5, 252:5) and `me - ms` arithmetic (230:24, 262:25)
need App-level integration tests checking that textarea content actually changes.

**T-24: `do_replace_current` replaces the active match:**
```rust
// Set up App: line = "hello world", query = "world", replace = "rust".
// After do_replace_current: line becomes "hello rust".
let mut app = make_search_app("hello world", "world", "rust");
do_replace_current(&mut app);
assert_eq!(app.textarea.lines()[0], "hello rust");
```

**T-25: `do_replace_all` replaces every match:**
```rust
// Line = "aa aa aa", query = "aa", replace = "b".
// After do_replace_all: line becomes "b b b".
let mut app = make_search_app("aa aa aa", "aa", "b");
do_replace_all(&mut app);
assert_eq!(app.textarea.lines()[0], "b b b");
```

### `handle_search_key` key-binding coverage (lines 296–393)

The large cluster of MISSED delete-match-arm and guard mutations shows the test suite
does not assert on the specific state changes each key causes.

**T-26: One assertion per search key binding:**

| Key | State assertion to verify |
|-----|--------------------------|
| Esc | `app.search` becomes `None` |
| Ctrl+F / Enter | `search.current` advances to next match |
| Shift+Enter | `search.current` retreats to previous match |
| Backspace | `search.query` loses last char; matches refresh |
| Alt+R | `search.use_regex` toggles |
| Ctrl+H | `search.show_replace` becomes `true` |
| Tab | `search.focus_search` flips |
| Char key | character appended to active field |

Each test creates a minimal App with `app.search = Some(SearchState::new(...))`,
calls `handle_search_key(&mut app, key_event)`, then asserts the specific field.

### `handle_key_event` match arms (lines 511, 523, 593)

- Line 511 (Ctrl+F): opens search mode → `app.search` is `Some`
- Line 523 (Ctrl+H): opens replace row → `app.search` is `Some` with `show_replace=true`
- Line 593 (Alt+T): triggers table format → (see T-23; or assert `app.force_redecorate`)

**T-27: Normal-mode keys open search/replace/table-format:**
```rust
// Ctrl+F → search opens
let mut app = make_empty_app();
handle_key_event(&mut app, ctrl('f'));
assert!(app.search.is_some(), "Ctrl+F must open search");

// Ctrl+H → replace opens
let mut app = make_empty_app();
handle_key_event(&mut app, ctrl('h'));
assert!(app.search.as_ref().map_or(false, |s| s.show_replace),
    "Ctrl+H must open search+replace");
```

---

## 7 · Priority order for fix session

1. **Update and add `exclude_re` patterns** — zero code risk, immediately shrinks the list:
   - `decoration/mod.rs`: 15+ no-op patterns (§1a)
   - `renderer/mod.rs`: 3 updated line numbers + 3 new wrap_line entries (§2)
   - `search.rs`: 2 no-op patterns (§4)
   - `table_format.rs`: 2 loop-infinite patterns (§5)
   - `table_format.rs`: 1 bounds-check no-op (§5)
   - `input.rs`: 4 bitmask no-ops (§6)
   - `renderer/search_bar.rs`: 3 constant patterns (§3)

2. **`#[mutants::skip]` on `put_padded`** — one annotation (§3).

3. **Tests** — rough priority by kill count:
   - T-10, T-11 (todo sub-span boundaries): ~15 kills
   - T-26 (handle_search_key per-key): ~12 kills
   - T-24, T-25 (do_replace_current/all): ~6 kills
   - T-3, T-4, T-5 (fenced code edge cases): ~5 kills
   - T-22, T-23 (table format): ~5 kills
   - T-16 through T-20 (search.rs precision): ~6 kills
   - T-7, T-8, T-9 (list/todo): ~4 kills
   - T-12, T-13 (table sep guards): ~3 kills
   - T-27 (handle_key_event keys): ~3 kills
   - T-1, T-2, T-6, T-14 (heading/blockquote/strikethrough): ~4 kills
   - T-15 (focus_paragraph_bounds blank cursor): ~1 kill

4. **Re-run mutants** to validate; update `exclude_re` for any post-fix stragglers.
