# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

### Added
- feat: Ctrl+I insert fenced code block (#186)
- feat: empty buffer mode — `yame` with no arguments opens an untitled buffer; Ctrl+S prompts for a filename (save-as); exit prompt also routes through save-as for untitled buffers (#152)

### Changed
- chore: migrate tui-textarea → tui-textarea-2, bump crossterm 0.28 → 0.29 (#184)
- refactor: split decoration/mod.rs into focused submodules (#181)
- chore: v0.2.1 version bump (#188)

### Fixed
- fix: click below last line should land at end-of-line not col 0 (#187)

## [0.2.1] — 2026-06-13

### Added

- **`Ctrl+K` code block insertion** — inserts a fenced code block (` ``` `\n` ``` `) with the cursor placed immediately after the opening ticks so the language tag can be typed without a separate cursor move

### Fixed

- Clicking below the last line of text now places the cursor at end-of-line rather than col 0
- Undo (`Ctrl+Z` / `Cmd+Z`) and redo (`Ctrl+Y` / `Cmd+Y` / `Cmd+Shift+Z`) were not functioning (#185)

## [0.2.0] — 2026-06-11

### Added

- **Search & find/replace** — `Ctrl+F` opens a search bar; `Enter` / `Shift+Enter` step through matches; `Alt+R` toggles regex mode; `Ctrl+H` switches to find-and-replace (`Tab` swaps fields, `Ctrl+A` replaces all)
- **Shortcut reference modal** — `F1` opens (and closes) a full in-app keybinding cheatsheet; also shown automatically on first search open
- **Line numbers** — add `line_numbers = true` under `[layout]` in config; cursor line uses accent color, others muted; gutter widens automatically at every order of magnitude
- **`==highlight==` inline decoration** — new inline span with configurable `highlight_bg` / `highlight_fg`; inner bold, italic, and links are preserved
- **YAML/TOML frontmatter styling** — auto-detected and rendered as a distinct block: delimiter lines muted, keys in code-green italic with a 3-space indent, values in body text, full-block background tinted from the code color
- **Focus mode** (`Ctrl+D`) — dims text outside the current paragraph; good for long documents
- **Typewriter mode** (`Ctrl+T`) — keeps the cursor line vertically centered in the viewport as you write
- **Read-only mode** — launch with `yame -r <file>` or toggle in-app with `Ctrl+E`; status bar changes color as a persistent reminder
- **Go-to-line** (`Ctrl+G`) — jump directly to any line number
- **`Alt+T` table reflow** — reformats the GFM pipe table the cursor is in to uniform column widths
- **`--version` / `-V` flag**

### Changed
- Release prep: CHANGELOG v0.2.0, version bump, gutter audit (#167)

- **Column width cap** — new `max_cols` setting under `[layout]` (e.g. `max_cols = 88`) limits the prose wrap column on wide terminals; previously the column expanded to 50 % of terminal width without a ceiling
- **Improved GFM table rendering** — better column alignment, padding, and border characters
- **Background decoration thread** — the syntax-highlighting and decoration pass now runs off the main event loop, keeping keystroke latency low on large files
- `[theme]` config gains `highlight_bg`, `highlight_fg`, `frontmatter_key`, `frontmatter_bg` overrides

### Fixed

- Blank-frame flash eliminated on file open
- Style bleed-through in modal overlays (search bar, shortcuts modal)
- Click-to-cursor off-by-one in soft-wrapped list items
- Bold/italic inside blockquotes no longer bleeds to the full line

## [0.1.0-alpha.1] - 2026-05-25

### Added
- Binary file detection on startup (#143)
- Plain-mode: syntect whole-file highlighting for non-markdown extensions (#129)
- feat: Windows support (Git Bash / Windows Terminal) (#125)
- feat: pull in two-face for extended syntax coverage (TOML, TypeScript, etc) (#135)
- richer fixture code + operator/punctuation palette colours (#132)
- Palette-derived highlight theme with per-token config overrides (#130)
- v1.5: CJK / wide character support (unicode-width) (#41)
- v1.5: syntect fenced code block syntax highlighting (#44)
- config: default to Nerd Font Powerline glyphs, add opt-out (#122)
- feat: yame write-config — write commented default config to XDG path (#117)
- feat: yame init — output shell wrapper function (eval-style) (#116)
- feat: -h/--help flag and no-args help text (#115)
- Todo items: indent continuation to text-start after full [ ] marker (#95)
- v1.5: blockquote continuation indent on soft-wrapped lines (#39)
- v1.5: tab character expansion on load (#40)
- v1.5: smart pair wrapping (wrap selection with bracket/quote) (#43)
- v1.5: Ctrl+R config reload (#42)
- Wrap terminal.draw() with synchronized output protocol to eliminate Enter/Backspace flicker (#78)
- Add no-args help output (suggest file path usage) (#88)
- Color tweaks: heading # toward bg, ~~ revert to muted, checkbox brackets muted, fenced lang tag muted (#80)
- Escape key exits (clean buffer) or prompts (dirty buffer), matching Ctrl+X (#66)
- Checked todo `[x]` bracket uses text color for visual pop; item text stays muted (#67)
- Heading delimiter blend, strikethrough delimiters, horizontal rule, H1–H3 bottom border (#48)
- v1 polish: italic startup warning, `delimiter_blend` config token, parent-dir creation on save (#35)

### Fixed
- Fix style bleed-through in modal overlays (shortcuts + search-help) (#159)
- startup: eliminate blank-frame flash on file open (#132)
- renderer: clip fenced/heading bg to content area, dim line-number colors (#131)
- FEEDBACK-2 2.3: cap highlight cache to prevent unbounded memory growth (#142)
- FEEDBACK-2 1.1: table header decoration swallows inline formatting (#141)
- FEEDBACK-2 batch 2: deduplicate selection, clipboard enum, path cache, mutants skip (#140)
- FEEDBACK-2 batch: exit-prompt modifier guard, tab-key spaces, empty-file save, centering fix, nav dirty-skip, highlight cache on reload (#139)
- fix: click-to-cursor off-by-one in wrapped list items (#137)
- fix: blank lines inside fenced code blocks lose fenced_bg (#133)
- Bug: syntect fg spans invisible — overlapping background span consumes char_pos (#131)
- Fix selection highlight clipping on wrapped indented list items (#128)
- fix cargo upgrade breakage: crossterm/ratatui API changes (#127)
- bug: bold/italic inside blockquote bleeds to full line (#120)
- fix flaky italic env-var tests (#119)
- investigate fenced code block beige color (#113)
- bug: indented list items wrap as if not indented — text clips edge (#111)
- Wide char (CJK) scroll redraw artifact — gap fills with stale content from above (#71)
- Free-scroll jitters and snaps back due to blanket free_scroll reset on every event (#99)
- Coloring of italic+bold non-adjacent nesting shows muted (#98)
- Three rendering regressions: scroll flicker, H1/H2 delimiter bold, bold+italic adjacency false-positive (#97)
- Fix nested bold+italic rendering (***text*** shows as bold only) (#50)
- Heading # delimiters not bold to match heading style (#91)
- Soft-wrap list items with continuation indent (#59)
- Soft-wrap space skip causes decoration spans to be off-by-1 char (#90)
- Fix decoration flash on Enter/Backspace: force immediate redecorate on structural keystrokes (#79)
- Cmd+C (forwarded as Ctrl+C) cuts/deletes selected text instead of copying (#73)
- Fix empty-file POSIX growth: saving empty buffer writes 1-byte newline (#84)
- Fix screen_to_doc missing upper-bound boundary checks on mouse click (#82)
- Fix exit-prompt Esc/Ctrl+C shadowed by outer key match arms (#81)
- Dim # heading and backtick/tilde delimiters using delimiter_blend like bold/italic/link delimiters (#75)
- Fenced code block background color wrong — blending toward code_color instead of staying dark (#72)
- Ctrl+Z undo not working (#70)
- Fix non-ASCII link text truncating closing bracket (#51)
- Fix mouse click to reposition cursor (gutter offset not subtracted) (#55)
- Scroll clamping now accounts for soft-wrapped visual rows, preventing cursor jumping off-screen (#63)
- Ghost scroll accumulation eliminated by intercepting scroll events before tui-textarea (#63)
- Italic default color now matches text color (not accent blend) (#60)
