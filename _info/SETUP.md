# Setting Up `yame`

This guide walks you through installing `yame` and its optional additions.

---

## 1. Install Rust and Cargo

`yame` is installed via Cargo, the Rust package manager. If you do not have Rust installed:

### macOS and Linux
Run the following command in your terminal:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Follow the on-screen prompts. Once completed, restart your terminal or run:
```sh
source "$HOME/.cargo/env"
```

### Windows
Download and run the installer from:
[rustup.rs](https://rustup.rs/)

---

## 2. Install `yame`

Run the following command to download, build, and install the latest version:
```sh
cargo install yame
```

Verify the installation succeeded by checking the version:
```sh
yame --version
```

---

## 3. Recommended Font Configuration

By default, `yame` uses custom status bar arrow separators (Powerline glyphs) and checklist icons. If your terminal font does not support these, they will render as broken boxes.

To fix this:
* Install a Nerd Font. Read [NERD-FONTS.md](file:///Users/watcher/githere/yame/_info/NERD-FONTS.md) for download links and configuration guides.
* Or, disable them by adding `powerline_glyphs = false` to your `~/.config/yame/config.toml` file.

---

## 4. Optional Enhancements

You can expand `yame` to support fuzzy file opening and previewing.
* For integrations with `fzf`, `fd`, `lf`, and `bat`, read [FRIENDS.md](file:///Users/watcher/githere/yame/_info/FRIENDS.md).
* For a hands-off, automated setup of all tools, read [SCRIPTS.md](file:///Users/watcher/githere/yame/_info/SCRIPTS.md).
