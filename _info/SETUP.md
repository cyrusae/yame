# Setting Up yame

A complete walkthrough from a fresh machine to a fully configured editor.

---

## 1. Install Rust and Cargo

yame is installed via Cargo, the Rust package manager. Skip this step if you already have Rust.

### macOS / Linux

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts. Restart your terminal or run:

```sh
source "$HOME/.cargo/env"
```

### Windows

Download and run the installer from [rustup.rs](https://rustup.rs/). Or let `install.ps1` handle it — see [SCRIPTS.md](SCRIPTS.md).

---

## 2. Install yame

```sh
cargo install yame
```

Verify it worked:

```sh
yame --version
```

---

## 3. Generate a Config File (optional but recommended)

yame works without any config — it defaults to Catppuccin Mocha. To get a commented template you can edit:

```sh
yame write-config
```

This writes `~/.config/yame/config.toml` (respects `$XDG_CONFIG_HOME` if set; on Windows: `%APPDATA%\yame\config.toml`). The file explains every available option inline.

See [THEMES.md](THEMES.md) for switching to a built-in palette preset.

---

## 4. Font Setup (optional)

yame uses Nerd Font / Powerline glyphs for the status bar arrows and todo checkboxes. If your font doesn't include them, they'll appear as boxes.

**Two options:**
- Install a patched font — see [NERD-FONTS.md](NERD-FONTS.md) for download links and terminal setup guides.
- Disable glyphs by adding to your config:
  ```toml
  [layout]
  powerline_glyphs = false
  ```

---

## 5. Shell Integration (optional, recommended)

`yame init` prints a shell wrapper function that adds fuzzy-file discovery. Add it to your shell startup file:

```sh
# Zsh
echo 'eval "$(yame init)"' >> ~/.zshrc

# Bash
echo 'eval "$(yame init)"' >> ~/.bashrc
```

Then reload your shell (`exec zsh` / `exec bash` or open a new terminal).

**What this enables:**

| Command | Without integration | With integration |
|---------|--------------------|-|
| `yame` | Opens an untitled buffer | Fuzzy-finds Markdown files in the current directory |
| `yame notes` | Error (no file named "notes") | Fuzzy-searches for files matching "notes" |
| `yame path/to/file.md` | Opens that file | Same — direct paths pass through |

**Requires [`fd`](https://github.com/sharkdp/fd) and [`fzf`](https://github.com/junegunn/fzf).** See [FRIENDS.md](FRIENDS.md) for install instructions.

---

## 6. File Picker Integration (optional)

Once inside yame, `Ctrl+O` opens a file picker without leaving the editor. Picker resolution order:

1. `$YAME_PICKER` (if set) — run as a shell command; must print the selected path to stdout
2. `lf` — if installed, opens lf with `Enter` confirming the selection
3. `fzf` — if installed, fuzzy-finds from the current directory

If neither `lf` nor `fzf` is available, a status-bar message explains what to install.

See [FRIENDS.md](FRIENDS.md) for lf install and lfrc config.

---

## 7. Quick Reference

```
yame <file>           Open file for editing (created if it doesn't exist)
yame                  Open an untitled buffer (Ctrl+S prompts for a filename)
yame -r <file>        Open in read-only mode
yame --preview <file> Render to stdout with ANSI colour (for lf/file-manager previewers)
yame init             Print shell integration function
yame write-config     Write default config to ~/.config/yame/config.toml
yame --version        Print version
yame --help           Show help
```

Key bindings inside the editor — press `F1` for the full in-app reference.
