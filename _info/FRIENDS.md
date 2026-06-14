# Friends of yame

yame works on its own, but several terminal tools expand what it can do. None of these are required.

---

## fd and fzf — Fuzzy File Finding

**What they add:** The `yame init` shell wrapper (see [SETUP.md](SETUP.md)) uses `fd` and `fzf` so that `yame <term>` fuzzy-searches by filename and `yame` (no argument) fuzzy-finds Markdown files in the current directory.

Without the shell wrapper, `yame` with no argument opens an untitled buffer instead.

### Install

#### macOS (Homebrew)

```sh
brew install fd fzf
```

#### Linux (Debian / Ubuntu)

```sh
sudo apt install fd-find fzf
```

> **Note:** Debian-based systems install `fd` as `fdfind`. The `yame init` shell wrapper handles this automatically — you don't need to alias it yourself.

#### Linux (Arch)

```sh
sudo pacman -S fd fzf
```

#### Windows (Winget)

```powershell
winget install sharkdp.fd
winget install junegunn.fzf
```

---

## lf — File Manager + Ctrl+O Picker

**What it adds:** `lf` is a fast terminal file manager. yame integrates with it in two ways:

1. **`Ctrl+O` inside yame** — if lf is installed, pressing `Ctrl+O` suspends the editor, opens lf, and loads the file you select. No config needed; it just works.
2. **`yame --preview` in lf** — yame serves as lf's file previewer for all file types: Markdown gets full live decoration, and everything else gets syntect syntax highlighting (150+ languages). No separate previewer tool needed.

### Install

#### macOS (Homebrew)

```sh
brew install lf
```

#### Linux (Debian / Ubuntu)

```sh
sudo apt install lf
```

#### Linux (Arch)

```sh
sudo pacman -S lf
```

#### Windows (Winget)

```powershell
winget install gokcehan.lf
```

### Configure lf to use yame

Add to `~/.config/lf/lfrc`:

```sh
# Open files with yame
cmd open $yame "$f"

# Use yame --preview for all files
set previewer ~/.config/lf/preview
```

Create `~/.config/lf/preview` (make it executable with `chmod +x`):

```sh
#!/usr/bin/env bash
COLUMNS="$2" yame --preview "$1"
```

> `$2` passes lf's preview pane width to yame so it wraps at the right column.

### Custom picker via $YAME_PICKER

If you want `Ctrl+O` to use something other than lf or fzf, set `YAME_PICKER` to a shell command that prints a file path to stdout:

```sh
# In your .zshrc / .bashrc
export YAME_PICKER='fzf --preview "head -20 {}"'
```

The command runs in a subshell (`sh -c`). Whatever it prints on the first non-empty line becomes the path yame opens.

