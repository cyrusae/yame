# `yame init` helper

Goal: Bash and Zsh (maybe Fish?) users can run `yame init` to create a helper function for their .bashrc/.zshrc/etc. that uses fzf/fd if available for fuzzy-finding logic.

## Order of operations

`yame` with no arguments should show help/intro to program (including `yame init` suggestion).

`yame illegal arguments multiple of them` should error.

`yame README.md` opens README.md.

`yame .gitignore` opens .gitignore (I think I might have to take the hit of `yame giti` not doing so?).

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

---

## Review notes (pre-implementation)

### What's working

The tiered search strategy (MD-first → any file) is the right shape. `fzf --select-1 --exit-0` is correct — auto-selects on one match, silently exits on zero. `command yame` to bypass the wrapper is correct. The bones are solid; what follows is hardening, not rethinking.

### Must-fix before shipping

**1. No `fd`/`fzf` availability check**
If either tool is missing the function silently falls through to creating a new file. Add a guard before the fuzzy tiers:

```zsh
if ! command -v fd &>/dev/null || ! command -v fzf &>/dev/null; then
  command yame "$@"; return   # or warn and exit 1
fi
```

**Fix this!** That got eaten in a rewrite.

**2. Hidden files won't be found**
`fd` skips hidden and gitignored files by default, so `.gitignore`, `.env`, etc. can't be found in tier 2 — `yame giti` would fall through to silently creating a file named `giti`. The doc flags this as a known concern but doesn't resolve it. Options:

- Add `--hidden` to tier 2 and tighten the `-E` exclusions (see below)
- Or add a tier 2.5 explicit hidden-file pass

**3. Fallback silently creates a new file**
`yame readm` with no matches → creates a file literally named `readm`. Consider a prompt or at least a message before creating. Possible pattern:

```zsh
else
  printf "yame: no file matching '%s' found. Open new file? [y/N] " "$1" >&2
  read -r ans && [[ "$ans" =~ ^[Yy]$ ]] && command yame "$1"
fi
```

> **TODO:** Debate this versus intended file creation behavior. Maybe alert on files without a .md type?

**4. `-E "target/*"` glob syntax is wrong**
`fd -E "target/*"` means "exclude files named target/*", not "exclude the target directory." The correct form is `-E "target"` — fd handles recursive exclusion from there. In practice this probably doesn't matter (fd respects `.gitignore` which already covers `target/`), but the intent is misleading.

### Lower priority / design choices

**`$SHELL` detects login shell, not running shell**
If the user's login shell is bash but they're invoking this from zsh, `$SHELL` says bash and they get the wrong output. This is a known limitation of the `$SHELL`-based approach (zoxide has the same one). Mitigation: accept an explicit argument (`yame init zsh`, `yame init bash`) and use `$SHELL` only as a fallback.

**Single function body works for both bash and zsh**
The draft function is bash-compatible as written — `(( ))`, `[[ ]]`, `=~`, and `command` all work in both. You may not need separate bash/zsh output at all. `eval "$(yame init)"` works in both shells.

### Docs structure recommendation

- **README**: Short section — `eval "$(yame init)"` and add the output to your rc. Requires `fd` and `fzf`. (Prefer `yame init bash` and `yame init zsh` explicitly? Why or why not?)
- **`_docs/SHELL.md`** (new file): Full breakdown of what the script does, tier logic, how to replicate for Fish/Nu/etc., invitation for contributions of alternate load scripts.

This matches the zoxide model (terse README, separate file for power users and suspicious minds).

---

**Addendum:** Would it make sense to expand the list of allowed fd/fzf file types to common text files: .txt, .json, .toml, .yaml (what else?) on top of .md? That seems like it could save a lot of the more expensive lookups--if someone wants `thing.type` in a dev environment it's a pretty restricted set of likely candidate types?
