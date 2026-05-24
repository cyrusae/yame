# `yame init` helper

Goal: Bash and Zsh (maybe Fish?) users can run `yame init` to create a helper function for their .bashrc/.zshrc/etc. that uses fzf/fd if available for fuzzy-finding logic.

## Order of operations

`yame` with no arguments should show help/intro to program (including `yame init` suggestion).

`yame illegal arguments multiple of them` should error.

`yame README.md` opens README.md.

`yame readm` opens README.md.

`yame docs/d` opens _docs/DESIGN.md.

## Draft code

```zsh
yame() {
  if (( $# != 1 )) || [[ "$1" =~ ^- ]]; then
    command yame "$@"
    return
  fi

  # If it exists exactly as typed (e.g., yame .gitignore or yame CHANGELOG)
  if [[ -f "$1" ]]; then
    command yame "$1"
    return
  fi

  local target

  # Tier 1: Try to find a matching Markdown file first (keeps it clean)
  target=$(fd --type f --extension md "$1" 2>/dev/null | fzf --select-1 --exit-0 --preview 'head -20 {}')

  # Tier 2: If no markdown file matched, search ANY text/extensionless file
  if [[ -z "$target" ]]; then
    # --type f ensures we don't grab directories. 
    # We filter out common heavy/binary folders just in case.
    target=$(fd --type f -E "node_modules/*" -E ".git/*" -E "target/*" "$1" 2>/dev/null | fzf --select-1 --exit-0 --preview 'head -20 {}')
  fi

  # Execution
  if [[ -n "$target" ]]; then
    command yame "$target"
  else
    # Fallback to native creation if both searches came up empty
    command yame "$1"
  fi
}
```

## Detecting shell in `yame init`

```rust
use std::env;
use std::process::exit;

enum Shell {
    Bash,
    Zsh,
}

fn detect_shell() -> Shell {
    // Look at the $SHELL environment variable
    match env::var("SHELL") {
        Ok(path) => {
            if path.contains("zsh") {
                Shell::Zsh
            } else if path.contains("bash") {
                Shell::Bash
            } else {
                // It's Fish, Nu, or something else
                eprintln!("Error: yame init only supports Bash and Zsh.");
                eprintln!("Your current shell profile ({}) is not supported yet.", path);
                exit(1);
            }
        }
        Err(_) => {
            eprintln!("Error: Could not detect your shell environment ($SHELL is unset).");
            exit(1);
        }
    }
}
```

Except improve the text--something more like

- [ ] Have a document for shell setup as a README companion
- [ ] This:

```rust
eprintln!("================================================================");
eprintln!("yame init error: Unsupported shell profile.");
eprintln!("================================================================");
eprintln!("'yame init' currently only auto-generates wrappers for Bash and Zsh.");
eprintln!();
eprintln!("To set up yame's intelligent fuzzy-finding features manually in ");
eprintln!("your shell, please visit the configuration guide:");
eprintln!("  https://github.com/cyrusae/yame/LINKTOSHELLFILEORSECTIONOFREADME");
eprintln!("================================================================");
std::process::exit(1);
```

Maybe section of readme for "how to run `yame init` and then a separate file breaking down what the script actually is (for power users/suspicious minds and for people who want to replicate it for their own shell)?
