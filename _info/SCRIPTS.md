# Automation Scripts

For a hands-off setup, the installer handles Rust, yame, and optional dependencies in one go.

| Platform | Script |
|----------|--------|
| macOS / Linux | `_info/install.sh` |
| Windows (PowerShell) | `_info/install.ps1` |

---

## install.sh (macOS / Linux)

```sh
bash _info/install.sh
```

### What it does

1. **OS detection** — identifies macOS or Linux (Debian/Ubuntu, Arch, Fedora/RHEL)
2. **Rust toolchain** — checks for Cargo; installs via `rustup` if missing
3. **Optional dependencies** — offers to install `fzf`, `fd`, and `lf` using the detected package manager
4. **yame** — runs `cargo install yame`
5. **Shell integration** — offers to add `eval "$(yame init)"` to your `.zshrc` or `.bashrc`
6. **Config template** — offers to run `yame write-config`

### Platform notes

- **Debian/Ubuntu:** `fd` is packaged as `fd-find` (binary: `fdfind`). The script installs the correct package name and the `yame init` wrapper handles the binary name difference automatically.
- **macOS:** Requires [Homebrew](https://brew.sh/) for optional tool installation. The script checks for it and skips if missing.
- **Windows:** Not supported by `install.sh`. Use `install.ps1` instead (see below).

---

## Shell Integration

To add fuzzy-file-finding to yame, add this line to your `.zshrc` or `.bashrc`:

```sh
eval "$(yame init)"
```

Or let `install.sh` do it. To specify your shell explicitly:

```sh
eval "$(yame init zsh)"
eval "$(yame init bash)"
```

Once active, `yame <term>` fuzzy-searches filenames and `yame` with no argument fuzzy-finds Markdown files. Requires `fd` and `fzf`. See [FRIENDS.md](FRIENDS.md) for details.

---

## Config Template

```sh
yame write-config
```

Writes a fully-commented `~/.config/yame/config.toml` (or prompts before overwriting an existing one). Every option is explained inline — open the file in yame itself to read and edit it.

---

## lf Setup

After installing `lf` (see [FRIENDS.md](FRIENDS.md)), configure it to open files with yame and use `yame --preview` for all file previews:

```sh
mkdir -p ~/.config/lf

# Add opener and previewer to lfrc
echo 'cmd open $yame "$f"' >> ~/.config/lf/lfrc
echo 'set previewer ~/.config/lf/preview' >> ~/.config/lf/lfrc

# Write the preview script
cat > ~/.config/lf/preview << 'EOF'
#!/usr/bin/env bash
COLUMNS="$2" yame --preview "$1"
EOF

chmod +x ~/.config/lf/preview
```

After this, `Ctrl+O` inside yame opens lf, and all files get yame's decorated preview in the lf pane — Markdown with full inline decoration, everything else with syntax highlighting.

---

## install.ps1 (Windows)

Run from PowerShell in the repository root:

```powershell
.\install.ps1
```

If scripts are blocked by execution policy, run once before running the script:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### What it does

1. **winget check** — warns gracefully if winget isn't available (ships with Windows 10 21H2+ and Windows 11)
2. **Rust toolchain** — installs via winget (`Rustlang.Rustup`) or falls back to downloading `rustup-init.exe` directly
3. **Optional dependencies** — offers to install `fd`, `fzf`, and `lf` via winget
4. **yame** — runs `cargo install yame`
5. **PowerShell wrapper function** — offers to add a native PowerShell `yame` wrapper to your `$PROFILE` (since `yame init` outputs bash/zsh syntax only); the wrapper provides the same fuzzy-file-finding behavior
6. **Git Bash note** — if Git Bash is detected, prints the `eval "$(yame init)"` line to add to `~/.bashrc`
7. **Config template** — offers to run `yame write-config` (writes to `%APPDATA%\yame\config.toml`)
8. **lf setup** — if lf is installed, offers to write `%APPDATA%\lf\lfrc` and a `preview.bat` that calls `yame --preview` for all files

### PowerShell shell integration (manual)

If you want to add the wrapper without running the script, add this to your `$PROFILE`:

```powershell
function Invoke-Yame {
    if ($args.Count -ne 1 -or $args[0] -match '^-') { & yame.exe @args; return }
    $term = $args[0]
    if (Test-Path $term) { & yame.exe $term; return }
    $haveFd  = $null -ne (Get-Command fd  -ErrorAction SilentlyContinue)
    $haveFzf = $null -ne (Get-Command fzf -ErrorAction SilentlyContinue)
    if (-not $haveFd -or -not $haveFzf) { & yame.exe @args; return }
    $selected = fd --type f --extension md $term 2>$null | fzf --select-1 --exit-0
    if (-not $selected) {
        $selected = fd --type f --hidden -E node_modules -E .git -E target $term 2>$null |
                    fzf --select-1 --exit-0
    }
    if ($selected) { & yame.exe $selected }
    else {
        $ans = Read-Host "yame: no file matching '$term' found. Open new file? [y/N]"
        if ($ans -match '^[Yy]$') { & yame.exe $term }
    }
}
Set-Alias -Name yame -Value Invoke-Yame -Scope Global -Force
```

> The function calls `yame.exe` explicitly to bypass the alias when invoking the real binary.
