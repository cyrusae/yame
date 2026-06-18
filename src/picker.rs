/// File-picker integration for Ctrl+O.
///
/// Suspends the TUI, runs an external picker, resumes the TUI, and returns the
/// selected path (or `None` if the user cancelled).
///
/// Picker resolution order:
/// 1. `$YAME_PICKER` — run as a shell command; the command must print the
///    selected path to stdout.
/// 2. `lf` — invoked with `-selection-path <tempfile>`; yame reads the file
///    after lf exits.
/// 3. `fzf` — invoked via `sh -c "find . -type f | fzf"`, selection on stdout.
/// 4. None of the above → an `io::Error` is returned and the caller shows a
///    status-bar message.
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

/// Suspend the TUI, open the configured file picker, resume the TUI, and
/// return the path the user selected (or `None` on cancel / empty selection).
///
/// On error the TUI is still restored before the error is propagated.
#[mutants::skip] // Subprocess + terminal I/O — not unit-testable.
pub(super) fn pick_file() -> io::Result<Option<PathBuf>> {
    // ── Suspend TUI ────────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    let result = run_picker_command();

    // ── Resume TUI (always, even on error) ────────────────────────────────
    let _ = enable_raw_mode();
    let _ = execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture);

    result
}

/// Dispatch to the appropriate picker backend.
#[mutants::skip]
fn run_picker_command() -> io::Result<Option<PathBuf>> {
    // Priority 1: explicit user-configured picker.
    if let Ok(cmd) = std::env::var("YAME_PICKER") {
        return run_stdout_picker(&cmd);
    }

    // Priority 2: lf (TUI file manager).
    if command_exists("lf") {
        return run_lf();
    }

    // Priority 3: fzf (fuzzy finder).
    if command_exists("fzf") {
        return run_stdout_picker("find . -type f -not -path '*/.*' 2>/dev/null | fzf");
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No file picker found. Install lf or fzf, or set $YAME_PICKER.",
    ))
}

/// Run a shell command, capture its stdout, return the first non-empty trimmed
/// line as a path (or `None` if the output is empty / the command was
/// cancelled).
#[mutants::skip]
fn run_stdout_picker(sh_cmd: &str) -> io::Result<Option<PathBuf>> {
    let output = Command::new("sh")
        .args(["-c", sh_cmd])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    let selected = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| PathBuf::from(l.trim()));

    Ok(selected)
}

/// Run `lf` with a temporary selection-path file and return whatever lf wrote
/// to it (or `None` if the file is empty / lf was cancelled).
#[mutants::skip]
fn run_lf() -> io::Result<Option<PathBuf>> {
    let tmp = std::env::temp_dir().join(format!("yame-picker-{}.txt", std::process::id()));

    Command::new("lf")
        .args([
            "-selection-path",
            tmp.to_str().unwrap_or("/tmp/yame-picker.txt"),
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    let selected = if tmp.exists() {
        let raw = std::fs::read_to_string(&tmp)?;
        let _ = std::fs::remove_file(&tmp);
        let line = raw.lines().find(|l| !l.trim().is_empty());
        line.map(|l| PathBuf::from(l.trim()))
    } else {
        None
    };

    Ok(selected)
}

/// Return `true` if `name` resolves to an executable in `$PATH`.
fn command_exists(name: &str) -> bool {
    which(name).is_some()
}

/// Minimal `which` — checks each directory in `$PATH` for `name`.
fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests (pure-logic parts only — I/O paths are all #[mutants::skip])
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // `command_exists` must return false for a name that cannot possibly exist.
    // Kills: `which(...).is_some()` → `which(...).is_none()` (invert).
    #[test]
    fn command_exists_returns_false_for_nonexistent_tool() {
        assert!(
            !command_exists("__yame_nonexistent_binary_xyzzy__"),
            "command_exists must return false for a binary that cannot be in PATH"
        );
    }

    // `which` must return None for a definitely-absent binary.
    // Kills: early-return None removed / path search skipped.
    #[test]
    fn which_returns_none_for_nonexistent_binary() {
        assert_eq!(
            which("__yame_nonexistent_binary_xyzzy__"),
            None,
            "which must return None when binary is not in PATH"
        );
    }

    // `which` must find `sh` (present on every Unix-like OS we target).
    // Kills: returning None unconditionally.
    #[cfg(unix)]
    #[test]
    fn which_finds_sh_on_unix() {
        assert!(
            which("sh").is_some(),
            "which must find `sh` on Unix — it is always in PATH"
        );
    }

    // Kills: replace command_exists -> bool with false.
    // The false-constant mutant passes the nonexistent-tool test but fails here.
    #[cfg(unix)]
    #[test]
    fn command_exists_returns_true_for_sh() {
        assert!(
            command_exists("sh"),
            "command_exists must return true for `sh`, which is always in PATH on Unix"
        );
    }
}
