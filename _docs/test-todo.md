 ---
  1. --preview (easiest, no TUI)
  
  ./target/release/yame --preview README.md | less -R
  
  less -R renders the ANSI codes. You should see headings in purple/bold, bold markers dimmed, inline code in green, etc. — the same colour
  scheme as the editor, but piped to stdout. Try it on a Rust file too:

  ./target/release/yame --preview src/main.rs | less -R
  
  Should get full syntect highlighting (Rust keywords, strings, comments).

  ---
  2. Empty buffer mode

  ./target/release/yame

  Status bar should show (untitled) instead of a filename. Type a few words. Then:
  - Ctrl+S → status bar switches to Save as: _ — type a path like test-untitled.md, Enter → file gets saved and path appears in the status bar
  - Ctrl+X on a dirty untitled buffer → exit prompt → Y → same save-as flow, then exits → N → discards and exits
  
  ---
  3. Ctrl+O file picker

  Open any file first, then inside the editor press Ctrl+O:
  - If buffer is clean → lf launches immediately (it's the auto-detected default)
  - Navigate to a file in lf, press Enter or l to select → lf exits → new file loads in the editor
  
  If you want to test the dirty-buffer guard: make a change (don't save), then press Ctrl+O → status bar shows the yellow "Unsaved changes will
   be lost — switch file?" prompt → Y to proceed, N to cancel.

  To test fzf instead of lf:
  YAME_PICKER=fzf ./target/release/yame README.md

  ---
  4. Quick lf previewer config (optional)

  If you want to wire it up in lf right now, add to ~/.config/lf/lfrc:
  set previewer yame --preview
  
  Then launch lf and navigate to any .md or .rs file — the right pane should show it rendered with colours.
  
  ---
  The one thing to watch for: after Ctrl+O returns from lf, the screen should fully repaint (we call terminal.clear() after resuming). If you
  see any visual artifacts from lf not fully cleaning up, let me know.