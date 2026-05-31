# Mutants Triage — 2026-05-30 Run

Generated from MUTANTS-0530.md.  Reference this when starting the fix session.

---

## 0 · Critical prerequisite: run flag

The comment in `mutants.toml` says to invoke as `cargo mutants --features mutants-skip`.
If this run was done **without** that flag, every `#[mutants::skip]` annotation in source
is inactive and those items will appear as MISSED or will TIMEOUT.  That single fact could
explain a large fraction of the list.  **Confirm the run command before touching anything.**

---

## 1 · Root cause of the TIMEOUT flood

`minimum_test_timeout = 20` with `timeout_multiplier = 2.0` means each mutant gets
`max(baseline × 2, 20)` seconds.  The test suite now has 522+ tests.  Two things happen:

**A — Genuine infinite loops** (most renderer/wrap/loop mutations): mutating a loop
counter (`+= → *=`, `+ → *` in a `while` condition) creates an infinite loop in the
*mutated binary*, so the test runner never exits.  These appear as TIMEOUT even when
we have good tests, because the test framework itself hangs waiting for a process that
never returns.  Raising the timeout does NOT help here; only `exclude_re` or `mutants::skip`
does.

**B — Suite-too-slow**: the 522-test suite is large enough that, for mutations where tests
don't fail immediately, the suite may genuinely run for > 20 s.  After raising the floor
(see §2) many of these will become CAUGHT or MISSED, giving cleaner data.

**Recommendation**: raise `minimum_test_timeout` from 20 → 60 in `mutants.toml`, re-run,
then re-triage.  Do not do the full skip/test work until you have a second run at 60 s.

---

## 2 · Quick wins before re-running

### 2a — mutants.toml: raise floor
```toml
minimum_test_timeout = 60   # was 20; suite has grown
```

### 2b — Already in `exclude_re` / already annotated?

Several MISSED items look like they should already be skipped or excluded:
- `handle_copy`, `handle_paste`, `ensure_clipboard` — previous session says these have
  `mutants::skip`; if they still appear → the `--features mutants-skip` flag was omitted.
- Same for `load_file`, `load_config`, `supports_italic`.

Verify before adding duplicate annotations.

---

## 3 · Missed mutants — action per file

### `src/main.rs` (9 MISSED)
**All → `mutants::skip`.**
- `setup_panic_hook`: sets `panic::set_hook`, void, process-level side-effect.
- `run`: launches crossterm terminal, runs the full TUI loop, returns `io::Result<()>`.
  No return value carries testable state; behaviour is I/O-only.
- `main`: calls `run`, exits process on error.  The `&&`/`!=` conditions guard argument
  parsing that already has test coverage via `parse_args`; but the `main` wrapper itself
  cannot be called from tests.

### `src/app.rs` (5 MISSED)
- `is_likely_binary` — **TESTS NEEDED.**  Pure fn: scans a `&[u8]` for null bytes.
  Three tests cover it: all-text bytes → false; slice containing `\0` → true;
  empty slice → false.  Very easy.
- `load_file` — **SKIP.**  Opens a real path with `fs::read`, branches on BOM/binary
  detection, constructs `TextArea`.  Filesystem I/O; no hermetic equivalent.

### `src/clipboard.rs` (4 MISSED)
**All → `mutants::skip`** (already should be; confirm flag issue from §0).
Requires a live display server.  `handle_copy`/`handle_paste`/`ensure_clipboard` all
call `arboard::Clipboard::new()` which fails in headless CI.

### `src/config.rs` (6 MISSED)
- `load_config` — **SKIP.**  Walks `~/.config`, reads TOML files from disk.
- `supports_italic` — **SKIP.**  Calls `terminfo` / reads `$TERM`; terminal-dependent,
  non-deterministic in CI.

### `src/decoration/mod.rs` (~100 MISSED, all in `build_decoration_map`)
This is the largest single gap.  `build_decoration_map` is a pure function that takes
`Vec<String>` and returns `DecorationMap`.  All mutations are in arithmetic on span
character positions and in boundary comparisons.

**Why so many?** The existing tests cover the *visual output* (what gets rendered) but
not the *exact character offsets* of every span.  Mutations that shift a `char_start`
by ±1 or swap `+` for `*` in position calculations only manifest as a test failure if a
test checks the precise span boundary.

**Action**: write targeted unit tests that assert on `(char_start, char_end)` of specific
spans in the output.  High-priority areas (by line cluster):
- Lines 327–351: inline code / blockquote span construction — add tests that check exact
  `char_start`/`char_end` of a code span inside a blockquote.
- Lines 422–455: list indent / hanging-indent math — test a nested list item and assert
  the `continuation_indent` value.
- Lines 616–640, 658–691: table cell spans — test a two-column table and assert exact
  cell boundary chars.
- Lines 726–736, 743–744: heading underline spans — assert `char_start = 0`.
- Lines 826–859: bold-italic span arithmetic — assert exact inner and outer char offsets.
- Lines 886–944: setext/ATX heading delimiter spans.
- Lines 966–971, 1001–1021: horizontal rule / table separator logic.

This is ~30–40 new tests.  Prioritise the table-cell and list-indent clusters as they
have the most missed mutants.

---

## 4 · Timeout mutants — per-function action

### `src/renderer/mod.rs`

**`wrap_line`** (lines 79–100, mix of MISSED and TIMEOUT):
- `+→*` and `+→-` in the main loop counter already have `exclude_re` entries for the
  structural no-ops.  The remaining TIMEOUTs (lines 81, 85, 94, 100) are genuine
  infinite-loop mutations (`==→!=` on the `while`, `+=→*=` on the advance, `+→*`/`+→-`
  on the break index).
- **Action**: add `exclude_re` entries for each with explanations, same style as the
  existing `wrap_line` entries.  Tests already exist; the mutations are unkillable because
  they create loops that never terminate.  No test can catch an infinite loop within
  the timeout.

**`<impl Widget for MarkdownView>::render`** (lines 344–586, all TIMEOUT):
**→ `mutants::skip`.**
This function writes into a ratatui `Buffer` (void, no return value).  Its mutations
either cause infinite loops (loop-counter mutations) or silently produce wrong pixels
with no way for tests to observe the difference without a full frame snapshot.  This is
the canonical "renders to framebuffer" case.

**`apply_search_overlay`** (lines 726–816, all TIMEOUT):
**→ `mutants::skip`.**
Same rationale as `render`: writes to `Buffer`, void, loop-based scan.  Every arithmetic
mutation is either an infinite loop (loop counter) or an off-by-one in cell coordinates
that requires a full-buffer snapshot test to catch.

**`focus_paragraph_bounds`** (lines 836–855, all TIMEOUT):
**DO NOT SKIP.**  This is a pure function and we have 8 unit tests.  The TIMEOUTs here
are *not* genuine infinite loops — the tests run and pass quickly.  These are **suite-too-slow**
timeouts: the overall 522-test suite takes > 20 s when those mutations don't cause an
early failure.  After raising `minimum_test_timeout` to 60 s (§2), these should all become
CAUGHT.  If any remain as MISSED after the re-run, add more targeted tests.

**`apply_focus_overlay`** (lines 870–907, all TIMEOUT):
**→ `mutants::skip`.**
Writes to `Buffer`, void.  Same reasoning as `apply_search_overlay`.

---

### `src/search.rs` (lines 305–351, all TIMEOUT)
All functions here (`update_matches`, `push_char`, `pop_char`, `next_match`, `prev_match`,
`current_match`, `snap_to_cursor`, `apply_replace_to_line`, `apply_replace_all`,
`escape_literal`, `find_all_matches`) already have test coverage from earlier sessions.
These are **suite-too-slow** timeouts.  After raising the floor to 60 s they should
all become CAUGHT.  No new action needed unless they appear as MISSED after the re-run.

---

### `src/status.rs` (lines 352–368, all TIMEOUT)
`StatusLine` methods (`set_timed`, `set_dismissible`, `dismiss`, `start_goto_line`,
`goto_push`, `goto_pop`, `goto_input`, `tick`, `message`) — all have test coverage.
**Suite-too-slow** timeouts.  Will become CAUGHT at 60 s.

---

### `src/table_format.rs` (lines 371–426, all TIMEOUT)
All table functions have tests.  **Suite-too-slow** timeouts.  Will become CAUGHT at 60 s.

---

### `src/cli.rs` (lines 427–456, all TIMEOUT)
- `shell_init_str` → **TESTS NEEDED.**  Returns a static shell-completion string.
  Pure.  Add one test: non-empty, contains expected keywords.
- `detect_shell` → **TESTS NEEDED.**  Reads `$SHELL` env var, returns "zsh"/"bash"/
  "unknown".  Test with `std::env::set_var("SHELL", "/bin/zsh")`.
- `print_help` → **SKIP.**  Prints to stdout.  Void.
- `version_string` → **TESTS NEEDED.**  Returns `env!("CARGO_PKG_VERSION")` + extras.
  Test: non-empty, starts with a digit.
- `parse_args` → **TESTS NEEDED.**  Takes `Vec<String>`, returns `Args`.  Tests for:
  no args → `Args::Open(None)`, `["file.md"]` → `Args::Open(Some("file.md"))`,
  `["init", "zsh"]` → `Args::Init("zsh")`, `["write-config"]` → `Args::WriteConfig`.
- `run_write_config` → **SKIP.**  Writes config to filesystem.

---

### `src/commands.rs` (lines 457–495, all TIMEOUT)
- `handle_save`, `handle_exit`, `clamp_scroll`, `center_scroll` → tests exist;
  **suite-too-slow** timeout.  BUT lines 108/110/111 have very long build times
  (113 s, 135 s, 143 s) — likely LLVM IR explosions for specific numeric mutations in
  a hot loop.  After 60 s floor re-run, check if these are now CAUGHT or still TIMEOUT;
  if still TIMEOUT, add targeted `exclude_re`.

---

### `src/input.rs` (lines 496–698, all TIMEOUT)

**`run_decorate`** → likely a thin wrapper; **suite-too-slow** timeout.

**`screen_to_doc`** (114 lines of TIMEOUT) → pure-ish coordinate math.
Tests exist from earlier sessions.  **Suite-too-slow** timeouts.  The very long
build times on some (43 s–64 s) suggest compiler explosion on those specific mutations.
After re-run at 60 s, check what's left; add `exclude_re` for any structural no-ops.

**`is_navigation_key`** → **suite-too-slow** (50 s–59 s build!); all tests exist.

**`handle_pair_wrap`** → tests exist; already has one `exclude_re` for the `|→^` no-op.
Remaining TIMEOUTs are genuine: deleting a match arm means that pair-wrap doesn't
insert the closing bracket.  Tests should catch these at 60 s.

**`do_replace_current`, `do_replace_all`** → tests exist; **suite-too-slow**.

**`handle_search_key`, `handle_goto_line_key`, `handle_key_event`, `handle_visual_move`**:
All have test coverage.  **Suite-too-slow** at 20 s floor.  Will become CAUGHT at 60 s.
Exception: the `|→^` bitmask mutations at `input.rs:474` already have `exclude_re`
entries; the `input.rs:387` ones may need similar entries (CONTROL|ALT|SUPER with
non-overlapping bits → `|` and `^` produce the same bitmask).

**`event_loop`** (lines 651–698) → **SKIP.**
The main crossterm event loop: blocks on `crossterm::event::read()`.  Any mutation
that doesn't crash the loop immediately will cause the test harness to hang forever
waiting for terminal events.  This is a true I/O boundary; no unit test can exercise
the live event path.

---

### `src/decoration/spans.rs` (lines 699–735, all TIMEOUT)
`line_start_bytes`, `byte_to_line_char`, `line_char_len`, `push_span`, `make_span`,
`add_byte_range_span` — all tests exist from earlier sessions.  **Suite-too-slow.**
Exception: `line_char_len` mutations (lines 712–718) have 29 s–49 s build times,
suggesting LLVM expansion; check after re-run.

---

### `src/decoration/words.rs` (lines 737–761, all TIMEOUT)
`count_words`, `link_split_char_idx`, `count_chars_in` — tests exist; `i += 1 → *= 1`
already has `exclude_re`.  The remaining TIMEOUTs (especially the 126 s build for one
`link_split_char_idx` mutation) are genuine infinite loops from broken loop advances.
After 60 s re-run, any that aren't caught should get structural `exclude_re` entries.

---

### `src/renderer/status.rs` (lines 763–777, all TIMEOUT)
- `pill1_parts` → returns `(Span, Color)`; **TESTS NEEDED.**  Pass an `App` with
  `is_dirty=false` → check bg is `theme.text`; with `is_dirty=true` → check bg is
  `theme.accent`.
- `render_status_bar`, `render_info_line` → **SKIP.**  Write to `Frame`; void.
- `build_timed_message_bar`, `build_normal_status_bar`, `build_goto_line_bar` →
  **TESTS NEEDED.**  All return `Line<'static>`.  Tests: call each, assert `spans.len()`,
  check first/last span's style fg/bg.  Medium complexity, high kill rate for
  status bar mutations.

---

### `src/renderer/utils.rs` (lines 778–799, all TIMEOUT)
`shorten_path`, `format_thousands`, `split_into_spans` — all tests exist.
**Suite-too-slow.**  After re-run at 60 s should become CAUGHT.

---

### `src/renderer/search_bar.rs` (lines 800–927, all TIMEOUT)
- `search_bar_height` → returns `u16`; tests exist; suite-too-slow.
- `render_search_bar`, `render_search_help_modal` → **SKIP.**  Write to `Frame`; void.

---

## 5 · Priority order for fix session

When the mutants fix session starts:

1. **Confirm run flag** (§0) — if `--features mutants-skip` was missing, re-run first.
2. **Raise timeout floor** (`mutants.toml`: 20 → 60) — immediately eliminates noise.
3. **Re-run mutants** — get clean second data set.
4. **Add `mutants::skip`** to functions identified in §3 / §4:
   - `main.rs`: `setup_panic_hook`, `run`, `main`
   - `app.rs`: `load_file`
   - `input.rs`: `event_loop`
   - `renderer/mod.rs`: `render` (impl Widget), `apply_search_overlay`, `apply_focus_overlay`
   - `renderer/status.rs`: `render_status_bar`, `render_info_line`
   - `renderer/search_bar.rs`: `render_search_bar`, `render_search_help_modal`
   - `cli.rs`: `print_help`, `run_write_config`
   (clipboard.rs / config.rs should already be annotated — verify first)
5. **Write tests** in priority order:
   a. `is_likely_binary` — 3 tests, trivial
   b. `pill1_parts`, `build_*_bar` in status.rs — ~10 tests, medium
   c. `cli.rs` pure functions — ~8 tests, medium
   d. `build_decoration_map` targeted span tests — ~30 tests, high effort, high impact
6. **Add `exclude_re`** for remaining structural no-ops found after re-run.

---

## 6 · New `exclude_re` patterns identified (for structural no-ops)

Add to `mutants.toml` after confirming they are truly unkillable:

```toml
# `wrap_line` remaining infinite-loop mutations not yet excluded:
# renderer/mod.rs:81 ==→!= (pathological guard inversion → infinite loop)  -- already in exclude_re
# renderer/mod.rs:85 ==→!= (while condition)
# renderer/mod.rs:94 +=→*= (loop advance → stays at same position forever)
# renderer/mod.rs:100 +→- / +→* (next_start goes backward → re-processes same chunk)

# `handle_search_key` CONTROL|ALT|SUPER non-overlapping bitmask no-ops —
# same pattern as existing input.rs cols 57/77 entries.
# Check exact col numbers after re-run before committing.
```
