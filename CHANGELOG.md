# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Escape key exits (clean buffer) or prompts (dirty buffer), matching Ctrl+X (#66)
- Checked todo `[x]` bracket uses text color for visual pop; item text stays muted (#67)
- Heading delimiter blend, strikethrough delimiters, horizontal rule, H1–H3 bottom border (#48)
- v1 polish: italic startup warning, `delimiter_blend` config token, parent-dir creation on save (#35)

### Fixed
- Fenced code block background color wrong — blending toward code_color instead of staying dark (#72)
- Ctrl+Z undo not working (#70)
- Fix non-ASCII link text truncating closing bracket (#51)
- Fix mouse click to reposition cursor (gutter offset not subtracted) (#55)
- Scroll clamping now accounts for soft-wrapped visual rows, preventing cursor jumping off-screen (#63)
- Ghost scroll accumulation eliminated by intercepting scroll events before tui-textarea (#63)
- Italic default color now matches text color (not accent blend) (#60)

### Changed
- Annotate build_decoration_map with mutants::skip (#69)
- Document yame init shell helper design notes and review (#68)
- Color fenced code language tag accent, backtick fences inline-code color (#57)
- Color scheme: Catppuccin Crust `#11111b` main bg; gutter and editor column unified (#49)
- Status bar redesigned as floating Powerline pills on canvas background (#49, #52, #53)
- Filename pill turns accent color when buffer is dirty (#58)
- UI chrome bg dynamically blends toward text color (`blend(text, bg, 0.10)`) (#52)
- Formatting now persists on the cursor line (decoration stripping removed) (#61)
- Virtual bottom padding added so last document lines don't sit flush against the UI bar (#62)
- `cargo deny`: suppressed RUSTSEC-2024-0436 (arboard transitive dep, not actionable) (#64)
- BSL-1.0 (Boost Software License) added to `cargo deny` allowlist for arboard transitive deps (#64)
- Phase 12: README & Distribution (#13)
- chore: remove scrollbar widget (#34)
- fix: initial decoration pass, gutter, todo muted, info line width (#33)
- fix: POSIX trailing newline, navigation dirty flag, mouse coordinate offset (#32)
- Phase 11: Integration Test & Coverage Audit (#12)
- Phase 10: Polish & Edge Cases (#11)
- Phase 9: File Operations & Edit Behaviors (#10)
- Phase 8: Debounce Loop (#9)
- Phase 7: Custom Renderer (#8)
- Wire MarkdownView into event loop draw call (#30)
- Selection overlay (full fg+bg override) (#29)
- Cursor rendering (#28)
- MarkdownView widget struct and Widget impl stub (#25)
- wrap_line() with word/hard-break and blockquote continuation (#27)
- span_split_into_spans() with multi-byte safety (#26)
- CI tooling: cargo-mutants, cargo-deny, cargo-nextest (#31)
- Word count (count_words) (#24)
- Cursor line exclusion (#23)
- GFM tables (with v3 TODO seam) (#22)
- Todo items (checked/unchecked) (#21)
- Lists and bullet/number accent color (#20)
- Links — split at ]( boundary, text/url styling (#19)
- Blockquotes with ▌ indicator and continuation indent flag (#18)
- Fenced code blocks (with v1.5 TODO seam) (#17)
- Inline code spans (#16)
- Bold and italic spans with delimiter blending (#15)
- Heading decoration (H1-H6 with heading_bg) (#14)
- Phase 6: Decoration Engine (#7)
- Phase 5: Status Bar & Info Line (#6)
- Phase 3: App State & tui-textarea Integration (#4)
- Phase 4: Layout Engine (#5)
- Phase 2: Config & Theming (#3)
- Phase 1: Terminal Lifecycle (#2)
- Phase 0: Project Scaffold (#1)
