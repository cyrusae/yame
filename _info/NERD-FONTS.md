# Nerd Fonts

`yame` uses custom status bar symbols, Todo checkboxes, and formatting indicators. To render these characters correctly, your terminal needs a patched Nerd Font.

---

## 1. Choose and Install a Font

1. Visit the [Nerd Fonts Downloads page](https://www.nerdfonts.com/font-downloads).
2. Download a popular monospace font such as **JetBrains Mono**, **Fira Code**, or **Cascadia Code** (Caskaydia Cove).
3. Extract the downloaded `.zip` file.
4. Install the font on your system:
   * **macOS:** Open the extracted folder, select the `.ttf` or `.otf` files, double-click, and click **Install Font** (or drag them into Font Book).
   * **Linux:** Copy the font files to `~/.local/share/fonts/` (or `/usr/share/fonts/` for system-wide) and run `fc-cache -fv`.
   * **Windows:** Right-click the `.ttf` or `.otf` files and select **Install** or **Install for all users**.

---

## 2. Configure Your Terminal Emulator

### macOS Terminal.app
1. Open Terminal.
2. Open Settings via `Terminal` -> `Settings` (or `Preferences` / `Cmd + ,`).
3. Select **Profiles** and go to the **Text** tab.
4. Under **Font**, click **Change...** and select your installed font (look for "Nerd Font" in the name).

### iTerm2
1. Open iTerm2.
2. Open Settings via `iTerm2` -> `Settings` (or `Cmd + ,`).
3. Select **Profiles** -> **Text**.
4. In the **Font** dropdown, select your installed Nerd Font.

### Alacritty
Open your configuration file (`~/.config/alacritty/alacritty.toml`) and define the font family:
```toml
[font]
normal = { family = "JetBrainsMono Nerd Font", style = "Regular" }
size = 12.0
```

### Kitty
Open your configuration file (`~/.config/kitty/kitty.conf`) and set the font family:
```conf
font_family JetBrainsMono Nerd Font
font_size 12.0
```

### Windows Terminal
1. Open Settings (`Ctrl + ,`).
2. Select your default profile (or the specific shell profile you use) under **Profiles**.
3. Select **Appearance**.
4. Change **Font face** to your installed Nerd Font.
5. Save your settings.

---

## 3. Fallback (No Font Installation)

If you do not want to install a Nerd Font, you can configure `yame` to use standard ASCII status line separators:

1. Open `~/.config/yame/config.toml` (run `yame write-config` to generate it if it does not exist).
2. Set `powerline_glyphs` to `false` under the `[layout]` section:
   ```toml
   [layout]
   powerline_glyphs = false
   ```
3. Save the config. `yame` will automatically reload the changes.
