# yame

**Yet Another Markdown Editor** — a lightweight terminal editor for Markdown files.
The goal is something meaningfully lighter than VS Code for editing READMEs, notes,
and CLAUDE.mds, while being more capable than nano for Markdown specifically.

> **Early / alpha.** The core editing loop and theming system are solid. The visual
> layout is going to change — don't treat the current UI as settled. Config keys and
> keybindings are stable.

<!-- screenshot or demo GIF goes here once the UI settles -->

---

## What it does

- Opens and saves Markdown files with inline decoration: headings, bold, italic, inline
  code, fenced blocks, blockquotes, links, lists, todo checkboxes, tables
- Centered editing column with soft word wrap
- Catppuccin Mocha theme by default, fully configurable via `~/.config/yame/config.toml`
- System clipboard (`Ctrl+C` / `Ctrl+V`)
- Save with `Ctrl+S`, exit with `Ctrl+X` (prompts if unsaved changes)
- Undo/redo via `Ctrl+Z` / `Ctrl+Y`

## What it doesn't do yet

- No syntax highlighting inside fenced code blocks (coming in v1.5)
- No search (coming in v2)
- No line numbers (coming in v2)
- No tab completion, file browser, or split panes

---

## Install

Requires Rust (stable). From source:

```sh
git clone https://github.com/yourname/yame
cd yame
cargo install --path .
```

> **Note:** Requires a [Nerd Fonts](https://www.nerdfonts.com/) patched terminal font
> for the Powerline separator in the status bar. If you don't have one, the separator
> will render as a box character — everything else works fine.

### Platform support

Tested on macOS. Should work on Linux; untested. Windows is not supported.

---

## Usage

```
yame <file>
```

Opens `<file>` for editing. If the file doesn't exist, an empty buffer is created and
saved to that path on `Ctrl+S`. Parent directories are created automatically.

### Shell wrapper (optional)

The binary takes only an explicit path. If you want fuzzy file discovery, add this
function to your `.bashrc` or `.zshrc`:

```bash
yame() {
  local target
  if [[ -z "$1" ]]; then
    # No argument: fuzzy find Markdown files in current directory
    target=$(fd --type f --extension md | fzf --select-1 --exit-0 --preview 'head -20 {}')
  elif [[ "$1" == */* || "$1" == *.* ]]; then
    # Looks like an explicit path: pass through directly
    target="$1"
  else
    # Treat as a fuzzy search term
    target=$(fd --type f "$1" | fzf --select-1 --exit-0 --preview 'head -20 {}')
  fi
  [[ -n "$target" ]] && command yame "$target"
}
```

Requires [`fd`](https://github.com/sharkdp/fd) and [`fzf`](https://github.com/junegunn/fzf).
Without `fd`, replace `fd --type f --extension md` with `find . -name "*.md"`.

> This is an example pattern, not a battle-tested wrapper. Adjust to taste.

---

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Ctrl+X` | Exit (prompts if unsaved changes) |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy selection (or current line if no selection) |
| `Ctrl+V` | Paste from system clipboard |
| Arrow keys | Move cursor |
| `Shift+Arrow` | Select text |
| `Home` / `End` | Start / end of line |
| `PgUp` / `PgDn` | Scroll |
| Mouse click | Place cursor |
| Mouse scroll | Scroll |

---

## Configuration

Config file: `~/.config/yame/config.toml`
(Respects `$XDG_CONFIG_HOME` if set.)

The file is optional. All values below are the defaults (Catppuccin Mocha). You can
set any subset — missing keys fall back to the default.

### Base palette

```toml
[palette]
text    = "#cdd6f4"   # body text
accent  = "#cba6f7"   # headings, links, bullets
muted   = "#585b70"   # blockquotes, URLs, completed todos
code    = "#a6e3a1"   # inline code and fenced blocks
bg      = "#1e1e2e"   # editor background
warning = "#f38ba8"   # dirty flag, warnings
```

Setting these six colors gives you a coherent theme. All other colors derive from
them automatically.

### Theme overrides

Optional per-element overrides. These take precedence over the derived defaults.

```toml
[theme]
# bold_color       = "#cdd6f4"
# italic_color     = "#f5c2e7"   # e.g. Catppuccin pink for tonal distinction
# blockquote_color = "#6c7086"
# link_text_color  = "#cba6f7"
# link_url_color   = "#6c7086"
# todo_done        = "#585b70"   # color for completed todo items
# code_bg          = "#262637"
# fenced_bg        = "#222233"
# heading_bg       = "#302d45"
# selection_bg     = "#413d5c"
# selection_fg     = "#1e1e2e"
# ui_bg            = "#1e1e2e"
# ui_bar           = "#313244"
# ui_text          = "#cdd6f4"
# delimiter_blend  = 0.4         # 0.0 = full muted, 1.0 = full span color
```

### Per-level heading colors

```toml
[headings]
# h1 = "#cba6f7"
# h2 = "#89b4fa"
# h3 = "#94e2d5"
# h4 = "#a6e3a1"
# h5 = "#f5c2e7"
# h6 = "#fab387"
```

### Layout

```toml
[layout]
# min_cols = 60   # minimum editing column width in characters
```

### Error handling

If the config file has invalid TOML, yame falls back to defaults and prints a warning
to stderr. If an individual color value is malformed, that field falls back to its
default and a dismissible warning banner appears at the top of the editor.

---

## License

MIT
