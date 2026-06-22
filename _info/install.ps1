#Requires -Version 5.1
<#
.SYNOPSIS
    yame installer for Windows.

.DESCRIPTION
    Installs Rust/Cargo (if missing), yame, and optional companion tools
    (fd, fzf, lf, bat) via winget. Offers to configure PowerShell shell
    integration and lf file-manager integration.

.NOTES
    Run from an elevated PowerShell prompt, or from a standard prompt if
    winget is available without elevation.

    If you see "running scripts is disabled", run once:
        Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

.EXAMPLE
    .\install.ps1
    # Run interactively — prompts before each optional step.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function Write-Info    ([string]$Msg) { Write-Host "  $Msg" -ForegroundColor Cyan }
function Write-Success ([string]$Msg) { Write-Host "  $Msg" -ForegroundColor Green }
function Write-Warn    ([string]$Msg) { Write-Host "  ! $Msg" -ForegroundColor Yellow }
function Write-Header  ([string]$Msg) { Write-Host "`n$Msg" -ForegroundColor White }

function Ask ([string]$Prompt) {
    $answer = Read-Host "$Prompt [y/N]"
    return $answer -match '^[Yy]$'
}

function Refresh-Path {
    # Pull any PATH changes made by installers into the current session.
    $env:PATH = [System.Environment]::GetEnvironmentVariable('PATH', 'Machine') + ';' +
                [System.Environment]::GetEnvironmentVariable('PATH', 'User')
}

function Test-Command ([string]$Name) {
    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

# ---------------------------------------------------------------------------
# 0. Execution policy reminder
# ---------------------------------------------------------------------------

Write-Header '=== yame installer for Windows ==='
Write-Host ''
Write-Host '  If this script was blocked by execution policy, run:'
Write-Host '    Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser'
Write-Host '  then re-run this script.'
Write-Host ''

# ---------------------------------------------------------------------------
# 1. Check for winget
# ---------------------------------------------------------------------------

Write-Header '--- Package manager'

$HaveWinget = Test-Command 'winget'

if ($HaveWinget) {
    Write-Success 'winget found'
} else {
    Write-Warn 'winget not found.'
    Write-Warn 'winget ships with Windows 10 21H2+ and Windows 11.'
    Write-Warn 'If missing, install "App Installer" from the Microsoft Store,'
    Write-Warn 'or install optional tools manually — see _info\FRIENDS.md.'
    Write-Host ''
    Write-Warn 'Continuing without winget — only yame itself will be installed.'
}

function Install-WithWinget ([string]$Id, [string]$DisplayName) {
    if (-not $HaveWinget) {
        Write-Warn "Skipping $DisplayName — winget not available."
        return $false
    }
    Write-Info "Installing $DisplayName via winget..."
    winget install --id $Id --silent --accept-package-agreements --accept-source-agreements
    Refresh-Path
    return $true
}

# ---------------------------------------------------------------------------
# 2. Rust / Cargo
# ---------------------------------------------------------------------------

Write-Header '--- Rust toolchain'

if (Test-Command 'cargo') {
    $ver = & cargo --version 2>$null
    Write-Success "Cargo already installed ($ver)"
} else {
    Write-Info 'Cargo not found. Installing Rust toolchain...'
    if ($HaveWinget) {
        winget install --id Rustlang.Rustup --silent --accept-package-agreements --accept-source-agreements
        Refresh-Path
        # rustup may need the user to restart; try sourcing env
        $CargoEnv = "$env:USERPROFILE\.cargo\env.ps1"
        if (Test-Path $CargoEnv) { . $CargoEnv }
        Refresh-Path
    } else {
        Write-Info 'Downloading rustup-init.exe...'
        $RustupPath = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $RustupPath
        & $RustupPath -y --quiet
        Remove-Item $RustupPath -ErrorAction SilentlyContinue
        Refresh-Path
    }
    if (Test-Command 'cargo') {
        Write-Success "Rust installed ($(& cargo --version 2>$null))"
    } else {
        Write-Warn 'cargo still not in PATH. You may need to restart your terminal.'
        Write-Warn 'After restarting, re-run this script or run: cargo install yame'
    }
}

# ---------------------------------------------------------------------------
# 3. Optional tools
# ---------------------------------------------------------------------------

Write-Header '--- Optional tools'
Write-Host '  fd  — fast file finder  (used by the yame PowerShell wrapper)'
Write-Host '  fzf — fuzzy finder      (used by the yame PowerShell wrapper + Ctrl+O)'
Write-Host '  lf  — file manager      (Ctrl+O inside yame opens lf directly)'
Write-Host ''

$InstallFd  = $false
$InstallFzf = $false
$InstallLf  = $false

if (Test-Command 'fd') {
    Write-Success 'fd already installed'
} else {
    $InstallFd = Ask 'Install fd?'
}

if (Test-Command 'fzf') {
    Write-Success 'fzf already installed'
} else {
    $InstallFzf = Ask 'Install fzf?'
}

if (Test-Command 'lf') {
    Write-Success 'lf already installed'
} else {
    $InstallLf = Ask 'Install lf?'
}

if ($InstallFd)  { Install-WithWinget 'sharkdp.fd'   'fd'  | Out-Null }
if ($InstallFzf) { Install-WithWinget 'junegunn.fzf'  'fzf' | Out-Null }
if ($InstallLf)  { Install-WithWinget 'gokcehan.lf'   'lf'  | Out-Null }

# ---------------------------------------------------------------------------
# 4. Install yame
# ---------------------------------------------------------------------------

Write-Header '--- Installing yame'

if (-not (Test-Command 'cargo')) {
    Write-Warn 'cargo is not available — cannot install yame.'
    Write-Warn 'Restart your terminal to pick up Rust, then run: cargo install yame'
    exit 1
}

Write-Info 'Running: cargo install yame'
cargo install yame
Refresh-Path

if (Test-Command 'yame') {
    Write-Success "yame installed ($(& yame --version 2>$null))"
} else {
    Write-Warn 'yame binary not found in PATH after install.'
    Write-Warn "Add $env:USERPROFILE\.cargo\bin to your PATH and try again."
}

# ---------------------------------------------------------------------------
# 5. PowerShell shell integration
# ---------------------------------------------------------------------------

Write-Header '--- PowerShell shell integration'
Write-Host '  Adds a "yame" wrapper function to your PowerShell profile so that:'
Write-Host '    yame         — fuzzy-finds Markdown files in the current directory'
Write-Host '    yame <term>  — fuzzy-searches for files matching the term'
Write-Host '    yame <file>  — opens that file directly (path pass-through)'
Write-Host '  Requires fd and fzf. Falls back to plain yame if either is missing.'
Write-Host ''

# The function calls yame.exe explicitly to avoid recursing into itself.
$YameFunction = @'

# yame shell integration — fuzzy file finding
# Requires fd and fzf. Generated by _info\install.ps1
function Invoke-Yame {
    # Pass flags and multi-arg invocations straight through.
    if ($args.Count -ne 1 -or $args[0] -match '^-') {
        & yame.exe @args
        return
    }

    $term = $args[0]

    # Direct path — open immediately.
    if (Test-Path $term) {
        & yame.exe $term
        return
    }

    # Need fd + fzf for fuzzy finding.
    $haveFd  = $null -ne (Get-Command fd  -ErrorAction SilentlyContinue)
    $haveFzf = $null -ne (Get-Command fzf -ErrorAction SilentlyContinue)
    if (-not $haveFd -or -not $haveFzf) {
        & yame.exe @args
        return
    }

    # Tier 1: fuzzy-find a Markdown file matching the term.
    $selected = fd --type f --extension md $term 2>$null |
                fzf --select-1 --exit-0 --preview 'head -20 {}'

    # Tier 2: any file (hidden included, heavy dirs excluded).
    if (-not $selected) {
        $selected = fd --type f --hidden -E node_modules -E .git -E target $term 2>$null |
                    fzf --select-1 --exit-0 --preview 'head -20 {}'
    }

    if ($selected) {
        & yame.exe $selected
    } else {
        $ans = Read-Host "yame: no file matching '$term' found. Open new file? [y/N]"
        if ($ans -match '^[Yy]$') { & yame.exe $term }
    }
}

Set-Alias -Name yame -Value Invoke-Yame -Scope Global -Force
'@

# Prefer PowerShell 7 profile if available, else Windows PowerShell.
if ($PSVersionTable.PSEdition -eq 'Core') {
    $ProfilePath = $PROFILE.CurrentUserAllHosts
} else {
    $ProfilePath = $PROFILE.CurrentUserCurrentHost
}

if (Ask "Add shell integration to $ProfilePath?") {
    # Create profile directory if it doesn't exist.
    $ProfileDir = Split-Path $ProfilePath -Parent
    if (-not (Test-Path $ProfileDir)) {
        New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null
    }

    # Idempotency check — don't add twice.
    if ((Test-Path $ProfilePath) -and (Select-String -Path $ProfilePath -Pattern 'yame shell integration' -Quiet)) {
        Write-Success "Shell integration already present in $ProfilePath"
    } else {
        Add-Content -Path $ProfilePath -Value $YameFunction
        Write-Success "Added to $ProfilePath"
        Write-Info "Reload with:  . `$PROFILE"
    }
}

# ---------------------------------------------------------------------------
# 5b. Git Bash integration (bonus note)
# ---------------------------------------------------------------------------

if (Test-Command 'bash') {
    Write-Host ''
    Write-Info 'Git Bash detected. To enable shell integration in Git Bash as well,'
    Write-Info 'add this line to ~/.bashrc:'
    Write-Host '    eval "$(yame init)"' -ForegroundColor DarkCyan
}

# ---------------------------------------------------------------------------
# 6. Config template
# ---------------------------------------------------------------------------

Write-Header '--- Config file'

$ConfigPath = Join-Path $env:APPDATA 'yame\config.toml'
Write-Host "  Config location: $ConfigPath"
Write-Host ''

if (Test-Path $ConfigPath) {
    Write-Success 'Config already exists'
} elseif (Test-Command 'yame') {
    if (Ask 'Generate a default config file?') {
        & yame write-config
    }
}

# ---------------------------------------------------------------------------
# 7. lf setup
# ---------------------------------------------------------------------------

if (Test-Command 'lf') {
    Write-Header '--- lf configuration'
    Write-Host '  Configure lf to open files with yame and use yame --preview for all file previews.'
    Write-Host ''

    if (Ask 'Set up lf config for yame?') {
        $LfConfigDir = Join-Path $env:APPDATA 'lf'
        $LfRc        = Join-Path $LfConfigDir 'lfrc'
        $Preview     = Join-Path $LfConfigDir 'preview.bat'

        if (-not (Test-Path $LfConfigDir)) {
            New-Item -ItemType Directory -Path $LfConfigDir -Force | Out-Null
        }

        # lfrc — opener and previewer
        $OpenerLine   = 'cmd open $yame "$f"'
        $PreviewerLine = "set previewer $Preview"

        $ExistingRc = if (Test-Path $LfRc) { Get-Content $LfRc -Raw } else { '' }

        if ($ExistingRc -notmatch 'cmd open') {
            Add-Content -Path $LfRc -Value $OpenerLine
            Write-Info "Added opener to $LfRc"
        } else {
            Write-Warn "Opener already in $LfRc — skipping"
        }

        if ($ExistingRc -notmatch 'set previewer') {
            Add-Content -Path $LfRc -Value $PreviewerLine
            Write-Info "Added previewer to $LfRc"
        } else {
            Write-Warn "Previewer already in $LfRc — skipping"
        }

        # preview.bat — called by lf with: preview.bat <file> <width> <height> <x> <y>
        if (-not (Test-Path $Preview)) {
            $PreviewBat = @'
@echo off
setlocal
set "FILE=%~1"
set "WIDTH=%~2"
set COLUMNS=%WIDTH%
yame --preview "%FILE%"
endlocal
'@
            Set-Content -Path $Preview -Value $PreviewBat -Encoding ASCII
            Write-Success "Created $Preview"
        } else {
            Write-Warn "$Preview already exists — skipping"
        }

        Write-Success 'lf configured. Press Ctrl+O inside yame to open lf.'
    }
}

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

Write-Header '=== Done! ==='
Write-Host ''
Write-Host '  Open a file:     ' -NoNewline; Write-Host 'yame README.md' -ForegroundColor Cyan
Write-Host '  Untitled buffer: ' -NoNewline; Write-Host 'yame' -ForegroundColor Cyan
Write-Host '  Help:            ' -NoNewline; Write-Host 'yame --help' -ForegroundColor Cyan
Write-Host '  Key reference:   Press ' -NoNewline; Write-Host 'F1' -ForegroundColor Cyan -NoNewline; Write-Host ' inside yame'
Write-Host ''
Write-Host '  Recommended: Windows Terminal with a Nerd Font for best rendering.'
Write-Host '  See _info\NERD-FONTS.md for font setup instructions.'
Write-Host ''
