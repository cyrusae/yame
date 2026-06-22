# Themes and Colors

yame ships with fourteen named palette presets and a full per-element override system. All of this lives in `~/.config/yame/config.toml` — run `yame write-config` to generate a commented template.

---

## Named Presets

Set a preset under `[palette]`:

```toml
[palette]
preset = "dracula"
```

### Available presets

| Preset name | Also accepts |
|-------------|-------------|
| `catppuccin-mocha` | `mocha` |
| `catppuccin-macchiato` | `macchiato` |
| `catppuccin-frappe` | `frappe`, `catppuccin-frappé`, `frappé` |
| `catppuccin-latte` | `latte` |
| `dracula` | |
| `nord` | |
| `gruvbox-dark` | `gruvbox` |
| `solarized-dark` | `solarized` |
| `solarized-light` | |
| `tokyo-night` | `tokyonight` |
| `rose-pine` | `rosepine` |
| `rose-pine-moon` | `rosepine-moon` |
| `rose-pine-dawn` | `rosepine-dawn` |
| `github-light` | `github` |

If no preset is set, yame defaults to **Catppuccin Mocha**.

---

## Expanded Variants

Append `-expanded` to any preset name to enable **rainbow heading colors** and a **per-theme italic tint**:

```toml
[palette]
preset = "dracula-expanded"
```

Expanded variants assign distinct colors to H1–H6 sourced from each theme's own palette (Dracula's cyan, green, yellow, etc. rather than all headings inheriting the accent color). The italic color is also set to a complementary tint chosen for that theme.

Every preset has an `-expanded` variant: `catppuccin-mocha-expanded`, `nord-expanded`, `rose-pine-expanded`, `github-light-expanded`, and so on.

---

## Overriding Individual Colors

Individual fields always override the preset. You can mix a preset with hand-tuned values:

```toml
[palette]
preset = "nord"
accent = "#88c0d0"   # override just the accent; everything else stays Nord
```

### Base palette fields

```toml
[palette]
# preset  = "catppuccin-mocha"
# text    = "#cdd6f4"   # body text
# accent  = "#cba6f7"   # headings, links, bullets
# muted   = "#585b70"   # blockquotes, URLs, completed todos
# code    = "#a6e3a1"   # inline code and fenced blocks
# bg      = "#11111b"   # editor background
# warning = "#f38ba8"   # dirty flag, exit prompt, error messages
```

### Per-element overrides

Fine-grained control over specific UI elements. These take precedence over both preset and derived defaults:

```toml
[theme]
# bold_color          = "#cdd6f4"
# italic_color        = "#f5c2e7"
# strikethrough_color = "#585b70"
# blockquote_color    = "#6c7086"
# link_text_color     = "#cba6f7"
# link_url_color      = "#6c7086"
# todo_done           = "#585b70"
# rule_color          = "#585b70"
# code_bg             = "#262637"
# fenced_bg           = "#222233"
# heading_bg          = "#302d45"
# selection_bg        = "#413d5c"
# selection_fg        = "#1e1e2e"
# ui_bg               = "#1e1e2e"
# ui_bar              = "#313244"
# ui_text             = "#cdd6f4"
# delimiter_blend     = 0.4         # 0.0 = full muted, 1.0 = full span color
# highlight_bg        = "#524568"   # ==text== background
# highlight_fg        = "#cdd6f4"   # ==text== foreground
# frontmatter_key     = "#a6e3a1"   # YAML/TOML frontmatter key color
# frontmatter_bg      = "#1e2620"   # YAML/TOML frontmatter block background
```

### Per-heading colors

Override individual heading levels (H1–H6). These win over both expanded-preset rainbow colors and derived accent blends:

```toml
[headings]
# h1 = "#cba6f7"
# h2 = "#89b4fa"
# h3 = "#94e2d5"
# h4 = "#a6e3a1"
# h5 = "#f5c2e7"
# h6 = "#fab387"
```

---

## Resolution Order

When computing the final color for any element, yame applies values in this order — last set wins:

1. Built-in Mocha defaults
2. Named preset base colors (`[palette] preset`)
3. Expanded preset overrides (rainbow headings, italic tint) if `-expanded` suffix is used
4. Individual `[palette]` fields (`text`, `accent`, `muted`, `code`, `bg`, `warning`)
5. `[headings]` per-level colors
6. `[theme]` per-element overrides

---

## Reloading

Config changes take effect immediately — press `Ctrl+R` inside yame to reload without restarting.
