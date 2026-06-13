# Automation Scripts

To simplify setup, we provide an automated installer script that handles dependencies and gets you up and running with one command.

---

## 1. The Installation Script

You can find the installation script at [install.sh](file:///Users/watcher/githere/yame/_info/install.sh).

### What it does:
1. **OS Detection:** Identifies whether you are running macOS or Linux.
2. **Rust Toolchain:** Checks for Cargo. If missing, it installs the official Rust toolchain via `rustup.rs`.
3. **Optional Dependencies:** Detects the package manager (Homebrew on Mac; `apt`, `dnf`, or `pacman` on Linux) and installs `fzf` and `fd` for fuzzy file searching.
4. **Editor Compilation:** Runs `cargo install yame` to compile and install the editor binary.

### How to run it:
From the root of the `yame` repository, execute:
```sh
chmod +x _info/install.sh
./_info/install.sh
```

---

## 2. Shell Integration Wrapper

To set up fuzzy-finding integrations, run:
```sh
yame init
```
This prints shell functions tailored to Bash or Zsh. You can append this directly to your shell startup file:

```sh
echo 'eval "$(yame init)"' >> ~/.zshrc  # For Zsh users
echo 'eval "$(yame init)"' >> ~/.bashrc  # For Bash users
```
For a breakdown of what this integration function does (or if you want to write a custom version for other shells like Fish or NuShell), read the internal development notes in [SHELL INTENTIONS.md](file:///Users/watcher/githere/yame/_docs/SHELL%20INTENTIONS.md).
