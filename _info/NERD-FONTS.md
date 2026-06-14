# Nerd Fonts

yame uses Powerline glyphs for status bar separators and Nerd Font icons for todo checkboxes and mode indicators. Your terminal needs a patched font to render them correctly — otherwise they appear as boxes or question marks.

If you'd rather skip fonts entirely, see the **Fallback** section at the bottom.

---

## 1. Choose and Download a Font

Visit [nerdfonts.com/font-downloads](https://www.nerdfonts.com/font-downloads) and pick a monospace font. Popular choices:

- **JetBrains Mono** — clean, designed for code, widely supported
- **Fira Code** — ligature support, very readable
- **Cascadia Code** (Caskaydia Cove NF) — Microsoft's font; also has a built-in Powerline variant that doesn't need patching
- **Hack** — neutral, no-frills

Download the `.zip` for your chosen font and extract it.

---

## 2. Install the Font

### macOS

Open the extracted folder, select all `.ttf` or `.otf` files, double-click any one, and click **Install Font**. Or drag them into **Font Book**.

### Linux

```sh
mkdir -p ~/.local/share/fonts
cp *.ttf ~/.local/share/fonts/    # or *.otf
fc-cache -fv
```

For system-wide install, copy to `/usr/share/fonts/` instead (requires sudo).

### Windows

Right-click the `.ttf` or `.otf` files and select **Install** (current user) or **Install for all users**.

---

## 3. Configure Your Terminal

### macOS Terminal.app

`Terminal → Settings → Profiles → Text → Font → Change…` — select your Nerd Font.

### iTerm2

`iTerm2 → Settings → Profiles → Text → Font` — select your Nerd Font.

### Ghostty

In `~/.config/ghostty/config`:

```
font-family = "JetBrainsMono Nerd Font"
```

### Alacritty

In `~/.config/alacritty/alacritty.toml`:

```toml
[font]
normal = { family = "JetBrainsMono Nerd Font", style = "Regular" }
size = 12.0
```

### Kitty

In `~/.config/kitty/kitty.conf`:

```
font_family JetBrainsMono Nerd Font
font_size 12.0
```

### WezTerm

In `~/.config/wezterm/wezterm.lua`:

```lua
config.font = wezterm.font("JetBrainsMono Nerd Font")
```

### Windows Terminal

`Settings (Ctrl+,) → Profiles → [your profile] → Appearance → Font face` — type your Nerd Font name and save.

---

## 4. Fallback: Disable Glyphs

If you don't want to change your font, disable the Powerline separators and Nerd Font icons:

```sh
yame write-config   # create config if it doesn't exist yet
```

Then open `~/.config/yame/config.toml` and set:

```toml
[layout]
powerline_glyphs = false
```

Save and press `Ctrl+R` in yame to reload. The status bar will use `│` instead of the arrow separators, and mode/checkbox icons will use ASCII fallbacks.
