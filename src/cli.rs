/// CLI argument parsing, shell integration, and config-write subcommand.
///
/// Extracted from `main.rs` so the entry-point file stays as a thin
/// orchestration layer (`setup_panic_hook` → `run` → `main`).
use std::io;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Subcommand type
// ---------------------------------------------------------------------------

pub(super) enum Command {
    /// Open a file for editing, or start with an untitled buffer (`path = None`).
    Edit { path: Option<PathBuf>, read_only: bool },
    Init { shell: Option<String> },
    WriteConfig,
}

// ---------------------------------------------------------------------------
// Shell integration
// ---------------------------------------------------------------------------

/// Returns the shell wrapper function string for `eval "$(yame init)"`.
///
/// The output is identical for bash and zsh — the function body uses constructs
/// common to both.  The `shell` argument is accepted for forward-compatibility
/// and explicit dotfile documentation; it does not change the output today.
pub(super) fn shell_init_str(_shell: &str) -> String {
    r#"yame() {
  if (( $# != 1 )) || [[ "$1" =~ ^- ]]; then
    command yame "$@"
    return
  fi

  # If the argument is an exact path, open it directly.
  if [[ -f "$1" ]]; then
    command yame "$1"
    return
  fi

  # Require fd and fzf; fall back to plain invocation if either is missing.
  if ! command -v fd &>/dev/null || ! command -v fzf &>/dev/null; then
    command yame "$@"
    return
  fi

  local target

  # Tier 1: fuzzy-find a Markdown file.
  target=$(fd --type f --extension md "$1" 2>/dev/null | fzf --select-1 --exit-0 --preview 'head -20 {}')

  # Tier 2: fuzzy-find any file (including hidden), skipping heavy directories.
  if [[ -z "$target" ]]; then
    target=$(fd --type f --hidden -E "node_modules" -E ".git" -E "target" "$1" 2>/dev/null | fzf --select-1 --exit-0 --preview 'head -20 {}')
  fi

  if [[ -n "$target" ]]; then
    command yame "$target"
  else
    printf "yame: no file matching '%s' found. Open new file? [y/N] " "$1" >&2
    read -r ans && [[ "$ans" =~ ^[Yy]$ ]] && command yame "$1"
  fi
}"#
    .to_string()
}

#[mutants::skip] // Reads $SHELL env var and calls process::exit — not unit-testable.
pub(super) fn detect_shell() -> String {
    match std::env::var("SHELL") {
        Ok(path) if path.contains("zsh") => "zsh".to_string(),
        Ok(path) if path.contains("bash") => "bash".to_string(),
        Ok(path) => {
            eprintln!("================================================================");
            eprintln!("yame init: unsupported shell.");
            eprintln!("================================================================");
            eprintln!("'yame init' currently only supports Bash and Zsh.");
            eprintln!();
            eprintln!("Detected shell: {path}");
            eprintln!("To set up yame's shell integration manually, see:");
            eprintln!("  https://github.com/cyrusae/yame");
            eprintln!("================================================================");
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("================================================================");
            eprintln!("yame init: $SHELL is unset.");
            eprintln!("================================================================");
            eprintln!("Pass the shell name explicitly:");
            eprintln!("  eval \"$(yame init bash)\"");
            eprintln!("  eval \"$(yame init zsh)\"");
            eprintln!("================================================================");
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Help text
// ---------------------------------------------------------------------------

#[mutants::skip] // Prints to stdout and calls process::exit — not unit-testable.
pub(super) fn print_help() {
    println!("yame — yet another markdown editor");
    println!();
    println!("USAGE");
    println!("  yame <file>           Open <file> for editing (created if it doesn't exist)");
    println!("  yame -r <file>        Open <file> in read-only mode (no edits, no save)");
    println!("  yame init             Print shell integration function (eval in .bashrc/.zshrc)");
    println!("  yame write-config     Write default config to ~/.config/yame/config.toml");
    println!(
        "  yame --version        Print version
  yame --help           Show this help"
    );
    println!();
    println!("KEYBINDINGS");
    println!("  Ctrl+S  Save          Ctrl+Z  Undo        Ctrl+C  Copy selection");
    println!("  Ctrl+X  Exit          Ctrl+Y  Redo        Ctrl+V  Paste");
    println!("  Ctrl+R  Reload config");
    println!("  Arrow keys · Home/End · PgUp/PgDn · mouse click / drag / scroll");
    println!();
    #[cfg(not(windows))]
    println!("CONFIG  ~/.config/yame/config.toml  (respects $XDG_CONFIG_HOME)");
    #[cfg(windows)]
    println!(r"CONFIG  %APPDATA%\yame\config.toml");
    println!();
    println!("  https://github.com/cyrusae/yame");
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

/// Returns the version string printed by `--version` / `-V`.
///
/// Separate from `parse_args` so it can be unit-tested without spawning a process.
pub(super) fn version_string() -> String {
    format!("yame {}", env!("CARGO_PKG_VERSION"))
}

#[mutants::skip] // Reads std::env::args() — side-effectful, not unit-testable.
pub(super) fn parse_args() -> Result<Command, ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        std::process::exit(0);
    }

    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("{}", version_string());
        std::process::exit(0);
    }

    match args.as_slice() {
        // No arguments → open an untitled buffer.
        [] => Ok(Command::Edit {
            path: None,
            read_only: false,
        }),
        [a] if a == "init" => Ok(Command::Init { shell: None }),
        [a, s] if a == "init" => Ok(Command::Init {
            shell: Some(s.clone()),
        }),
        [a] if a == "write-config" => Ok(Command::WriteConfig),
        [path] if !path.starts_with('-') => Ok(Command::Edit {
            path: Some(PathBuf::from(path)),
            read_only: false,
        }),
        [flag, path] if (flag == "-r" || flag == "--read-only") && !path.starts_with('-') => {
            Ok(Command::Edit {
                path: Some(PathBuf::from(path)),
                read_only: true,
            })
        }
        [path, flag] if (flag == "-r" || flag == "--read-only") && !path.starts_with('-') => {
            Ok(Command::Edit {
                path: Some(PathBuf::from(path)),
                read_only: true,
            })
        }
        _ => {
            eprintln!("error: unexpected arguments");
            eprintln!("Run 'yame --help' for usage.");
            Err(())
        }
    }
}

// ---------------------------------------------------------------------------
// write-config subcommand
// ---------------------------------------------------------------------------

#[mutants::skip] // Filesystem + stdin I/O — not unit-testable.
pub(super) fn run_write_config() {
    use std::io::Write;
    use yame::config::{DEFAULT_CONFIG_TEMPLATE, config_path};

    let path = config_path();

    if path.exists() {
        print!(
            "Config already exists at {}.\nOverwrite? [y/N] ",
            path.display()
        );
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("error: could not read input");
            std::process::exit(1);
        }
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted — config unchanged.");
            return;
        }
    }

    if let Some(dir) = path.parent()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        eprintln!("error: could not create config directory: {e}");
        std::process::exit(1);
    }

    match std::fs::write(&path, DEFAULT_CONFIG_TEMPLATE) {
        Ok(()) => println!("Config written to {}", path.display()),
        Err(e) => {
            eprintln!("error: could not write config: {e}");
            std::process::exit(1);
        }
    }
}
